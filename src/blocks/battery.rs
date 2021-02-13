use super::Block;
use log::info;
use std::io::Write;
use std::time::Duration;

const BUF_LEN: usize = 11;
const BATTERY_CAPACITY_PATH: &str = "/sys/class/power_supply/BAT0/capacity";
const BATTERY_STATUS_PATH: &str = "/sys/class/power_supply/BAT0/status";

pub struct BatteryBlock {
    last_updated: Duration,
    interval: Duration,
    bytes: [u8; BUF_LEN],
}

impl BatteryBlock {
    pub fn new(interval: Duration) -> BatteryBlock {
        BatteryBlock {
            last_updated: Duration::new(0, 0),
            interval: interval,
            bytes: [0u8; BUF_LEN],
        }
    }
}

impl Block for BatteryBlock {
    fn update(&mut self, unixtime: std::time::Duration) {
        if unixtime - self.last_updated < self.interval {
            return;
        }
        self.last_updated = unixtime;

        let capacity_raw = match std::fs::read_to_string(BATTERY_CAPACITY_PATH) {
            Err(_) => "err".to_owned(),
            Ok(s) => s,
        };
        let status_raw = match std::fs::read_to_string(BATTERY_STATUS_PATH) {
            Err(_) => 'E',
            Ok(s) => s.chars().next().unwrap_or('E'),
        };
        write!(
            &mut self.bytes[..],
            "Bat:{: >3}%({})",
            capacity_raw.trim(),
            status_raw
        )
        .expect("failed to write");
        info!("battery updated");
    }
    fn get_bytes(&self) -> &[u8] {
        &self.bytes[0..BUF_LEN]
    }
}
