use std::{
    path::PathBuf,
    sync::{Arc, Mutex, RwLock, mpsc::Sender},
};

use crate::{config::sze_config::SchedulerConfig, logger::log_handle::LogHandle};

use super::super::file::file_handle::{FileHander, Readable};

pub struct LoadHandle {
    load_file: FileHander<Readable>,
    total: u32,
    idle: u32,
}

use anyhow::Context;
use aya::{Ebpf, Pod, maps::Array, programs::TracePoint};
use clap::Parser;

#[derive(Debug)]
pub enum LoadLevel {
    Low,
    Mid,
    High,
    Max,
}

impl LoadHandle {
    pub fn new() -> Self {
        let load_file = match FileHander::open_read("/proc/stat") {
            Ok(file) => file,
            Err(e) => {
                panic!("打开stat文件失败: {:?}", e);
            }
        };
        Self {
            load_file,
            total: 0,
            idle: 0,
        }
    }
    pub fn read_load(&mut self) -> u32 {
        self.load_file.seek_to_start().unwrap();
        let content = self.load_file.read_to_string().unwrap();

        for line in content.lines() {
            if line.starts_with("cpu ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    // 提取用户、nice、系统、空闲时间
                    let user: u32 = parts[1].parse().unwrap_or(0);
                    let nice: u32 = parts[2].parse().unwrap_or(0);
                    let system: u32 = parts[3].parse().unwrap_or(0);
                    let idle: u32 = parts[4].parse().unwrap_or(0);

                    // 计算总CPU时间
                    let total = user + nice + system + idle;

                    let delta_total = total.saturating_sub(self.total);
                    let delta_idle = idle.saturating_sub(self.idle);

                    // 计算CPU使用率
                    let usage = if delta_total > 0 {
                        (delta_total as f32 - delta_idle as f32) / delta_total as f32
                    } else {
                        0.8
                    };

                    // 更新历史值
                    self.total = total;
                    self.idle = idle;

                    return (usage * 100.0) as u32;
                }
            }
        }
        80
    }
}

pub fn load_sender(
    send: Sender<LoadLevel>,
    sleep_time: Arc<Mutex<u64>>,
    on_or_off: Arc<Mutex<bool>>,
    scheduler_config: Arc<RwLock<SchedulerConfig>>,
) {
    let mut load_handle = LoadHandle::new();
    loop {
        if !*on_or_off.lock().unwrap() {
            std::thread::sleep(std::time::Duration::from_millis(3000));
            break;
        }
        let load = load_handle.read_load();
        let load_level = get_load_level(load, scheduler_config.clone());
        send.send(load_level).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(
            *sleep_time.lock().unwrap(),
        ));
    }
}

//————————————————————————————————————————————————————————————————————————————————————————————————————————————

#[derive(Parser)]
struct Opt {
    ebpf_obj: PathBuf,
    #[clap(short, long, default_value_t = 1)]
    interval: u64,
}

#[repr(C)]
#[derive(Copy, Debug, Clone)]
struct CpuStat {
    busy_ns: u64,
    idle_ns: u64,
    last_ts: u64,
    initialized: u8,
    _padding: [u8; 7],
}

pub struct EbpfLoadHandle {
    bpf: Ebpf,
    ncpu: u32,
    prev_snapshot: Vec<CpuStat>,
}

unsafe impl Pod for CpuStat {}

impl EbpfLoadHandle {
    pub fn new(log_handle: Arc<Mutex<LogHandle>>) -> Self {
        let mut bpf = Ebpf::load_file("./system/lib/libebpf.so")
            .context("加载eBPF对象失败")
            .unwrap();
        if let Ok(mut log) = log_handle.lock() {
            log.clear();
            log.info(format!("加载eBPF对象成功"));
        }
        let program: &mut TracePoint = bpf
            .program_mut("sched_switch")
            .context("获取sched_switch程序失败")
            .unwrap()
            .try_into()
            .unwrap();
        if let Ok(mut log) = log_handle.lock() {
            log.info(format!("加载sched_switch程序成功"));
        }
        let result = program.load().context("加载sched_switch程序失败");

        if let Err(e) = result {
            if let Ok(mut log) = log_handle.lock() {
                log.error(format!("加载sched_switch程序失败: {:?}", e));
            }
        }

        program
            .attach("sched", "sched_switch")
            .context("挂载sched_switch程序失败")
            .unwrap();

        if let Ok(mut log) = log_handle.lock() {
            log.info(format!("挂载sched_switch程序成功"));
            log.info(format!("ebpf程序初始化完毕"));
        }

        let ncpu = num_cpus::get();

        let mut prev_snapshot: Vec<CpuStat> = Vec::with_capacity(ncpu);
        for _ in 0..ncpu {
            prev_snapshot.push(CpuStat {
                busy_ns: 0,
                idle_ns: 0,
                last_ts: 0,
                initialized: 0,
                _padding: [0; 7],
            });
        }

        Self {
            bpf,
            ncpu: ncpu as u32,
            prev_snapshot,
        }
    }

