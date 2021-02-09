pub mod clock;
pub mod battery;
pub mod swap;
pub mod memory;
pub mod cpufreq;
pub mod loadavg;
pub mod network;

pub trait Block {
    fn update(&mut self, unixtime: std::time::Duration);
    fn get_bytes(&self) -> &[u8];
}
