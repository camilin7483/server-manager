use crate::id::{ConnectionId, JobId, PluginId, ServerId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    // Server
    ServerAdded(ServerId),
    ServerRemoved(ServerId),
    ServerUpdated(ServerId),
    ServerStatusChanged(ServerId, String),

    // Connection
    ConnectionOpened(ConnectionId),
    ConnectionClosed(ConnectionId),
    ConnectionError(ConnectionId, String),

    // SSH
    SshSessionEstablished(ServerId),
    SshSessionClosed(ServerId),
    CommandExecuted(ServerId, String),
    CommandOutput { server_id: ServerId, output: String, exit_code: i32 },

    // Monitoring
    MetricsCollected(ServerId, SystemMetrics),
    AlertTriggered { server_id: ServerId, alert: Alert },

    // Plugin
    PluginLoaded(PluginId),
    PluginUnloaded(PluginId),
    PluginError(PluginId, String),

    // Automation
    JobStarted(JobId),
    JobCompleted(JobId),
    JobFailed(JobId, String),

    // System
    ConfigChanged,
    DatabaseMigrated(u32),
    AppShutdown,
    AppStarted { version: String, timestamp: DateTime<Utc> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub server_id: ServerId,
    pub timestamp: DateTime<Utc>,
    pub cpu_percent: f64,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub disk_used_bytes: u64,
    pub disk_total_bytes: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub swap_used_bytes: u64,
    pub swap_total_bytes: u64,
    pub load_average: [f64; 3],
    pub uptime_seconds: u64,
    pub process_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub alert_type: AlertType,
    pub message: String,
    pub severity: AlertSeverity,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    HighCpu,
    HighMemory,
    LowDisk,
    ServiceDown,
    ConnectionLost,
    SslExpiring,
    AuthenticationFailure,
    Custom(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event: Event,
    pub timestamp: DateTime<Utc>,
    pub source: String,
}

impl EventEnvelope {
    pub fn new(event: Event, source: impl Into<String>) -> Self {
        Self {
            event,
            timestamp: Utc::now(),
            source: source.into(),
        }
    }
}
