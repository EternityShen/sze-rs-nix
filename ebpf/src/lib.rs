#![no_std]
#![no_main]

use aya_ebpf::{
    // helpers 模块：包含 eBPF helper 函数
    // 这些函数是内核提供的，用于在 eBPF 程序中执行特定操作
    helpers::{bpf_get_smp_processor_id, bpf_ktime_get_ns},
    // macros 模块：包含各种 eBPF 宏
    macros::{map, tracepoint},
    // maps 模块：包含各种 eBPF Map 类型
    // Map 是 eBPF 程序和用户空间程序之间通信的主要方式
    // 就像是一个共享的"公告板"，内核写数据，用户空间读数据
    maps::Array,
    // programs 模块：包含各种 eBPF 程序类型
    // TracePointContext 是 tracepoint 类型程序的上下文
    // 包含触发事件的详细信息
    programs::TracePointContext,
};

const MAX_CPUS: u32 = 256;

const PREV_COMM_OFFSET: usize = 8;

const PREV_PID_OFFSET: usize = 24;

const PREV_STATE_OFFSET: usize = 32;

#[repr(C)]
#[derive(Copy, Clone, Default)]

pub struct CpuStat {
    pub busy_ns: u64,

    pub idle_ns: u64,

    pub last_ts: u64,

    pub initialized: u8,

    pub _padding: [u8; 7],
}

#[map(name = "CPU_STATS")]
// 【map 宏的作用】
// 告诉编译器这是一个 eBPF Map 定义。
// name = "CPU_STATS" 指定了 Map 的名称，
// 用户空间程序通过这个名称来查找和访问这个 Map。
static mut CPU_STATS: Array<CpuStat> = Array::<CpuStat>::with_max_entries(MAX_CPUS, 0);

#[tracepoint]
pub fn sched_switch(ctx: TracePointContext) -> i32 {
    // 【返回值约定】
    // eBPF 程序返回 i32 类型：
    // - 0 表示成功
    // - 非 0 表示错误
    //
    // 【为什么用 match 包装？】
    // 因为 eBPF 程序不能 panic（会导致内核问题），
    // 所以用 Result 来处理错误，将错误转换为返回值。
    match try_sched_switch(ctx) {
        Ok(_) => 0,  // 成功返回 0
        Err(_) => 1, // 错误返回 1
    }
}

fn try_sched_switch(ctx: TracePointContext) -> Result<(), ()> {
    let cpu = unsafe { bpf_get_smp_processor_id() } as u32;

    let ts = unsafe { bpf_ktime_get_ns() };

    let stat_ptr = unsafe { CPU_STATS.get_ptr_mut(cpu) };

    // 【错误处理】
    // 如果获取指针失败（比如 CPU ID 超出范围），返回错误。
    let stat = match stat_ptr {
        Some(ptr) => unsafe { &mut *ptr }, // 解引用指针，获得可变引用
        None => return Err(()),            // 获取失败，返回错误
    };

    let prev_pid: i32 = match unsafe { ctx.read_at::<i32>(PREV_PID_OFFSET) } {
        Ok(pid) => pid, // 成功读取 PID
        Err(_) => -1,   // 读取失败，设为 -1（不会匹配 idle）
    };

    let prev_is_idle = if prev_pid == 0 {
        // PID 为 0，肯定是 idle 进程
        true
    } else {
        // PID 不为 0，需要检查命令名
        // 读取命令名（16 字节的字符数组）
        let comm_buf: [u8; 16] = match unsafe { ctx.read_at(PREV_COMM_OFFSET) } {
            Ok(buf) => buf,      // 成功读取
            Err(_) => [0u8; 16], // 读取失败，返回空数组
        };
        // 检查命令名是否以 "swapper" 或 "idle" 开头
        // starts_with 是 [u8] 的方法，用于检查字节序列是否以指定前缀开头
        // b"swapper" 是字节字符串字面量，类型是 &[u8; 7]
        comm_buf.starts_with(b"swapper") || comm_buf.starts_with(b"idle")
    };

    if stat.initialized == 0 {
        stat.last_ts = ts; // 记录当前时间戳
        stat.initialized = 1; // 标记为已初始化
        return Ok(()); // 直接返回，不计算时间间隔
    }
    let delta = ts.saturating_sub(stat.last_ts);

    // 更新上次时间戳为当前时间
    stat.last_ts = ts;

    if prev_is_idle {
        // 前一个进程是 idle，这段时间算空闲
        stat.idle_ns = stat.idle_ns.saturating_add(delta);
    } else {
        // 前一个进程不是 idle，这段时间算忙碌
        stat.busy_ns = stat.busy_ns.saturating_add(delta);
    }

    // 返回成功
    Ok(())
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // unreachable_unchecked：告诉编译器这行代码永远不会被执行
    // 返回类型 ! 表示这个函数永远不会返回（发散函数）
    unsafe { core::hint::unreachable_unchecked() }
}
