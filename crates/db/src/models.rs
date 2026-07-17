use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmServer {
    pub id: String,
    pub name: String,
    pub group_id: Option<String>,
    pub host: String,
    pub port: u16,
    pub protocol: String,
    pub username: Option<String>,
    pub auth_method: String,
    pub credential_data: Option<String>,
    pub status: String,
    pub os_type: Option<String>,
    pub os_flavor: Option<String>,
    pub profile_type: Option<String>,
    pub tags: Option<String>,
    pub notes: String,
    pub metadata: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub last_connected_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmLog {
    pub id: Option<i64>,
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub source: String,
    pub server_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmMetric {
    pub id: Option<i64>,
    pub server_id: String,
    pub timestamp: String,
    pub cpu_percent: f64,
    pub memory_used_bytes: i64,
    pub memory_total_bytes: i64,
    pub disk_used_bytes: i64,
    pub disk_total_bytes: i64,
    pub network_rx_bytes: i64,
    pub network_tx_bytes: i64,
    pub swap_used_bytes: i64,
    pub swap_total_bytes: i64,
    pub load_1m: f64,
    pub load_5m: f64,
    pub load_15m: f64,
    pub uptime_seconds: i64,
    pub process_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmJob {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cron_expr: String,
    pub action_type: String,
    pub action_data: Option<String>,
    pub server_id: Option<String>,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmSshKey {
    pub id: String,
    pub name: String,
    pub key_type: String,
    pub public_key: String,
    pub private_key_path: String,
    pub bits: u32,
    pub comment: String,
    pub fingerprint: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmAuditEntry {
    pub id: Option<i64>,
    pub timestamp: String,
    pub actor: String,
    pub action: String,
    pub resource: String,
    pub details: Option<String>,
    pub ip_address: Option<String>,
}
