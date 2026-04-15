// crates/wayback-core-net-echo/src/main.rs

//! wayback-core-net-echo
//!
//! Small diagnostic binary that starts the TUN echo loop from
//! `wayback-core-net` and logs the interface name.
//!
//! Usage (from the workspace root):
//!   cargo run -p wayback-core-net-echo
//!
//! On Linux, this will:
//!   - Create /dev/net/tun-backed interface (e.g. wayback-tun0).
//!   - Assign a default IPv4 address (10.13.37.1/24).
//!   - Echo any packets sent to that interface back to the sender.
//!
//! You can test basic connectivity with (as root or via sudo):
//!   ip addr show wayback-tun0
//!   ping -I wayback-tun0 10.13.37.1

use wayback_core_net::run_tun_echo_cli;

#[tokio::main]
async fn main() {
    if let Err(err) = run_tun_echo_cli().await {
        eprintln!("wayback-core-net-echo: error: {err}");
        std::process::exit(1);
    }
}
