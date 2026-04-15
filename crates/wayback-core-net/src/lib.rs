// crates/wayback-core-net/src/lib.rs

//! wayback-core-net
//!
//! Minimal core for the Internet-of-Games / Wayback networking layer.
//!
//! This first pass provides a simple TUN-based echo proxy:
//! - Creates and configures a TUN device in userspace.
//! - Spawns an async task that reads IPv4 packets from the TUN.
//! - Echoes packets back to the TUN device (round-trip inside the virtual link).
//!
//! The design is intentionally small and pluggable:
//! - `TunDevice` wraps OS-specific TUN configuration.
//! - `TunEchoProxy` is a reusable async runner that can later be
//!   replaced with protocol-aware forwarding, DNS override, NAT logic, etc.
//!
//! Platform notes:
//! - Linux/Unix: uses `tun`-style character device at `/dev/net/tun`.
//! - Windows: the module compiles but currently returns `Unsupported`.
//!
//! Future directions:
//! - Replace echo with routing into a game-aware proxy.
//! - Add DNS override and IPX emulation modules.
//! - Integrate with knowledge-graph-aware configuration and session profiles.

use std::io;
use std::sync::Arc;

use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::task::JoinHandle;

#[cfg(unix)]
mod tun_unix;
#[cfg(unix)]
pub use tun_unix::TunDevice;

#[cfg(windows)]
mod tun_windows;
#[cfg(windows)]
pub use tun_windows::TunDevice;

/// Size of the read/write buffer for TUN IO.
///
/// 16 KiB is plenty for typical MTU-sized packets and some batching.
const TUN_BUF_SIZE: usize = 16 * 1024;

/// High-level error type for wayback-core-net.
#[derive(Debug, Error)]
pub enum WaybackNetError {
    #[error("TUN device error: {0}")]
    Tun(#[from] TunError),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Proxy task join error: {0}")]
    Join(#[from] tokio::task::JoinError),

    #[error("Unsupported operation on this platform")]
    Unsupported,
}

/// OS-specific TUN setup and errors are encapsulated in TunError.
#[derive(Debug, Error)]
pub enum TunError {
    #[error("IO error while configuring TUN device: {0}")]
    Io(#[from] io::Error),

    #[error("Unsupported platform or configuration")]
    Unsupported,

    #[error("Invalid TUN configuration: {0}")]
    InvalidConfig(String),
}

/// Configuration for creating a TUN device.
///
/// This struct is intentionally minimal. Future additions:
/// - Explicit MTU
/// - IPv6 enable flags
/// - Platform-specific adapter options
#[derive(Debug, Clone)]
pub struct TunConfig {
    /// Optional requested interface name; OS may choose a different one.
    pub name: Option<String>,
    /// IPv4 address/cidr to assign (platform-specific helper will apply).
    pub ipv4_cidr: Option<String>,
}

impl Default for TunConfig {
    fn default() -> Self {
        Self {
            name: Some("wayback-tun0".to_string()),
            // Example: 10.13.37.1/24 for a private virtual segment.
            ipv4_cidr: Some("10.13.37.1/24".to_string()),
        }
    }
}

/// Handle for a running TUN echo proxy.
///
/// Dropping this handle does not automatically cancel the task; call
/// `shutdown` if you want a graceful stop.
pub struct TunEchoProxy {
    tun: Arc<TunDevice>,
    task: JoinHandle<Result<(), WaybackNetError>>,
}

impl TunEchoProxy {
    /// Spawns an async echo loop that:
    /// - Reads packets from the TUN device.
    /// - Writes the same bytes back to the TUN.
    ///
    /// This is a diagnostic and scaffolding tool, not a final router.
    pub fn spawn(tun: TunDevice) -> Self {
        let tun = Arc::new(tun);
        let tun_clone = tun.clone();

        let task = tokio::spawn(async move {
            let mut buf = vec![0u8; TUN_BUF_SIZE];

            loop {
                let n = tun_clone.read(&mut buf).await?;
                if n == 0 {
                    // TUN closed by OS or teardown.
                    break;
                }

                // Minimal sanity: ignore obviously tiny frames.
                if n < 20 {
                    continue;
                }

                // Echo packet back to TUN. In the future, this is where
                // routing and protocol handlers will be invoked.
                tun_clone.write_all(&buf[..n]).await?;
            }

            Ok(())
        });

        Self { tun, task }
    }

    /// Waits for the echo loop to terminate.
    ///
    /// In most cases this will run forever until the TUN is closed.
    pub async fn join(self) -> Result<(), WaybackNetError> {
        self.task.await??;
        Ok(())
    }

    /// Attempts a cooperative shutdown by closing the TUN device.
    ///
    /// Depending on platform details, this should cause the echo loop
    /// to observe EOF and terminate.
    pub async fn shutdown(self) -> Result<(), WaybackNetError> {
        self.tun.close().await?;
        self.join().await
    }

    /// Returns the name of the underlying TUN interface (if known).
    pub fn if_name(&self) -> &str {
        self.tun.if_name()
    }
}

/// Convenience function: create a TUN device with `TunConfig::default`
/// and start the echo proxy.
///
/// This is primarily for quick smoke tests and examples.
pub async fn start_default_tun_echo() -> Result<TunEchoProxy, WaybackNetError> {
    let cfg = TunConfig::default();
    let tun = TunDevice::create(cfg).await?;
    let proxy = TunEchoProxy::spawn(tun);
    Ok(proxy)
}

/// Example integration entry point.
///
/// A binary crate `wayback-core-net-echo` can call this from its `main`
/// to provide a ready-to-run echo utility.
pub async fn run_tun_echo_cli() -> Result<(), WaybackNetError> {
    let proxy = start_default_tun_echo().await?;
    println!("wayback-core-net: TUN echo running on interface {}", proxy.if_name());
    proxy.join().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn tun_config_default_has_ipv4() {
        let cfg = TunConfig::default();
        assert!(cfg.ipv4_cidr.is_some());
    }
}
