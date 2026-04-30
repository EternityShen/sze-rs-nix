use crate::se_config_parser::parser::{SeConfig, SeConfigVec};

#[derive(Clone, Debug)]
pub struct LoadLevelConfig {
    pub min: u32,
    pub mid: u32,
    pub high: u32,
    pub e_high: u32,
}

#[derive(Clone, Debug)]
pub struct SchedulerConfig {
    pub polling_interval: u32,
    pub load_level_config: LoadLevelConfig,
}

impl LoadLevelConfig {
    pub fn new(mode: &str) -> LoadLevelConfig {
        let se_config_vec = SeConfigVec::new("/data/adb/modules/ETERNAL_CPU_SZE/config/config.se");
        let se_config = SeConfig::new(&se_config_vec, format!("{}/LoadLevel", mode).as_str());
        let min = se_config.get_u32("Min");
        let mid = se_config.get_u32("Mid");
        let high = se_config.get_u32("High");
        let e_high = se_config.get_u32("EHigh");
        LoadLevelConfig {
            min,
            mid,
            high,
            e_high,
        }
    }
}

impl SchedulerConfig {
    pub fn new(mode: &str) -> SchedulerConfig {
        let se_config_vec = SeConfigVec::new("/data/adb/modules/ETERNAL_CPU_SZE/config/config.se");
        let se_config = SeConfig::new(&se_config_vec, mode);
        let polling_interval = se_config.get_u32("PollingInterval");
        let load_level_config = LoadLevelConfig::new(mode);
        SchedulerConfig {
            polling_interval,
            load_level_config,
        }
    }
}

#[test]
fn test_load_level_config() {
    let scheduler_config = SchedulerConfig::new("Power");
    println!("{:?}", scheduler_config);
}