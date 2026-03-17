use serde::Deserialize;
use std::collections::HashMap;
use tokio::process::Command;

const TS: &str = "/usr/bin/tailscale";

// ── Status types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Status {
    pub version: String,
    pub backend_state: String,
    #[serde(rename = "Self")]
    pub self_node: Option<PeerStatus>,
    pub peer: Option<HashMap<String, PeerStatus>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PeerStatus {
    #[serde(rename = "ID")]
    pub id: String,
    pub host_name: String,
    #[serde(rename = "DNSName")]
    pub dns_name: String,
    #[serde(rename = "TailscaleIPs")]
    pub tailscale_ips: Option<Vec<String>>,
    pub online: bool,
    #[serde(rename = "OS")]
    pub os: String,
    pub exit_node: Option<bool>,
    pub exit_node_option: Option<bool>,
}

impl PeerStatus {
    pub fn primary_ip(&self) -> &str {
        self.tailscale_ips
            .as_deref()
            .and_then(|ips| ips.first())
            .map(|s| s.as_str())
            .unwrap_or("—")
    }

    pub fn short_name(&self) -> &str {
        self.dns_name
            .split('.')
            .next()
            .filter(|s| !s.is_empty())
            .unwrap_or(&self.host_name)
    }
}

// ── Exit node types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ExitNode {
    pub name: String,
    pub ip: String,
    pub active: bool,
}

// ── Netcheck ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct NetcheckReport {
    #[serde(rename = "UDP")]
    pub udp: bool,
    #[serde(rename = "IPv4")]
    pub ipv4: bool,
    #[serde(rename = "IPv6")]
    pub ipv6: bool,
    #[serde(rename = "MappingVariesByDestIP")]
    pub mapping_varies_by_dest_ip: Option<bool>,
    #[serde(rename = "PreferredDERP")]
    pub preferred_derp: i32,
    // values are nanoseconds
    #[serde(rename = "RegionLatency")]
    pub region_latency: Option<HashMap<String, u64>>,
}

// ── Prefs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct Prefs {
    #[serde(rename = "CorpDNS")]
    pub accept_dns: bool,
    #[serde(rename = "RouteAll")]
    pub accept_routes: bool,
    #[serde(rename = "ShieldsUp")]
    pub shields_up: bool,
    #[serde(rename = "RunSSH")]
    pub run_ssh: bool,
    #[serde(rename = "ExitNodeAllowLANAccess")]
    pub exit_node_allow_lan_access: bool,
    #[serde(rename = "AdvertiseRoutes")]
    pub advertise_routes: Option<Vec<String>>,
    #[serde(rename = "Hostname")]
    pub hostname: String,
    #[serde(rename = "AdvertiseExitNode", default)]
    pub advertise_exit_node: bool,
}

// ── CLI helpers ───────────────────────────────────────────────────────────────

async fn run(args: &[&str]) -> Result<String, String> {
    let out = Command::new(TS)
        .args(args)
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).to_string())
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

pub async fn status() -> Result<Status, String> {
    let raw = run(&["status", "--json"]).await?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

pub async fn connect() -> Result<(), String> {
    run(&["up"]).await.map(|_| ())
}

pub async fn disconnect() -> Result<(), String> {
    run(&["down"]).await.map(|_| ())
}

pub async fn ping(ip: &str) -> Result<String, String> {
    run(&["ping", "--c", "3", ip]).await
}

pub async fn netcheck() -> Result<NetcheckReport, String> {
    let raw = run(&["netcheck", "--format=json"]).await?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

pub async fn exit_nodes() -> Result<Vec<ExitNode>, String> {
    let raw = run(&["exit-node", "list"]).await?;
    let nodes = raw
        .lines()
        .skip(1) // header
        .filter(|l| !l.trim().is_empty())
        .map(|line| {
            let cols: Vec<&str> = line.split_whitespace().collect();
            ExitNode {
                ip: cols.first().copied().unwrap_or("").to_string(),
                name: cols.get(1).copied().unwrap_or("").to_string(),
                active: line.contains('*'),
            }
        })
        .collect();
    Ok(nodes)
}

pub async fn set_exit_node(name: &str) -> Result<(), String> {
    run(&["set", "--exit-node", name]).await.map(|_| ())
}

pub async fn clear_exit_node() -> Result<(), String> {
    run(&["set", "--exit-node="]).await.map(|_| ())
}

pub async fn prefs() -> Result<Prefs, String> {
    let raw = run(&["debug", "prefs"]).await?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

pub async fn set_bool(flag: &str, value: bool) -> Result<(), String> {
    let arg = format!("--{}={}", flag, value);
    run(&["set", &arg]).await.map(|_| ())
}
