use super::Block;
use log::{error, info};
use std::io::Read;
use std::time::Duration;

const BUF_LEN: usize = 20;
const CPUFREQ_PATH: &str = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq";

pub struct CPUFreqBlock {
    last_updated: Duration,
    interval: Duration,
    bytes: [u8; BUF_LEN],
    len: usize,
}

impl CPUFreqBlock {
    pub fn new(interval: Duration) -> CPUFreqBlock {
        CPUFreqBlock {
            last_updated: Duration::new(0, 0),
            interval: interval,
            bytes: [0u8; BUF_LEN],
            len: 0,
        }
    }
}

impl Block for CPUFreqBlock {
    fn update(&mut self, unixtime: std::time::Duration) {
        if unixtime - self.last_updated < self.interval {
            return;
        }
        self.last_updated = unixtime;

        let s = match std::fs::read_to_string(CPUFREQ_PATH) {
            Ok(cpufreq_raw) => match cpufreq_raw.trim().parse::<f64>() {
                Ok(freq) => {
                    let freq = freq / 1_000_000.0;
                    format!("Freq:{:.1}GHz", freq)
                }
                Err(e) => {
                    error!("failed to parse string {}: {}", cpufreq_raw, e);
                    "".to_string()
                }
            },
            Err(e) => {
                error!("failed to read cpufreq: {}", e);
                "".to_string()
            }
        };

        self.len = s.as_bytes().read(&mut self.bytes).unwrap_or(0);
        info!("cpufreq updated");
    }
    fn get_bytes(&self) -> &[u8] {
        &self.bytes[0..self.len]
    }
}
