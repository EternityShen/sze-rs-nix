use crate::{
    file::file_handle::{FileHander, Writable},
    logger::log_handle::LogHandle,
    utils::utils_lib::divide_u32_vector_into_intervals_map,
    utils::utils_lib::insertion_sort,
    utils::utils_lib::policy_path_insertion_sort,
};

use std::{
    collections::HashMap,
    fs::read_dir,
    io::Error,
    path::PathBuf,
    sync::{Arc, Mutex},
};

//Policy结构体(存储最大/最小频率文件句柄,频率列表,频率区间)
pub struct Policy {
    pub freq_max_file: FileHander<Writable>,
    pub freq_min_file: FileHander<Writable>,
    pub freq_list: Vec<u32>,
    pub freq_range_map: HashMap<usize, (u32, u32)>,
}

//CpuHandle结构体(存储CPU名称,当前策略索引,Policy映射,日志句柄)
pub struct CpuHandle {
    pub cpu_name: String,
    pub policy_index: usize,
    pub policy_range_map: HashMap<u32, (u32, u32)>,
    pub policy_map: HashMap<usize, Policy>,
    pub log_handle: Arc<Mutex<LogHandle>>,
}

impl Policy {
    fn new(log_handle: Arc<Mutex<LogHandle>>, policy_path: PathBuf) -> Self {
        let mut freq_list = Vec::new();

        let max_freq_file_path = policy_path.join("scaling_max_freq");
        let min_freq_file_path = policy_path.join("scaling_min_freq");

        let freq_max_file = match FileHander::open_write(max_freq_file_path.to_str().unwrap()) {
            Ok(file) => file,
            Err(e) => {
                if let Ok(mut log_handle) = log_handle.lock() {
                    log_handle.error(format!("打开max_freq文件失败: {:?}", e));
                }
                panic!("打开max_freq文件失败");
            }
        };
        let freq_min_file = match FileHander::open_write(min_freq_file_path.to_str().unwrap()) {
            Ok(file) => file,
            Err(e) => {
                if let Ok(mut log_handle) = log_handle.lock() {
                    log_handle.error(format!("打开min_freq文件失败: {:?}", e));
                }
                panic!("打开min_freq文件失败");
            }
        };

        if let Ok(mut log_handle) = log_handle.lock() {
            log_handle.debug(format!(
                "成功打开max_freq文件,并保存为句柄: {:?}",
                max_freq_file_path
            ));
            log_handle.debug(format!(
                "成功打开min_freq文件,并保存为句柄: {:?}",
                min_freq_file_path
            ));
            log_handle.debug(format!("开始读取freq_list文件"));
        }

        let freq_list_file_path = policy_path.join("scaling_available_frequencies");

        let mut freq_list_file = match FileHander::open_read(freq_list_file_path.to_str().unwrap())
        {
            Ok(file) => file,
            Err(e) => {
                if let Ok(mut log_handle) = log_handle.lock() {
                    log_handle.error(format!("打开freq_list文件失败: {:?}", e));
                }
                panic!("打开freq_list文件失败");
            }
        };

        if let Ok(mut log_handle) = log_handle.lock() {
            log_handle.debug(format!(
                "成功打开freq_list文件,并保存为句柄: {:?}",
                freq_list_file_path
            ));
            log_handle.debug(format!(
                "开始读取freq_list文件内容,并解析为u32类型存入freq_list向量"
            ));
        }

        let freq_list_content = freq_list_file.read_to_string().unwrap();
        for line in freq_list_content.split_ascii_whitespace() {
            if let Ok(freq) = line.trim().parse::<u32>() {
                freq_list.push(freq);
            }
        }

        insertion_sort(&mut freq_list);

        if let Ok(mut log_handle) = log_handle.lock() {
            log_handle.debug(format!(
                "成功读取freq_list文件内容,并解析为u32类型存入freq_list向量"
            ));
        }

        let freq_range_map = divide_u32_vector_into_intervals_map(freq_list.clone(), 4);

        if let Ok(mut log_handle) = log_handle.lock() {
            log_handle.debug(format!(
                "成功将freq_list向量分为4个区间,并保存为freq_range_map"
            ));
        }

        Self {
            freq_max_file,
            freq_min_file,
            freq_list,
            freq_range_map,
        }
    }

    fn set_max_req_freq(&mut self, freq: u32) -> Result<(), Error> {
        self.freq_max_file.write_str(format!("{}", freq).as_str())?;
        Ok(())
    }

    fn set_min_req_freq(&mut self, freq: u32) -> Result<(), Error> {
        self.freq_min_file.write_str(format!("{}", freq).as_str())?;
        Ok(())
    }

    pub fn set_freq_range(&mut self, freq_range: (u32, u32)) -> Result<(), Error> {
        self.set_max_req_freq(freq_range.1)?;
        self.set_min_req_freq(freq_range.0)?;
        Ok(())
    }
}

impl CpuHandle {
    pub fn new(log_handle: Arc<Mutex<LogHandle>>) -> Self {
        let mut policy_map = HashMap::new();
        let mut policy_index = 0;
        let mut policy_range_map = HashMap::new();
        let mut policy_path_vec_clone = Vec::new();
        let result = read_dir("/sys/devices/system/cpu/cpufreq");

        match result {
            Ok(all_dir) => {
                let mut policy_path_vec = Vec::new();
                for dir in all_dir {
                    let path = dir.unwrap().path();
                    if path.is_dir() {
                        policy_path_vec.push(path.clone());
                        policy_path_vec_clone.push(path);
                    }
                }

                policy_path_insertion_sort(&mut policy_path_vec);
                policy_path_insertion_sort(&mut policy_path_vec_clone);

                let policy_line = policy_path_vec.len();
                for path in policy_path_vec {
                    let result = path.to_string_lossy()[path.to_string_lossy().len() - 1..]
                        .trim()
                        .parse::<u32>();
                    let now_policy_index = match result {
                        Ok(index) => index,
                        Err(e) => {
                            if let Ok(mut log_handle) = log_handle.lock() {
                                log_handle.error(format!("解析policy_index失败: {:?}", e));
                            }
                            panic!("解析policy_index失败");
                        }
                    };
                    if policy_index != policy_line - 1 {
                        let next_policy_index = policy_path_vec_clone[policy_index + 1]
                            .to_string_lossy()[path.to_string_lossy().len() - 1..]
                            .parse::<u32>()
                            .unwrap();
                        policy_range_map
                            .insert(policy_index as u32, (now_policy_index, next_policy_index));
                    }
                    if policy_index == policy_line - 1 {
                        policy_range_map.insert(policy_index as u32, (now_policy_index, 8));
                    }
                    let policy = Policy::new(log_handle.clone(), path);
                    policy_map.insert(policy_index, policy);
                    policy_index += 1;
                }

                Self {
                    cpu_name: "cpu".to_string(),
                    policy_index,
                    policy_range_map,
                    policy_map,
                    log_handle,
                }
            }

            Err(e) => {
                if let Ok(mut log_handle) = log_handle.lock() {
                    log_handle.error(format!("读取cpufreq目录失败: {:?}", e));
                }
                panic!("读取cpufreq目录失败");
            }
        }
    }

    pub fn get_policy(&self, policy_index: u32) -> &Policy {
        self.policy_map.get(&(policy_index as usize)).unwrap()
    }
}
