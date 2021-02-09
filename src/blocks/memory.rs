use super::Block;
use log::info;
use std::io::Read;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use std::time::Duration;

const BUF_LEN: usize = 10;
const MEMINFO_PATH: &str = "/proc/meminfo";

pub struct MemoryBlock {
    last_updated: Duration,
    interval: Duration,
    bytes: [u8; BUF_LEN],
    len: usize,
}

impl MemoryBlock {
    pub fn new(interval: Duration) -> MemoryBlock {
        MemoryBlock {
            last_updated: Duration::new(0, 0),
            interval: interval,
            bytes: [0u8; BUF_LEN],
            len: 0,
        }
    }
}

impl Block for MemoryBlock {
    fn update(&mut self, unixtime: std::time::Duration) {
        if unixtime - self.last_updated < self.interval {
            return;
        }
        self.last_updated = unixtime;
        let mut mem_total = -1;
        let mut mem_avail = -1;
        if let Ok(file) = std::fs::File::open(MEMINFO_PATH) {
            for line in BufReader::new(file).lines() {
                let line = match line {
                    Err(_) => continue,
                    Ok(l) => l,
                };
                let field = match line.split(':').next() {
                    Some("MemTotal") => &mut mem_total,
                    Some("MemAvailable") => &mut mem_avail,
                    _ => continue,
                };
                if let Some(rawstr) = line.rsplit(' ').nth(1) {
                    if let Ok(val) = i64::from_str(rawstr) {
                        *field = val;
                    }
                }
            }
        };

        let s = if mem_total != -1 && mem_avail != -1 {
            let ratio = (mem_total - mem_avail) * 100 / mem_total;
            format!("Mem:{}%", ratio)
        } else {
            "".to_string()
        };
        self.len = s.as_bytes().read(&mut self.bytes).unwrap_or(0);
        info!("memory updated");
    }
    fn get_bytes(&self) -> &[u8] {
        &self.bytes[0..self.len]
    }
}
