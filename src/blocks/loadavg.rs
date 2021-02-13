use super::Block;
use log::{error, info};
use std::io::Read;
use std::time::Duration;

const BUF_LEN: usize = 10;
const LOADAVG_PATH: &str = "/proc/loadavg";

pub struct LoadAvgBlock {
    last_updated: Duration,
    interval: Duration,
    bytes: [u8; BUF_LEN],
    len: usize,
}

impl LoadAvgBlock {
    pub fn new(interval: Duration) -> LoadAvgBlock {
        LoadAvgBlock {
            last_updated: Duration::new(0, 0),
            interval: interval,
            bytes: [0u8; BUF_LEN],
            len: 0,
        }
    }
}

impl Block for LoadAvgBlock {
    fn update(&mut self, unixtime: std::time::Duration) {
        if unixtime - self.last_updated < self.interval {
            return;
        }
        self.last_updated = unixtime;

        let s = match std::fs::read_to_string(LOADAVG_PATH) {
            Err(e) => {
                error!("failed to read file {}: {}", LOADAVG_PATH, e);
                "".to_owned()
            }
            Ok(rawstr) => match rawstr.split(' ').next() {
                None => {
                    error!("failed to extract loadavg: {}", rawstr);
                    "".to_owned()
                }
                Some(col) => match col.parse::<f64>() {
                    Err(e) => {
                        error!("failed to parse {}: {}", col, e);
                        "".to_owned()
                    }
                    Ok(val) => format!("LA:{:.1}", val),
                },
            },
        };

        self.len = s.as_bytes().read(&mut self.bytes).unwrap_or(0);
        info!("loadavg updated");
    }
    fn get_bytes(&self) -> &[u8] {
        &self.bytes[0..self.len]
    }
}
