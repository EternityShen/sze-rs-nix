use crate::file::file_handle::{FileHander, Writable};

enum LogLevel {
    Debug {
        str: String
    },
    Info {
        str: String
    },
    Warn {
        str: String
    },
    Error {
        str: String
    },
}

pub struct LogHandle {
    file: FileHander<Writable>
}

fn get_log_level_str(level: LogLevel) -> String {
    match level {
        LogLevel::Debug { str } => format!("[DEBUG] {}", str),
        LogLevel::Info { str } => format!("[INFO] {}", str),
        LogLevel::Warn { str } => format!("[WARN] {}", str),
        LogLevel::Error { str } => format!("[ERROR] {}", str),
    }
}

fn get_log_time() -> String {
    let now = chrono::Local::now();
    now.format("%Y-%m-%d %H:%M:%S").to_string()
}

impl LogHandle {
    pub fn new(log_file_path: &str) -> Self {
        let result = FileHander::<Writable>::open_append(log_file_path);

        match result {
            Ok(file) => LogHandle { file },
            Err(e) => {
                eprintln!("无法打开日志文件: {}", e);
                std::process::exit(1);
            }
        }
    }

    fn log(&mut self, level: LogLevel) {
        let log_str = format!("{} {}\n", get_log_time(), get_log_level_str(level));
        if let Err(e) = self.file.write_string(format!("{}", log_str)) {
            eprintln!("写入日志文件时出错: {}", e);
        }
    }

    // 清除日志文件内容
    pub fn clear(&mut self) {
        if let Err(e) = self.file.clear() {
            eprintln!("清除日志文件时出错: {}", e);
        }
    }

    pub fn debug(&mut self, msg: String) {
        self.log(LogLevel::Debug { str: msg.to_string() });
    }

    pub fn info(&mut self, msg: String) {
        self.log(LogLevel::Info { str: msg.to_string() });
    }

    pub fn warn(&mut self, msg: String) {
        self.log(LogLevel::Warn { str: msg.to_string() });
    }

    pub fn error(&mut self, msg: String) {
        self.log(LogLevel::Error { str: msg.to_string() });
    }
}