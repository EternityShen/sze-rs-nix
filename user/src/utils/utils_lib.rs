use core::panic;
use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{BufReader, Read},
    path::PathBuf,
    process::Command,
};

pub fn divide_u32_vector_into_intervals_map(
    vec: Vec<u32>,
    portion: u32,
) -> HashMap<usize, (u32, u32)> {
    let mut intervals_map = HashMap::new();
    let vec_len = vec.len() as u32;

    // 处理特殊情况
    if vec_len == 0 {
        panic!("向量为空");
    }

    if portion == 0 {
        panic!(" portion 不能为0");
    }

    // 计算基础区间大小：总长度除以部分数
    let interval_size = vec_len / portion;
    // 计算余数：总长度除以部分数的余数
    let remainder = vec_len % portion;

    // 遍历每个部分，为每个部分创建一个区间
    for i in 0..portion {
        // 计算当前区间的起始索引
        // 基础起始位置是 i * interval_size
        // 如果当前索引小于余数，则加上 i（因为前 remainder 个区间每个都多一个元素）
        // 否则，加上余数（因为后面的区间不再有多出的元素）
        let start = i * interval_size + if i < remainder { i } else { remainder };

        // 计算当前区间的结束索引
        // 起始位置加上基础区间大小
        // 如果当前索引小于余数，则再加上1（因为前 remainder 个区间每个都多一个元素）
        // 最后减1得到结束索引（因为索引从0开始）
        let end = start + interval_size + if i < remainder { 1 } else { 0 } - 1;

        // 将计算出的区间（起始值和结束值）添加到结果映射中
        intervals_map.insert(i as usize, (vec[start as usize], vec[end as usize]));
    }

    intervals_map
}

pub fn insertion_sort(arr: &mut Vec<u32>) {
    for i in 1..arr.len() {
        let key = arr[i];
        let mut j = i;

        // 将比key大的元素向右移动
        while j > 0 && arr[j - 1] > key {
            arr[j] = arr[j - 1];
            j -= 1;
        }

        arr[j] = key;
    }
}

fn get_policy_index_from_path(path: &PathBuf) -> u32 {
    let path_str = path.to_string_lossy().to_string();
    // 提取路径的最后一个组件
    let last_component = path_str.split('/').last().unwrap_or("");
    // 移除 "policy" 前缀，只保留数字部分
    let index_str = last_component.trim_start_matches("policy");
    // 解析数字部分为 u32
    index_str.parse::<u32>().unwrap()
}

pub fn policy_path_insertion_sort(vec: &mut Vec<PathBuf>) {
    for i in 1..vec.len() {
        let key = get_policy_index_from_path(&vec[i]);
        let current = vec[i].clone();
        let mut j = i;
        while j > 0 && get_policy_index_from_path(&vec[j - 1]) > key {
            vec[j] = vec[j - 1].clone();
            j -= 1;
        }
        vec[j] = current;
    }
}

pub fn inotify_init(path: &PathBuf) -> inotify::Inotify {
    let inotify = inotify::Inotify::init().unwrap();
    inotify
        .watches()
        .add(path, inotify::WatchMask::MODIFY)
        .unwrap();
    inotify
}

pub fn inotify_blockage(inotify: &mut inotify::Inotify) {
    let mut buffer = [0u8; 4096];
    loop {
        match inotify.read_events(&mut buffer) {
            Ok(events) => {
                #[allow(clippy::never_loop)]
                for _ in events {
                    break;
                }
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(e) => {
                panic!("读取 inotify 事件失败: {:?}", e);
            }
        }
    }
}

pub fn get_top_app() -> String {
    let result = Command::new("sh")
        .arg("dumpsys window | grep mCurrentFocus")
        .output()
        .expect("执行命令失败");

    let output = String::from_utf8_lossy(&result.stdout);
    output.trim().to_string()
}

pub fn get_game_list() -> Vec<String> {
    let result = OpenOptions::new()
        .read(true)
        .open("/data/adb/modules/ETERNAL_CPU_SZE/config/game_list.txt")
        .expect("打开文件失败");
    let mut reader = BufReader::new(result);
    let mut contents = String::new();
    reader.read_to_string(&mut contents).expect("读取文件失败");
    contents.lines().map(|line| line.to_string()).collect()
}
#[test]
fn test_inotify_init() {
    let path = PathBuf::from("/home/sheneternity/Work/sze-rs-nix/debug/test.txt");
    let mut inotify = inotify_init(&path);
    loop {
        inotify_blockage(&mut inotify);
        println!("文件被修改");
    }
}

#[test]
fn test_divide_u32_vector_into_intervals_map() {
    let mut vec = vec![1, 3, 2, 4, 8, 6, 7, 5, 11, 10, 9];
    insertion_sort(&mut vec);
    let portion = 4;
    let intervals_map = divide_u32_vector_into_intervals_map(vec, portion);

    for i in 0..portion {
        let (start, end) = intervals_map.get(&(i as usize)).unwrap();
        println!("{}-{}", start, end);
    }
}

#[test]
fn test_policy_path_insertion_sort() {
    let mut vec = vec![
        PathBuf::from("/sys/devices/system/cpu/cpufreq/policy1"),
        PathBuf::from("/sys/devices/system/cpu/cpufreq/policy5"),
        PathBuf::from("/sys/devices/system/cpu/cpufreq/policy9"),
        PathBuf::from("/sys/devices/system/cpu/cpufreq/policy8"),
    ];
    policy_path_insertion_sort(&mut vec);

    for path in vec {
        println!("{}", path.display());
    }
}
