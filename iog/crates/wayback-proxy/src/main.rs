use anyhow::Result;
use iog_protocol_model::{PacketDirection, PacketEnvelope, TransportKind};
use std::net::{IpAddr, Ipv4Addr};
use tokio::task;
use tracing_subscriber::EnvFilter;
use wayback_core_net::TunLink;
use wayback_dns::{LegacyDomainRule, SelectiveResolver};
use wayback_dht::{GameKey, IogNode};
use wayback_scripting::LuaSandbox;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // 1) TUN echo loop (Phase 1 diagnostic scaffold)
    let tun_task = task::spawn(async {
        let tun = TunLink::open("iog0").await?;
        tun.run_echo().await
    });

    // 2) DNS override for GameSpy master
    let dns_task = task::spawn(async {
        let rules = vec![LegacyDomainRule {
            domain_suffix: "master.gamespy.com".to_string(),
            override_ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        }];
        let resolver = SelectiveResolver::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), rules).await?;
        let ip = resolver.lookup_ipv4("master.gamespy.com").await?;
        tracing::info!("Resolved master.gamespy.com to {ip}");
        Ok::<_, anyhow::Error>(())
    });

    // 3) DHT node advertising one game
    let dht_task = task::spawn(async {
        let listen = "/ip4/0.0.0.0/tcp/30007".parse().unwrap();
        let mut node = IogNode::new(listen)?;
        node.advertise_game(GameKey {
            gamename: "Command & Conquer: Red Alert 2".to_string(),
        });
        node.run().await
    });

    // 4) Lua handler demo
    let lua_task = task::spawn(async {
        let lua = LuaSandbox::new()?;
        lua.load_script(
            r#"
            function on_packet(pkt)
              -- passthrough example: never drop, no rewrite
              return { drop = false, rewritten_payload_b64 = nil }
            end
            "#,
        )?;

        let pkt = PacketEnvelope {
            conn_id: "demo".into(),
            direction: PacketDirection::ClientToServer,
            transport: TransportKind::Udp,
            payload_b64: base64::encode(b"hello"),
            protocol_hint: Some("gamespy".into()),
        };

        let res = lua.handle_packet(&pkt)?;
        tracing::info!("Lua handler result: {:?}", res);
        Ok::<_, anyhow::Error>(())
    });

    let _ = tokio::try_join!(tun_task, dns_task, dht_task, lua_task)?;
    Ok(())
}
