use std::{
    path::PathBuf,
    process::Command,
    sync::{Arc, Mutex, RwLock},
};

use crate::{
    config::sze_config::SchedulerConfig,
    file::file_handle::{FileHander, Readable},
    logger::log_handle::LogHandle,
    utils::utils_lib::{inotify_blockage, inotify_init},
};

pub fn monitor_screen_status() -> bool {
    let result = Command::new("sh")
        .arg("-c")
        .arg("dumpsys power | grep -E \"mHoldingDisplaySuspendBlocker|mScreenOn\"")
        .output()
        .expect("Failed to execute command");
    let output = String::from_utf8_lossy(&result.stdout);
    output.contains("true")
}

pub struct SzeRs {
    mode_file: FileHander<Readable>,
    inotify: inotify::Inotify,
    scheduler_config: Arc<RwLock<SchedulerConfig>>,
    log_handle: Arc<Mutex<LogHandle>>,
}

impl SzeRs {
    pub fn new(
        path: &str,
        scheduler_config: Arc<RwLock<SchedulerConfig>>,
        log_handle: Arc<Mutex<LogHandle>>,
    ) -> Self {
        let inotify = inotify_init(&PathBuf::from(path));
        let mode_file = FileHander::open_read(path).expect("Failed to open mode file");

        Self {
            mode_file,
            inotify,
            scheduler_config,
            log_handle,
        }
    }

    pub fn sze_rs_scheduler(&mut self) {
        let mut tmp_mode = String::from("Null");
        loop {
            inotify_blockage(&mut self.inotify);
            std::thread::sleep(std::time::Duration::from_millis(500));
            self.mode_file
                .seek_to_start()
                .expect("Failed to seek to start");
            let content = self
                .mode_file
                .read_to_string()
                .expect("Failed to read mode file");
            let mode = content.trim();
            let mut scheduler_config = self.scheduler_config.write().unwrap();
            *scheduler_config = SchedulerConfig::new(mode);
            let mut log_handle = self.log_handle.lock().unwrap();
            log_handle.info(format!("Scheduler Mode From: {} To: {}", tmp_mode, mode));
            tmp_mode = String::from(mode);
        }
    }
}

