mod manager;
mod watcher;

pub use manager::ConfigManager;
pub use watcher::ConfigWatcher;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub general: GeneralConfig,

    #[serde(default)]
    pub ui: UiConfig,

    #[serde(default)]
    pub servers: ServersConfig,

    #[serde(default)]
    pub security: SecurityConfig,

    #[serde(default)]
    pub monitor: MonitorConfig,

    #[serde(default)]
    pub plugins: PluginsConfig,

    #[serde(default)]
    pub automation: AutomationConfig,

    #[serde(default)]
    pub network: NetworkConfig,

    #[serde(default)]
    pub integrations: HashMap<String, IntegrationConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_data_dir")]
    pub data_dir: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub check_updates: bool,
    #[serde(default)]
    pub send_telemetry: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub font_size: f32,
    #[serde(default)]
    pub animations: bool,
    #[serde(default)]
    pub compact_mode: bool,
    #[serde(default)]
    pub show_status_bar: bool,
    #[serde(default)]
    pub sidebar_width: f32,
    #[serde(default)]
    pub terminal_font: String,
    #[serde(default)]
    pub terminal_font_size: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServersConfig {
    #[serde(default = "default_ssh_port")]
    pub default_ssh_port: u16,
    #[serde(default)]
    pub connection_timeout_secs: u64,
    #[serde(default)]
    pub keepalive_interval_secs: u64,
    #[serde(default)]
    pub max_connections: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    #[serde(default)]
    pub encrypt_credentials: bool,
    #[serde(default)]
    pub audit_enabled: bool,
    #[serde(default)]
    pub session_timeout_minutes: u32,
    #[serde(default)]
    pub max_login_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_monitor_interval")]
    pub interval_ms: u64,
    #[serde(default)]
    pub retain_days: u32,
    #[serde(default)]
    pub alert_cpu_threshold: f64,
    #[serde(default)]
    pub alert_memory_threshold: f64,
    #[serde(default)]
    pub alert_disk_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginsConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub directory: String,
    #[serde(default)]
    pub auto_load: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub max_concurrent_jobs: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    #[serde(default)]
    pub scan_timeout_ms: u64,
    #[serde(default)]
    pub max_parallel_scans: usize,
    #[serde(default)]
    pub default_subnets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    pub enabled: bool,
    pub path: Option<String>,
    pub args: Option<Vec<String>>,
}

fn default_data_dir() -> String { String::new() }
fn default_language() -> String { "en".into() }
fn default_theme() -> String { "dark".into() }
fn default_ssh_port() -> u16 { 22 }
fn default_monitor_interval() -> u64 { 5000 }

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            ui: UiConfig::default(),
            servers: ServersConfig::default(),
            security: SecurityConfig::default(),
            monitor: MonitorConfig::default(),
            plugins: PluginsConfig::default(),
            automation: AutomationConfig::default(),
            network: NetworkConfig::default(),
            integrations: HashMap::new(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            data_dir: String::new(),
            language: "en".into(),
            check_updates: true,
            send_telemetry: false,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            font_size: 14.0,
            animations: true,
            compact_mode: false,
            show_status_bar: true,
            sidebar_width: 260.0,
            terminal_font: "monospace".into(),
            terminal_font_size: 13.0,
        }
    }
}

impl Default for ServersConfig {
    fn default() -> Self {
        Self {
            default_ssh_port: 22,
            connection_timeout_secs: 30,
            keepalive_interval_secs: 60,
            max_connections: 10,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            encrypt_credentials: true,
            audit_enabled: true,
            session_timeout_minutes: 30,
            max_login_attempts: 5,
        }
    }
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_ms: 5000,
            retain_days: 30,
            alert_cpu_threshold: 90.0,
            alert_memory_threshold: 90.0,
            alert_disk_threshold: 90.0,
        }
    }
}

impl Default for PluginsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            directory: String::new(),
            auto_load: vec![],
        }
    }
}

impl Default for AutomationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_concurrent_jobs: 5,
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            scan_timeout_ms: 5000,
            max_parallel_scans: 20,
            default_subnets: vec!["192.168.1.0/24".into()],
        }
    }
}