    pub fn read_load(&mut self) -> u32 {
        let mut load = 0;

        // 每次读取时都从 bpf 中获取 CPU_STATS map
        let cpu_stat_map: Array<_, CpuStat> =
            Array::try_from(self.bpf.map_mut("CPU_STATS").unwrap()).unwrap();

        for cpu in 0..self.ncpu {
            let v = cpu_stat_map.get(&cpu, 0).unwrap();
            let delta_busy_ns = v
                .busy_ns
                .saturating_sub(self.prev_snapshot[cpu as usize].busy_ns);
            let delta_idle_ns = v
                .idle_ns
                .saturating_sub(self.prev_snapshot[cpu as usize].idle_ns);
            let delta_ns = delta_busy_ns.saturating_add(delta_idle_ns);
            if delta_ns > 0 {
                load += (delta_busy_ns as f32 / delta_ns as f32 * 100.0) as u32;
            }
            self.prev_snapshot[cpu as usize] = v;
        }
        load / self.ncpu
    }

    pub fn read_load_policy(&mut self, policy: (u32, u32)) -> u32 {
        let mut load = 0;
        let cpu_stat_map: Array<_, CpuStat> =
            Array::try_from(self.bpf.map_mut("CPU_STATS").unwrap()).unwrap();
        for cpu in policy.0..policy.1 {
            let v = cpu_stat_map.get(&cpu, 0).unwrap();
            let delta_busy_ns = v
                .busy_ns
                .saturating_sub(self.prev_snapshot[cpu as usize].busy_ns);
            let delta_idle_ns = v
                .idle_ns
                .saturating_sub(self.prev_snapshot[cpu as usize].idle_ns);
            let delta_ns = delta_busy_ns.saturating_add(delta_idle_ns);
            if delta_ns > 0 {
                load += (delta_busy_ns as f32 / delta_ns as f32 * 100.0) as u32;
            }
            self.prev_snapshot[cpu as usize] = v;
        }
        load / (policy.1 - policy.0)
    }
}

pub fn ebpf_load_sender(
    send: Sender<LoadLevel>,
    sleep_time: Arc<Mutex<u64>>,
    on_or_off: Arc<Mutex<bool>>,
    scheduler_config: Arc<RwLock<SchedulerConfig>>,
) {
    let mut load_handle =
        EbpfLoadHandle::new(Arc::new(Mutex::new(LogHandle::new("./log/ebpf.log"))));

    loop {
        if !*on_or_off.lock().unwrap() {
            std::thread::sleep(std::time::Duration::from_millis(3000));
            break;
        }
        let load = load_handle.read_load();
        let load_level = get_load_level(load, scheduler_config.clone());
        send.send(load_level).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(
            *sleep_time.lock().unwrap(),
        ));
    }
}

pub fn get_load_level(load: u32, scheduler_config: Arc<RwLock<SchedulerConfig>>) -> LoadLevel {
    let scheduler_config = scheduler_config.read().unwrap();
    let load_level_config = &scheduler_config.load_level_config;

    if load < load_level_config.min {
        LoadLevel::Low
    } else if load < load_level_config.mid {
        LoadLevel::Mid
    } else if load < load_level_config.high {
        LoadLevel::High
    } else {
        LoadLevel::Max
    }
}

#[test]
fn test_read_load() {
    let mut load_handle = LoadHandle::new();
    loop {
        let load = load_handle.read_load();
        println!("load: {}", load);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
