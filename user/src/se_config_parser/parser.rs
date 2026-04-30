pub struct SeConfigVec{
    pub config_vec: Vec<String>,
}

impl SeConfigVec {
    pub fn new(filename: &str) -> Self {
        if filename[filename.len()-3..].trim() != ".se" {
            panic!("配置文件后缀不是.se")
        }
        let mut config = Vec::new();
        let contents = std::fs::read_to_string(filename).expect("无法读取文件");
        for line in contents.lines() {
            config.push(line.to_string());
        }
        Self { config_vec: config }
    }

    pub fn contains_path(&self, path: &str) -> bool {
        for line in &self.config_vec {
            if line.contains(format!("[{}]", path).as_str()) {
                return true;
            }
        }
        false
    }
}

pub struct SeConfig {
    pub config_vec: Vec<String>,
}

impl SeConfig {
    pub fn new(se_config_vec: &SeConfigVec, path: &str) -> Self {
        let path_vec = path.split("/").collect::<Vec<&str>>();
        let mut config_vec = se_config_vec.config_vec.clone();
        let mut config = Vec::new();
        let mut start = 0;
        let mut started = false;
        let mut ended = false;
        for path in path_vec {
            config_vec = get_quian(config_vec, path);
        }
        for line in config_vec {
            if line.contains("[") && !started {
                started = true;
                continue;
            }
            if started && !ended {
                if line.contains("{") {
                    let a = line.find("{");
                    if let Some(a) = a {
                        start = a;
                        ended = true;
                        continue;
                    }
                }
            }
            if ended {
                if start + 1 <= line.len() && line[start..start+1].trim() == "}" {
                    ended = false;
                    started = false;
                    continue;
                }
                continue;
            }
            config.push(line.trim().to_string());
        }
        Self { config_vec: config }
    }

    pub fn get_string(&self, key: &str) -> String {
        let value = self.get_string_value(key);
        if value.starts_with("\"") && value.ends_with("\"") {
            value[1..value.len()-1].to_string()
        }else {
            panic!("{} 不是一个字符串", value);
        }
    }

    pub fn get_bool(&self, key: &str) -> bool {
        let value = self.get_string_value(key);
        if value == "true" {
           true
        }else if value == "false" {
            return false;
        }else {
            panic!("{} 不是一个布尔值", value);
        }
    }

    pub fn get_i32(&self, key: &str) -> i32 {
        let value = self.get_string_value(key);
        value.parse::<i32>().expect(format!("{} 不是一个i32整数", value).as_str())
    }

    pub fn get_i64(&self, key: &str) -> i64 {
        let value = self.get_string_value(key);
        value.parse::<i64>().expect(format!("{} 不是一个i64整数", value).as_str())
    }

    pub fn get_u32(&self, key: &str) -> u32 {
        let value = self.get_string_value(key);
        value.parse::<u32>().expect(format!("{} 不是一个无符号i32整数", value).as_str())
    }

    pub fn get_u64(&self, key: &str) -> u64 {
        let value = self.get_string_value(key);
        value.parse::<u64>().expect(format!("{} 不是一个无符号i64整数", value).as_str())
    }

    pub fn contains_key(&self, key: &str) -> bool {
        for line in &self.config_vec {
            if line.contains(format!("{} =", key).as_str()) {
                return true;
            }
        }
        false
    }

    fn get_string_value(&self, key: &str) -> String {
        for line in &self.config_vec {
            if line.contains(format!("{} =", key).as_str()) {
                let a = line.find("=");
                if let Some(a) = a {
                    let value = line[a+1..].trim().to_string();
                    return value;
                }
            }
        }
        panic!("{} 没有这个键", key);
    }

}


fn get_quian(config: Vec<String>, path: &str) -> Vec<String> {
    let mut quian = Vec::new();
    let mut started = false;
    let mut found = false;
    let mut start = 0;
    for line in config {
        if line.contains(format!("[{}]", path).as_str()) && !started {
            started = true;
            continue;
        }
        if started {
            if !found {
                let a = line.find("{");
                if let Some(a) = a {
                    start = a;
                    found = true;
                    continue;
                }
            }
            if start + 1 <= line.len() && line[start..start+1].trim() == "}" && found {
                break;
            } else {
                quian.push(line);
            }
        }
    }
    quian
}