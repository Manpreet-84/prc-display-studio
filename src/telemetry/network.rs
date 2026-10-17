//! Network: rates from `NetworkData::received()/transmitted()` which are
//! byte counts *since the last refresh* — divide by elapsed time for MB/s.
//! Local IP comes from `ip_networks()` (no extra FFI). Public IP is never
//! collected here.

use std::time::Instant;

use sysinfo::Networks;

use super::NetworkStats;

const BYTES_TO_MB: f64 = 1_000_000.0; // decimal MB (network convention)

pub struct NetProbe {
    last_refresh: Option<Instant>,
}

impl Default for NetProbe {
    fn default() -> Self {
        Self::new()
    }
}

impl NetProbe {
    pub fn new() -> Self {
        Self { last_refresh: None }
    }
}

fn conn_type(name: &str) -> &'static str {
    let n = name.to_ascii_lowercase();
    if n.contains("wi-fi") || n.contains("wifi") || n.contains("wlan") {
        "WI-FI"
    } else if n.contains("ethernet") {
        "ETHERNET"
    } else if n.contains("vpn") || n.contains("tailscale") || n.contains("wireguard") {
        "VPN"
    } else if n.contains("loopback") || n.contains("bluetooth") {
        "LOCAL"
    } else {
        "NETWORK"
    }
}

pub fn collect(networks: &mut Networks, probe: &mut NetProbe) -> NetworkStats {
    networks.refresh(true);
    let now = Instant::now();
    let dt = probe
        .last_refresh
        .map(|t| now.duration_since(t).as_secs_f64())
        .unwrap_or(0.0);
    probe.last_refresh = Some(now);

    let mut down_bps = 0.0f64;
    let mut up_bps = 0.0f64;
    let mut active: Option<(&String, u64, Option<String>)> = None; // (name, traffic, ip)

    for (name, net) in networks.iter() {
        if name.to_ascii_lowercase().contains("loopback") {
            continue; // loopback traffic is not "network" for the display
        }
        let traffic = net.received() + net.transmitted();
        down_bps += net.received() as f64;
        up_bps += net.transmitted() as f64;
        {
            let ip = net
                .ip_networks()
                .iter()
                .find_map(|n| {
                    if n.addr.is_ipv4() && !n.addr.is_loopback() {
                        Some(n.addr.to_string())
                    } else {
                        None
                    }
                });
            let better = match &active {
                None => true,
                Some((_, t, _)) => traffic > *t,
            };
            if better && traffic > 0 {
                active = Some((name, traffic, ip));
            }
        }
    }

    // Fall back to any non-loopback interface with an IPv4 if traffic was 0.
    if active.is_none() {
        for (name, net) in networks.iter() {
            if name.to_ascii_lowercase().contains("loopback") {
                continue;
            }
            let ip = net.ip_networks().iter().find_map(|n| {
                if n.addr.is_ipv4() && !n.addr.is_loopback() {
                    Some(n.addr.to_string())
                } else {
                    None
                }
            });
            if ip.is_some() {
                active = Some((name, 0, ip));
                break;
            }
        }
    }

    let (down_mbps, up_mbps) = if dt > 0.05 {
        (
            Some((down_bps / dt * 8.0 / BYTES_TO_MB) as f32),
            Some((up_bps / dt * 8.0 / BYTES_TO_MB) as f32),
        )
    } else {
        (None, None) // first tick has no window yet
    };

    NetworkStats {
        down_mbps,
        up_mbps,
        local_ip: active.as_ref().and_then(|a| a.2.clone()),
        conn_type: active.as_ref().map(|a| conn_type(a.0).to_string()),
        adapter: active.map(|a| a.0.clone()),
    }
}
