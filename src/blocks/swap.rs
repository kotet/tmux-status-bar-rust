use super::Block;
use log::info;
use std::io::Read;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use std::time::Duration;

const BUF_LEN: usize = 10;
const MEMINFO_PATH: &str = "/proc/meminfo";

pub struct SwapBlock {
    last_updated: Duration,
    interval: Duration,
    bytes: [u8; BUF_LEN],
    len: usize,
}

impl SwapBlock {
    pub fn new(interval: Duration) -> SwapBlock {
        SwapBlock {
            last_updated: Duration::new(0, 0),
            interval: interval,
            bytes: [0u8; BUF_LEN],
            len: 0,
        }
    }
}

impl Block for SwapBlock {
    fn update(&mut self, unixtime: std::time::Duration) {
        if unixtime - self.last_updated < self.interval {
            return;
        }
        self.last_updated = unixtime;
        let mut swap_total = -1;
        let mut swap_free = -1;
        if let Ok(file) = std::fs::File::open(MEMINFO_PATH) {
            for line in BufReader::new(file).lines() {
                let line = match line {
                    Err(_) => continue,
                    Ok(l) => l,
                };
                let field = match line.split(':').next() {
                    Some("SwapTotal") => &mut swap_total,
                    Some("SwapFree") => &mut swap_free,
                    _ => continue,
                };
                if let Some(rawstr) = line.rsplit(' ').nth(1) {
                    if let Ok(val) = i64::from_str(rawstr) {
                        *field = val;
                    }
                }
            }
        };

        let s = if swap_total != -1 && swap_free != -1 {
            let ratio = (swap_total - swap_free) * 100 / swap_total;
            format!("Swp:{}%", ratio)
        } else {
            "".to_string()
        };
        self.len = s.as_bytes().read(&mut self.bytes).unwrap_or(0);
        info!("swap updated");
    }
    fn get_bytes(&self) -> &[u8] {
        &self.bytes[0..self.len]
    }
}
