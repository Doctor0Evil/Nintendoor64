// crates/wayback-core-net/src/tun_windows.rs

use std::io;

use crate::{TunConfig, TunError};

/// Placeholder Windows implementation.
///
/// This compiles but always reports Unsupported. A future pass can
/// integrate with Wintun or the Windows TAP driver to create an
/// equivalent virtual adapter.
pub struct TunDevice {
    if_name: String,
}

impl TunDevice {
    pub async fn create(_cfg: TunConfig) -> Result<Self, TunError> {
        Err(TunError::Unsupported)
    }

    pub async fn read(&self, _buf: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "TUN not implemented on Windows",
        ))
    }

    pub async fn write_all(&self, _buf: &[u8]) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "TUN not implemented on Windows",
        ))
    }

    pub async fn close(&self) -> io::Result<()> {
        Ok(())
    }

    pub fn if_name(&self) -> &str {
        &self.if_name
    }
}
