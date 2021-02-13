use log::error;
use std::error::Error;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

mod blocks;
use blocks::Block;
use blocks::{
    battery::BatteryBlock, clock::ClockBlock, cpufreq::CPUFreqBlock, loadavg::LoadAvgBlock,
    memory::MemoryBlock, network::NetworkBlock, swap::SwapBlock,
};

const SOCKET_PATH: &str = "/tmp/tmux-status-bar-rust.sock";

fn main() {
    env_logger::init();
    let socket_path = Path::new(SOCKET_PATH);

    let mut blocks = [
        &mut NetworkBlock::new(Duration::new(5, 0)) as &mut dyn Block,
        &mut LoadAvgBlock::new(Duration::new(10, 0)) as &mut dyn Block,
        &mut CPUFreqBlock::new(Duration::new(2, 0)) as &mut dyn Block,
        &mut MemoryBlock::new(Duration::new(5, 0)) as &mut dyn Block,
        &mut SwapBlock::new(Duration::new(5, 0)) as &mut dyn Block,
        &mut BatteryBlock::new(Duration::new(10, 0)) as &mut dyn Block,
        &mut ClockBlock::new(Duration::new(1, 0)) as &mut dyn Block,
    ];
    if socket_path.exists() {
        std::fs::remove_file(socket_path).expect("failed to remove file");
    }
    let listener = UnixListener::bind(socket_path).expect("failed to bind socket");
    let mut buf = [0u8; 1024];
    for stream in listener.incoming() {
        match stream {
            Err(e) => error!("{}", e),
            Ok(mut stream) => match handle_stream(&mut stream, &mut blocks, &mut buf) {
                Err(e) => error!("failed to handle stream: {}", e),
                Ok(()) => {}
            },
        }
    }
}

fn handle_stream(
    stream: &mut UnixStream,
    blocks: &mut [&mut dyn Block],
    buf: &mut [u8],
) -> Result<(), Box<dyn Error>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
    for i in 0..blocks.len() {
        let block = &mut blocks[i];
        block.update(now);
        match block.get_bytes().read(buf) {
            Err(_) => {
                stream.write(b" error")?;
            }
            Ok(nbytes) => {
                if nbytes == 0 {
                    continue;
                }
                stream.write(b" ")?;
                stream.write(&buf[0..nbytes])?;
            }
        }
    }
    stream.flush()?;
    return Ok(());
}
