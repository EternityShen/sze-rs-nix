use crate::{cpu_info::cpu_handle::Policy, listener::load_handle::LoadLevel};

pub struct PolicyScheduler<'a> {
    policy: &'a mut Policy,
}

impl<'a> PolicyScheduler<'a> {
    pub fn new(policy: &'a mut Policy) -> Self {
        Self { policy }
    }

    pub fn get_freq_range(&self, _policy_index: u32, loadlevel: &LoadLevel) -> (u32, u32) {
        let policy = &self.policy;
        match loadlevel {
            LoadLevel::Low => *policy.freq_range_map.get(&0).unwrap(),
            LoadLevel::Mid => *policy.freq_range_map.get(&1).unwrap(),
            LoadLevel::High => *policy.freq_range_map.get(&2).unwrap(),
            LoadLevel::Max => *policy.freq_range_map.get(&3).unwrap(),
        }
    }

    pub fn set_freq(&mut self, _policy_index: u32, freq_range: (u32, u32)) {
        let policy = &mut self.policy;
        let _ = policy.set_freq_range(freq_range);
    }
}

pub fn policy_freq_scheduler(
    policy_index: u32,
    load_level: &LoadLevel,
    policy_scheduler: &mut PolicyScheduler,
) {
    let freq_range = policy_scheduler.get_freq_range(policy_index, load_level);
    policy_scheduler.set_freq(policy_index, freq_range);
}

