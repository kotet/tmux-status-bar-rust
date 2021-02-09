use super::Block;
use log::{error, info};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::time::Duration;

const BUF_LEN: usize = 128;
const NET_PATH: &str = "/sys/class/net";

pub struct NetworkBlock {
    last_updated: Duration,
    interval: Duration,
    bytes: [u8; BUF_LEN],
    len: usize,
    map: HashMap<String, NetworkBytes>,
}

#[derive(Debug, Copy, Clone)]
struct NetworkBytes {
    rx: u64,
    tx: u64,
}

#[derive(Debug)]
struct NetworkEntry {
    name: String,
    bytes: NetworkBytes,
}

fn get_networks() -> Vec<NetworkEntry> {
    let mut ret: Vec<NetworkEntry> = Vec::new();
    let dirs = fs::read_dir(NET_PATH);
    if let Ok(dirs) = dirs {
        for dir in dirs {
            if let Ok(dir) = dir {
                let path = dir.path();
                let name = path.file_name().map(|p| p.to_str()).unwrap_or(None);
                if let Some(name) = name {
                    let rx_path = path.join("statistics/rx_bytes");
                    let tx_path = path.join("statistics/tx_bytes");
                    let rx = fs::read_to_string(rx_path)
                        .map(|s| s.trim().parse::<u64>().ok())
                        .unwrap_or(None);
                    let tx = fs::read_to_string(tx_path)
                        .map(|s| s.trim().parse::<u64>().ok())
                        .unwrap_or(None);
                    if let (Some(rx), Some(tx)) = (rx, tx) {
                        ret.push(NetworkEntry {
                            name: name.to_string(),
                            bytes: NetworkBytes { rx: rx, tx: tx },
                        });
                    }
                }
            }
        }
    }
    ret
}

impl NetworkBlock {
    pub fn new(interval: Duration) -> NetworkBlock {
        NetworkBlock {
            last_updated: Duration::new(0, 0),
            interval: interval,
            bytes: [0u8; BUF_LEN],
            len: 0,
            map: HashMap::new(),
        }
    }
}

impl Block for NetworkBlock {
    fn update(&mut self, unixtime: std::time::Duration) {
        let deltatime = unixtime - self.last_updated;
        if deltatime < self.interval {
            return;
        }
        self.last_updated = unixtime;
        let mut s: String = "".to_owned();
        let networks = get_networks();

        for network in networks {
            match self.map.insert(network.name.clone(), network.bytes) {
                Some(old) => {
                    if network.bytes.rx < old.rx || network.bytes.tx < old.tx {
                        error!(
                            "new < old: rx {} -> {}, tx {} -> {}",
                            old.rx, network.bytes.rx, old.tx, network.bytes.tx
                        );
                        continue;
                    }
                    let rxdiff = (network.bytes.rx - old.rx) / deltatime.as_secs();
                    let txdiff = (network.bytes.tx - old.tx) / deltatime.as_secs();
                    if rxdiff != 0 || txdiff != 0 {
                        if s.len() != 0 {
                            s = s + " ";
                        }
                        let (rxdiff, rxprefix) = format_si_prefix(rxdiff);
                        let (txdiff, txprefix) = format_si_prefix(txdiff);
                        let f = format!(
                            "[{}: U{: >4}{}B/s D{: >4}{}B/s]",
                            &network.name, rxdiff, rxprefix, txdiff, txprefix,
                        );
                        s = s + &f;
                    }
                }
                None => {}
            }
        }

        self.len = s.as_bytes().read(&mut self.bytes).unwrap_or(0);
        info!("network updated");
    }
    fn get_bytes(&self) -> &[u8] {
        &self.bytes[0..self.len]
    }
}

const PREFIXES: [&str; 5] = ["", "Ki", "Mi", "Gi", "Ti"];

fn format_si_prefix(x: u64) -> (u64, &'static str) {
    let mut y = x;
    for i in 0..PREFIXES.len() - 1 {
        if y < 1024 {
            return (y, PREFIXES[i]);
        } else {
            y = y / 1024;
        }
    }
    return (y, PREFIXES[PREFIXES.len() - 1]);
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn format_si_prefix_test() {
        assert!(format_si_prefix(0) == (0, ""));
        assert!(format_si_prefix(1023) == (1023, ""));
        assert!(format_si_prefix(1024) == (1, "Ki"));
        assert!(format_si_prefix(1024 * 1024) == (1, "Mi"));
        assert!(format_si_prefix(1024 * 1024 * 1024) == (1, "Gi"));
        assert!(format_si_prefix(1024 * 1024 * 1024 * 1024) == (1, "Ti"));
        assert!(format_si_prefix(1024 * 1024 * 1024 * 1024 * 1024) == (1024, "Ti"));
    }
}
