use super::Block;
use chrono::{DateTime, Local, TimeZone};
use log::info;
use std::io::Read;
use std::time::Duration;

const BUF_LEN: usize = 20;

pub struct ClockBlock {
    last_updated: Duration,
    interval: Duration,
    bytes: [u8; BUF_LEN],
    len: usize,
}

impl ClockBlock {
    pub fn new(interval: Duration) -> ClockBlock {
        ClockBlock {
            last_updated: Duration::new(0, 0),
            interval: interval,
            bytes: [0u8; BUF_LEN],
            len: 0,
        }
    }
}

impl Block for ClockBlock {
    fn update(&mut self, unixtime: std::time::Duration) {
        if unixtime - self.last_updated < self.interval {
            return;
        }
        self.last_updated = unixtime;
        let date: DateTime<Local> = Local.timestamp(unixtime.as_secs() as i64, 0);
        let s: String = date.format("%m/%d(%a)%H:%M:%S").to_string();
        self.len = s.as_bytes().read(&mut self.bytes).unwrap_or(0);
        info!("clock updated");
    }
    fn get_bytes(&self) -> &[u8] {
        &self.bytes[0..self.len]
    }
}
