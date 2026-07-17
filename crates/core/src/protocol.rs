use serde::{Deserialize, Serialize};

/// Protocolo de comunicación entre módulos del manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ManagerCommand {
    Ping,
    Connect { server_id: String },
    Disconnect { connection_id: String },
    Execute { connection_id: String, command: String },
    CollectMetrics { server_id: Option<String> },
    ScanNetwork { subnet: String },
    StartService { name: String },
    StopService { name: String },
    RestartService { name: String },
    GetFile { path: String, remote: bool, connection_id: Option<String> },
    PutFile { local_path: String, remote_path: String, connection_id: String },
    ScheduleJob { job: serde_json::Value },
    CancelJob { job_id: String },
    LoadPlugin { path: String },
    UnloadPlugin { plugin_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ManagerResponse {
    Ok,
    Error { code: u32, message: String },
    Pong,
    Connected { connection_id: String },
    Output { stdout: String, stderr: String, exit_code: i32 },
    Metrics(crate::events::SystemMetrics),
    HostList(Vec<HostInfo>),
    ServiceStatus { name: String, running: bool },
    FileContent { path: String, content: Vec<u8>, is_binary: bool },
    JobScheduled { job_id: String },
    PluginLoaded { plugin_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostInfo {
    pub ip: String,
    pub hostname: Option<String>,
    pub os: Option<String>,
    pub ports: Vec<u16>,
    pub mac: Option<String>,
    pub vendor: Option<String>,
}
