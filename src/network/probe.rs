use crate::core::errors::AppError;
use std::{
    net::{Ipv4Addr, SocketAddr, TcpStream},
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

pub fn web_ui_available(target: Ipv4Addr) -> bool {
    TcpStream::connect_timeout(&SocketAddr::from((target, 80)), Duration::from_millis(1500)).is_ok()
}

pub fn probe_once(source: Ipv4Addr, target: Ipv4Addr) -> Result<bool, AppError> {
    #[cfg(windows)]
    let status = Command::new("ping")
        .args([
            "-4",
            "-S",
            &source.to_string(),
            "-n",
            "1",
            "-w",
            "1000",
            &target.to_string(),
        ])
        .status()
        .map_err(|e| AppError::Platform(e.to_string()))?;
    #[cfg(not(windows))]
    let status = Command::new("ping")
        .args(["-c", "1", "-W", "1", &target.to_string()])
        .status()
        .map_err(|e| AppError::Platform(e.to_string()))?;
    Ok(status.success())
}

pub fn start(
    source: Ipv4Addr,
    target: Ipv4Addr,
    cancel: Arc<AtomicBool>,
    on_result: impl Fn(bool) + Send + 'static,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while !cancel.load(Ordering::Relaxed) {
            let result = probe_once(source, target).unwrap_or(false);
            on_result(result);
            if result {
                break;
            }
            thread::sleep(Duration::from_secs(1));
        }
    })
}
