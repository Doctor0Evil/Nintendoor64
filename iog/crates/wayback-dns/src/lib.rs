use anyhow::Result;
use hickory_resolver::{
    config::{NameServerConfig, NameServerConfigGroup, ResolverConfig, ResolverOpts},
    TokioAsyncResolver,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LegacyDomainRule {
    pub domain_suffix: String,
    pub override_ip: IpAddr,
}

#[derive(Debug, Clone)]
pub struct SelectiveResolver {
    inner: TokioAsyncResolver,
    rules: Vec<LegacyDomainRule>,
}

impl SelectiveResolver {
    pub async fn new(upstream: IpAddr, rules: Vec<LegacyDomainRule>) -> Result<Self> {
        let mut cfg = ResolverConfig::new();
        let ns = NameServerConfig::udp(upstream, 53);
        cfg.add_name_server(NameServerConfigGroup::from(ns));

        let resolver = TokioAsyncResolver::tokio(cfg, ResolverOpts::default())?;
        Ok(Self {
            inner: resolver,
            rules,
        })
    }

    pub async fn lookup_ipv4(&self, name: &str) -> Result<Ipv4Addr> {
        if let Some(rule) = self
            .rules
            .iter()
            .find(|r| name.ends_with(&r.domain_suffix))
        {
            if let IpAddr::V4(v4) = rule.override_ip {
                info!("DNS override: {name} -> {v4}");
                return Ok(v4);
            }
        }

        let resp = self.inner.ipv4_lookup(name).await?;
        let addr = resp.iter().next().unwrap_or(Ipv4Addr::LOCALHOST);
        Ok(addr)
    }
}
