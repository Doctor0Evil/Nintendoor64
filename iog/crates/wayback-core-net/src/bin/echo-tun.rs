use anyhow::Result;
use tokio::runtime::Builder;
use tracing_subscriber::EnvFilter;
use wayback_core_net::TunLink;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let rt = Builder::new_multi_thread().enable_all().build()?;
    rt.block_on(async {
        let tun = TunLink::open("iog0").await?;
        tun.run_echo().await
    })
}
