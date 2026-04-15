// crates/wayback-core-net/src/tun_unix.rs

use std::ffi::CString;
use std::fs::File;
use std::io::{self, Read, Write};
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::os::unix::io::IntoRawFd;
use std::path::Path;

use nix::fcntl::{fcntl, FcntlArg, OFlag};
use nix::sys::socket;
use nix::sys::socket::{SockAddr, SockFlag, SockType};
use nix::sys::socket::{AF_INET, AF_UNIX};
use nix::unistd;
use tokio::io::unix::AsyncFd;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{TunConfig, TunError};

/// Simple wrapper around a Unix TUN device.
///
/// This uses /dev/net/tun and an ioctl-based configuration. It is intentionally
/// conservative and only supports a basic point-to-point IPv4 setup in the
/// first pass.
pub struct TunDevice {
    if_name: String,
    fd: AsyncFd<File>,
}

impl TunDevice {
    /// Create and configure a new TUN device using the supplied config.
    pub async fn create(cfg: TunConfig) -> Result<Self, TunError> {
        let dev_path = Path::new("/dev/net/tun");
        let file = File::options()
            .read(true)
            .write(true)
            .open(dev_path)
            .map_err(TunError::from)?;

        // SAFETY: we immediately wrap into AsyncFd and keep ownership.
        let raw_fd = file.into_raw_fd();

        // Configure TUN interface via ioctl(TUNSETIFF).
        let if_name = configure_tun(raw_fd, cfg.name.clone())?;

        let file = unsafe { File::from_raw_fd(raw_fd) };
        let async_fd = AsyncFd::new(file).map_err(TunError::from)?;

        // Apply IPv4 address if requested.
        if let Some(cidr) = cfg.ipv4_cidr {
            configure_ipv4(&if_name, &cidr)?;
        }

        Ok(Self {
            if_name,
            fd: async_fd,
        })
    }

    /// Asynchronously read from the TUN device into the provided buffer.
    pub async fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let mut guard = self.fd.readable().await?;
            match guard.get_inner().read(buf) {
                Ok(n) => return Ok(n),
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    guard.clear_ready();
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Asynchronously write bytes into the TUN device.
    pub async fn write_all(&self, buf: &[u8]) -> io::Result<()> {
        loop {
            let mut guard = self.fd.writable().await?;
            match guard.get_inner().write(buf) {
                Ok(_) => return Ok(()),
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    guard.clear_ready();
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Close the underlying file descriptor.
    pub async fn close(&self) -> io::Result<()> {
        // Dropping the File will close the FD; AsyncFd does this on Drop.
        // Here we just flush and let the owner drop the handle.
        Ok(())
    }

    /// Return the interface name assigned to this TUN.
    pub fn if_name(&self) -> &str {
        &self.if_name
    }
}

/// Configure /dev/net/tun via TUNSETIFF.
///
/// This uses the common `struct ifreq` pattern and requests a TUN device
/// without packet information (IFF_TUN | IFF_NO_PI).
fn configure_tun(fd: RawFd, requested: Option<String>) -> Result<String, TunError> {
    use libc::{c_char, c_short, c_ushort, ifreq, IFF_NO_PI, IFF_TUN};

    let mut ifr: ifreq = unsafe { std::mem::zeroed() };

    let name = requested.unwrap_or_else(|| "wayback-tun0".to_string());
    let c_name = CString::new(name.clone()).map_err(|e| {
        TunError::InvalidConfig(format!("invalid interface name: {e}"))
    })?;

    // Copy name into ifr.ifr_name
    let bytes = c_name.as_bytes_with_nul();
    if bytes.len() > ifr.ifr_name.len() {
        return Err(TunError::InvalidConfig(
            "requested interface name too long".to_string(),
        ));
    }
    for (i, b) in bytes.iter().enumerate() {
        ifr.ifr_name[i] = *b as c_char;
    }

    // Set flags
    unsafe {
        let flags: c_short = (IFF_TUN | IFF_NO_PI) as c_short;
        let ifr_flags = &mut ifr.ifr_ifru.ifru_flags as *mut c_short;
        *ifr_flags = flags;
    }

    // Issue ioctl(TUNSETIFF)
    let res = unsafe { libc::ioctl(fd, libc::TUNSETIFF, &ifr) };
    if res < 0 {
        return Err(TunError::Io(io::Error::last_os_error()));
    }

    // Extract the actual interface name the kernel chose.
    let mut out = String::new();
    for c in ifr.ifr_name.iter() {
        let b = *c as u8;
        if b == 0 {
            break;
        }
        out.push(b as char);
    }

    Ok(out)
}

/// Apply a basic IPv4 address and bring the interface up.
///
/// This implementation shells out to `ip` as a simple first pass.
/// In later iterations, this should be replaced with netlink calls
/// for robustness and better error handling.
fn configure_ipv4(if_name: &str, cidr: &str) -> Result<(), TunError> {
    use std::process::Command;

    let status = Command::new("ip")
        .args(["addr", "add", cidr, "dev", if_name])
        .status()
        .map_err(TunError::from)?;

    if !status.success() {
        return Err(TunError::InvalidConfig(format!(
            "failed to assign IPv4 address {cidr} to {if_name}"
        )));
    }

    let status = Command::new("ip")
        .args(["link", "set", "dev", if_name, "up"])
        .status()
        .map_err(TunError::from)?;

    if !status.success() {
        return Err(TunError::InvalidConfig(format!(
            "failed to bring interface {if_name} up"
        )));
    }

    Ok(())
}
