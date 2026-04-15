use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{info, warn};

/// Simple wrapper around a userspace TUN device.
pub struct TunLink {
    dev: tun::AsyncDevice,
}

impl TunLink {
    pub async fn open(name: &str) -> Result<Self> {
        let config = tun::Configuration::default()
            .name(name)
            .layer(tun::Layer::L3)
            .up();

        let dev = tun::create_as_async(&config)?;
        Ok(Self { dev })
    }

    /// Minimal echo loop: read packets and write them straight back.
    /// Useful as a diagnostic scaffold before adding routing logic.
    pub async fn run_echo(mut self) -> Result<()> {
        let mut buf = vec![0u8; 65535];

        loop {
            let n = self.dev.read(&mut buf).await?;
            if n == 0 {
                continue;
            }
            let pkt = &buf[..n];
            info!("TUN echo: {} bytes", n);
            if let Err(e) = self.dev.write_all(pkt).await {
                warn!("Failed to echo packet: {e:#}");
            }
        }
    }
}
