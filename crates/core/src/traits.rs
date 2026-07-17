use async_trait::async_trait;
use crate::error::CoreResult;
use crate::events::{Event, EventEnvelope, SystemMetrics};
use crate::id::{ConnectionId, JobId, ServerId};
use crate::types::{ConnectionInfo, LogEntry, Server};

/// Event bus para comunicación desacoplada entre módulos.
#[async_trait]
pub trait EventBus: Send + Sync {
    async fn publish(&self, event: Event, source: &str);
    async fn subscribe(&self) -> tokio::sync::broadcast::Receiver<EventEnvelope>;
}

/// Proveedor de conexiones (SSH, RDP, etc.).
#[async_trait]
pub trait ConnectionProvider: Send + Sync {
    async fn connect(&self, info: &ConnectionInfo) -> CoreResult<ConnectionId>;
    async fn disconnect(&self, id: ConnectionId) -> CoreResult<()>;
    async fn execute(&self, id: ConnectionId, command: &str) -> CoreResult<CommandResult>;
    async fn is_connected(&self, id: ConnectionId) -> bool;
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// Colector de métricas del sistema.
#[async_trait]
pub trait MetricsCollector: Send + Sync {
    async fn collect_local(&self) -> CoreResult<SystemMetrics>;
    async fn collect_remote(&self, server_id: ServerId) -> CoreResult<SystemMetrics>;
}

/// Repositorio de servidores.
#[async_trait]
pub trait ServerRepository: Send + Sync {
    async fn list(&self) -> CoreResult<Vec<Server>>;
    async fn get(&self, id: ServerId) -> CoreResult<Option<Server>>;
    async fn save(&self, server: &Server) -> CoreResult<()>;
    async fn delete(&self, id: ServerId) -> CoreResult<()>;
}

/// Servicio de logging centralizado.
#[async_trait]
pub trait LogService: Send + Sync {
    async fn log(&self, entry: LogEntry);
    async fn query(
        &self,
        server_id: Option<ServerId>,
        limit: usize,
    ) -> CoreResult<Vec<LogEntry>>;
}

/// Motor de plugins.
#[async_trait]
pub trait PluginEngine: Send + Sync {
    async fn load(&self, path: &str) -> CoreResult<()>;
    async fn unload(&self, id: &str) -> CoreResult<()>;
    async fn list(&self) -> CoreResult<Vec<crate::types::PluginManifest>>;
    async fn call(&self, plugin_id: &str, action: &str, args: serde_json::Value) -> CoreResult<serde_json::Value>;
}

/// Programador de tareas automatizadas.
#[async_trait]
pub trait TaskScheduler: Send + Sync {
    async fn schedule(&self, job: Job) -> CoreResult<JobId>;
    async fn cancel(&self, id: JobId) -> CoreResult<()>;
    async fn list(&self) -> CoreResult<Vec<JobDefinition>>;
    async fn run_now(&self, job: &JobDefinition) -> CoreResult<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub definition: JobDefinition,
    pub state: JobState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobDefinition {
    pub id: Option<JobId>,
    pub name: String,
    pub description: String,
    pub cron_expr: String,
    pub action: JobAction,
    pub enabled: bool,
    pub server_id: Option<ServerId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobAction {
    RunCommand(String),
    BackupServer(ServerId),
    UpdatePackages,
    RestartService(String),
    CustomScript(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobState {
    Scheduled,
    Running,
    Completed,
    Failed(String),
    Cancelled,
}

use serde::{Deserialize, Serialize};
