use crate::id::ServerId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Server ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub id: ServerId,
    pub name: String,
    pub group_id: Option<String>,
    pub connection: ConnectionInfo,
    pub status: ServerStatus,
    pub profile: Option<ServerProfile>,
    pub tags: Vec<String>,
    pub notes: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_connected_at: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ServerStatus {
    #[default]
    Offline,
    Online,
    Connecting,
    Error,
    Restarting,
    Maintenance,
}

impl ServerStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Offline => "Offline",
            Self::Online => "Online",
            Self::Connecting => "Conectando",
            Self::Error => "Error",
            Self::Restarting => "Reiniciando",
            Self::Maintenance => "Mantenimiento",
        }
    }
}

// ─── Connection ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub protocol: ConnectionProtocol,
    pub host: String,
    pub port: u16,
    pub credential: Credential,
    pub options: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConnectionProtocol {
    Ssh,
    Sftp,
    Ftp,
    Ftps,
    Rdp,
    Vnc,
    Http,
    Https,
    WebSocket,
    Serial,
    Telnet,
    Api,
}

impl ConnectionProtocol {
    pub fn default_port(&self) -> u16 {
        match self {
            Self::Ssh | Self::Sftp => 22,
            Self::Ftp => 21,
            Self::Ftps => 990,
            Self::Rdp => 3389,
            Self::Vnc => 5900,
            Self::Http => 80,
            Self::Https => 443,
            Self::WebSocket => 80,
            Self::Serial => 0,
            Self::Telnet => 23,
            Self::Api => 443,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub auth_method: AuthMethod,
    pub username: Option<String>,
    pub password: Option<String>,
    pub private_key_path: Option<String>,
    pub private_key_data: Option<String>,
    pub passphrase: Option<String>,
    pub certificate_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthMethod {
    None,
    Password,
    Key,
    KeyWithPassphrase,
    Agent,
    Certificate,
    Token,
}

// ─── SSH Keys ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKeyPair {
    pub name: String,
    pub key_type: SshKeyType,
    pub public_key: String,
    pub private_key_path: String,
    pub bits: u32,
    pub comment: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SshKeyType {
    Rsa,
    Ed25519,
    Ecdsa,
}

// ─── OS / Profiles ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperatingSystem {
    Linux(OsFlavor),
    Windows,
    MacOS,
    FreeBSD,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OsFlavor {
    Ubuntu,
    Debian,
    Arch,
    Fedora,
    CentOS,
    AlmaLinux,
    RockyLinux,
    OpenSUSE,
    Raspbian,
    OpenMediaVault,
    TrueNAS,
    Proxmox,
    Alpine,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerProfile {
    pub profile_type: ProfileType,
    pub commands: Vec<String>,
    pub services: Vec<String>,
    pub ports: Vec<u16>,
    pub monitor_endpoints: Vec<String>,
    pub init_script: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProfileType {
    Generic,
    WebServer,
    Database,
    MinecraftJava,
    MinecraftBedrock,
    DockerHost,
    NodeJs,
    Php,
    Python,
    Java,
    Vpn,
    Nas,
    Custom(String),
}

// ─── Server Groups ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerGroup {
    pub id: String,
    pub name: String,
    pub color: String,
    pub server_ids: Vec<ServerId>,
    pub collapsed: bool,
}

// ─── Resource Limits ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_cpu_percent: Option<f64>,
    pub max_memory_mb: Option<u64>,
    pub max_disk_mb: Option<u64>,
    pub max_network_mbps: Option<u64>,
}

// ─── Service ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceState {
    Running,
    Stopped,
    Failed,
    Unknown,
}

// ─── Log ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
    pub server_id: Option<ServerId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Success,
    Warn,
    Error,
    Command,
    Output,
}

// ─── Plugin Manifest ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub plugin_type: PluginType,
    pub dependencies: Vec<String>,
    pub permissions: Vec<String>,
    pub entry_point: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginType {
    Page,
    Widget,
    Driver,
    Protocol,
    Monitor,
    Automation,
    Theme,
}

impl Default for AuthMethod {
    fn default() -> Self {
        Self::Password
    }
}
