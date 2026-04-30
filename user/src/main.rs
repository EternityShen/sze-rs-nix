use std::sync::{Arc, Mutex, RwLock};
use sze_rs_nix::config::sze_config::SchedulerConfig;
use sze_rs_nix::cpu_info::cpu_handle::CpuHandle;
use sze_rs_nix::listener::load_handle::{EbpfLoadHandle, get_load_level};
use sze_rs_nix::logger::log_handle::LogHandle;
use sze_rs_nix::scheduler::cpu::{PolicyScheduler, policy_freq_scheduler};
use sze_rs_nix::scheduler::sze_rs::{SzeRs, monitor_screen_status};

fn main() {
    let mut sheneternity_log = LogHandle::new("./log/sheneternity.log");
    sheneternity_log.clear();
    sheneternity_log.info(format!("你好啊！感谢你使用我的调度！当前版本为: 0.1.0"));
    sheneternity_log.info(format!(
        "虽然我不知道你在用，但对我这个小众调度来说，有用户就是对我付出的最好答复！"
    ));
    sheneternity_log.info(format!(
        "我会持续优化我的调度，以提供更好的性能和用户体验。"
    ));
    sheneternity_log.info(format!(
        "如果有任何问题或建议，请随时联系我: sheneternity2008@outlook.com/QQ群: 1050689431"
    ));
    sheneternity_log.info(format!("好啦，感谢你能看到这，我会继续努力的！"));
    let mut user_log = LogHandle::new("./log/cpu-info.log");
    user_log.clear();
    let mut ebpf_log = LogHandle::new("./log/ebpf.log");
    ebpf_log.clear();
    let mut mode_log = LogHandle::new("./log/mode.log");
    mode_log.clear();
    let ebpf_log_handle = Arc::new(Mutex::new(ebpf_log));
    let user_log_handle = Arc::new(Mutex::new(user_log));
    let cpu_handle = Arc::new(RwLock::new(CpuHandle::new(Arc::clone(&user_log_handle))));
    let mode_log_handle = Arc::new(Mutex::new(mode_log));
    // Clone necessary data before moving cpu_handle
    let policy_index = cpu_handle.read().unwrap().policy_index;
    let policy_range_map = cpu_handle.read().unwrap().policy_range_map.clone();
    let on_or_off = Arc::new(Mutex::new(true));
    let scheduler_config = Arc::new(RwLock::new(SchedulerConfig::new("powersave")));
    for policy_idx in 0..policy_index {
        let scheduler_config_clone = Arc::clone(&scheduler_config);
        let policy_range_map_clone = policy_range_map.clone();
        let ebpf_log_handle_clone = Arc::clone(&ebpf_log_handle);
        let on_or_off = Arc::clone(&on_or_off);
        let cpu_handle = Arc::clone(&cpu_handle);
        let _ = std::thread::Builder::new()
            .name(format!("Eternity_{}", policy_idx))
            .spawn(move || {
                let mut load_handle = EbpfLoadHandle::new(ebpf_log_handle_clone);
                let policy_range = policy_range_map_clone
                    .get(&(policy_idx as u32))
                    .unwrap()
                    .clone();
                loop {
                    let on_or_off = on_or_off.lock().unwrap().clone();
                    if !on_or_off {
                        std::thread::sleep(std::time::Duration::from_secs(5));
                        continue;
                    }
                    let load = load_handle.read_load_policy(policy_range);
                    let load_level = get_load_level(load, Arc::clone(&scheduler_config_clone));
                    let mut cpu_handle = cpu_handle.write().unwrap();
                    let policy = cpu_handle
                        .policy_map
                        .get_mut(&(policy_idx as usize))
                        .unwrap();
                    let mut policy_scheduler = PolicyScheduler::new(policy);
                    policy_freq_scheduler(policy_idx as u32, &load_level, &mut policy_scheduler);
                    let sleep_time = scheduler_config_clone.read().unwrap().polling_interval;
                    std::thread::sleep(std::time::Duration::from_millis(sleep_time as u64));
                }
            });
    }
    let mut sze = SzeRs::new(
        "/data/adb/modules/ETERNAL_CPU_SZE/config/config.txt",
        Arc::clone(&scheduler_config),
        Arc::clone(&mode_log_handle),
    );

    let _ = std::thread::Builder::new()
        .name(format!("Eternity_Screen"))
        .spawn(move || {
            loop {
                let screen_on = monitor_screen_status();
                if screen_on {
                    *on_or_off.lock().unwrap() = true;
                } else {
                    *on_or_off.lock().unwrap() = false;
                }
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        });

    loop {
        sze.sze_rs_scheduler();
    }

    /* let mut load_landle = EbpfLoadHandle::new(Arc::new(Mutex::new(LogHandle::new("./log/log.log"))));

    loop {
        let load = load_landle.read_load_policy((0,16));
        println!("load: {}", load);
        std::thread::sleep(std::time::Duration::from_secs(1));
    } */

    /* let cpu_handle = CpuHandle::new(Arc::new(Mutex::new(LogHandle::new("./log/log.log"))));
    let mut load_handle = EbpfLoadHandle::new(Arc::new(Mutex::new(LogHandle::new("./log/log.log"))));

    for i in cpu_handle.policy_range_map.clone() {
        println!("{:?}", i);
    }

    loop {
        for (policy_index, (start, end)) in cpu_handle.policy_range_map.clone() {
            let load = load_handle.read_load_policy((start, end));
            println!("policy_index: {}, load: {}", policy_index, load);
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    } */
}
