use sm_automation::TaskScheduler;
use sm_core::id::ServerId;
use sm_core::types::ConnectionInfo;
use sm_docker::DockerManager;
use sm_minecraft::MinecraftManager;
use sm_monitor::SystemMonitor;
use sm_net::shell;
use sm_net::{SessionManager, SshClient, SshOutput};
use sm_reports::ReportEngine;
use sm_security::CredentialVault;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use tracing::info;

/// Contexto central de la aplicación.
/// Conecta todos los módulos backend con la UI.
/// Todas las operaciones pesadas se ejecutan en un runtime de Tokio.
pub struct AppContext {
    runtime: Runtime,
    sessions: Arc<SessionManager>,
    ssh_clients: Arc<Mutex<HashMap<ServerId, Arc<SshClient>>>>,
    monitor: Arc<Mutex<SystemMonitor>>,
    vault: Arc<Mutex<Option<CredentialVault>>>,
    scheduler: Arc<Mutex<TaskScheduler>>,
    reports: Arc<ReportEngine>,
    // Minecraft servers keyed by server name
    minecraft_managers: Arc<Mutex<HashMap<String, Arc<MinecraftManager>>>>,
    // Docker managers keyed by server id
    docker_managers: Arc<Mutex<HashMap<ServerId, Arc<DockerManager>>>>,
    // Pending async results
    pending_results: Arc<Mutex<Vec<AsyncResult>>>,
}

#[derive(Debug, Clone)]
pub enum AsyncResult {
    Connected(ServerId),
    Disconnected(ServerId),
    Error(ServerId, String),
    CommandOutput(ServerId, String),
    MinecraftLog(String, String),
    MinecraftStatus(String, sm_minecraft::MinecraftStatus),
    DockerContainers(ServerId, Vec<sm_docker::DockerContainer>),
    BackupCreated(String, String),
    PluginInstalled(String, String),
    ReportGenerated(String, Vec<u8>),
}

impl AppContext {
    pub fn new() -> Self {
        let runtime = Runtime::new().expect("failed to create tokio runtime");
        Self {
            runtime,
            sessions: Arc::new(SessionManager::new(15)),
            ssh_clients: Arc::new(Mutex::new(HashMap::new())),
            monitor: Arc::new(Mutex::new(SystemMonitor::new())),
            vault: Arc::new(Mutex::new(None)),
            scheduler: Arc::new(Mutex::new(TaskScheduler::new())),
            reports: Arc::new(ReportEngine::new()),
            minecraft_managers: Arc::new(Mutex::new(HashMap::new())),
            docker_managers: Arc::new(Mutex::new(HashMap::new())),
            pending_results: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Initialize credential vault with master password
    pub fn init_vault(&self, master_password: &str) {
        let vault = CredentialVault::new(master_password);
        *self.vault.blocking_lock() = Some(vault);
        info!("Vault de credenciales inicializado");
    }

    // ─── SSH ──────────────────────────────────────────────────────

    pub fn connect(&self, server_id: ServerId, info: ConnectionInfo) {
        let sessions = self.sessions.clone();
        let ssh_clients = self.ssh_clients.clone();
        let pending = self.pending_results.clone();
        let sid = server_id;

        self.runtime.spawn(async move {
            let client = Arc::new(SshClient::new());
            let username = info.credential.username.as_deref().unwrap_or("root");
            match client
                .connect(&info.host, info.port, username, &info.credential)
                .await
            {
                Ok(()) => {
                    ssh_clients.lock().await.insert(sid, client.clone());
                    sessions
                        .connect(sid, &info)
                        .await
                        .ok();
                    pending.lock().await.push(AsyncResult::Connected(sid));
                }
                Err(e) => {
                    pending.lock().await.push(AsyncResult::Error(sid, e));
                }
            }
        });
    }

    pub fn disconnect(&self, server_id: ServerId) {
        let sessions = self.sessions.clone();
        let ssh_clients = self.ssh_clients.clone();
        let pending = self.pending_results.clone();
        let sid = server_id;

        self.runtime.spawn(async move {
            sessions.disconnect(sid).await;
            ssh_clients.lock().await.remove(&sid);
            pending.lock().await.push(AsyncResult::Disconnected(sid));
        });
    }

    pub fn execute(&self, server_id: ServerId, command: String) {
        let sessions = self.sessions.clone();
        let pending = self.pending_results.clone();
        let sid = server_id;

        self.runtime.spawn(async move {
            match sessions.execute(sid, &command).await {
                Ok(output) => {
                    pending
                        .lock()
                        .await
                        .push(AsyncResult::CommandOutput(sid, output.stdout));
                }
                Err(e) => {
                    pending.lock().await.push(AsyncResult::Error(sid, e));
                }
            }
        });
    }

    pub fn active_connections(&self) -> Vec<ServerId> {
        self.runtime.block_on(self.sessions.active_sessions())
    }

    pub fn is_connected(&self, server_id: ServerId) -> bool {
        self.runtime.block_on(self.sessions.is_connected(server_id))
    }

    // ─── Monitor ──────────────────────────────────────────────────

    pub fn collect_local_metrics(&self) -> sm_core::events::SystemMetrics {
        let monitor = self.monitor.clone();
        self.runtime
            .block_on(async move { monitor.lock().await.collect(sm_core::id::ServerId::new()) })
    }

    // ─── Minecraft ────────────────────────────────────────────────

    pub fn create_minecraft_manager(&self, server_name: &str, ssh: Arc<SshClient>, dir: &str) {
        let mgr = Arc::new(MinecraftManager::new(ssh, dir));
        let mc = self.minecraft_managers.clone();
        let name = server_name.to_string();
        self.runtime.spawn(async move {
            mc.lock().await.insert(name, mgr);
        });
    }

    pub fn minecraft_create_server(
        &self,
        server_name: &str,
        config: sm_minecraft::MinecraftServer,
    ) {
        let mc = self.minecraft_managers.clone();
        let pending = self.pending_results.clone();
        let name = server_name.to_string();
        let cfg = config;

        self.runtime.spawn(async move {
            let managers = mc.lock().await;
            if let Some(mgr) = managers.get(&name) {
                match mgr.create(&cfg).await {
                    Ok(()) => {
                        pending
                            .lock()
                            .await
                            .push(AsyncResult::PluginInstalled(
                                name.clone(),
                                format!("Servidor {} creado", name),
                            ));
                    }
                    Err(e) => {
                        pending
                            .lock()
                            .await
                            .push(AsyncResult::Error(ServerId::nil(), e));
                    }
                }
            } else {
                pending
                    .lock()
                    .await
                    .push(AsyncResult::Error(
                        ServerId::nil(),
                        format!("Manager de Minecraft '{}' no encontrado", name),
                    ));
            }
        });
    }

    pub fn minecraft_send_command(&self, server_name: &str, command: &str) {
        let mc = self.minecraft_managers.clone();
        let name = server_name.to_string();
        let cmd = command.to_string();
        let pending = self.pending_results.clone();

        self.runtime.spawn(async move {
            let managers = mc.lock().await;
            if let Some(mgr) = managers.get(&name) {
                if let Err(e) = mgr.send_command(&cmd).await {
                    pending
                        .lock()
                        .await
                        .push(AsyncResult::Error(ServerId::nil(), e));
                }
            }
        });
    }

    pub fn minecraft_get_logs(&self, server_name: &str, lines: u32) {
        let mc = self.minecraft_managers.clone();
        let name = server_name.to_string();
        let pending = self.pending_results.clone();

        self.runtime.spawn(async move {
            let managers = mc.lock().await;
            if let Some(mgr) = managers.get(&name) {
                match mgr.get_recent_logs(lines).await {
                    Ok(logs) => {
                        pending
                            .lock()
                            .await
                            .push(AsyncResult::MinecraftLog(name, logs));
                    }
                    Err(e) => {
                        pending
                            .lock()
                            .await
                            .push(AsyncResult::Error(ServerId::nil(), e));
                    }
                }
            }
        });
    }

    pub fn minecraft_status(&self, server_name: &str) {
        let mc = self.minecraft_managers.clone();
        let name = server_name.to_string();
        let pending = self.pending_results.clone();

        self.runtime.spawn(async move {
            let managers = mc.lock().await;
            if let Some(mgr) = managers.get(&name) {
                match mgr.status().await {
                    Ok(status) => {
                        pending
                            .lock()
                            .await
                            .push(AsyncResult::MinecraftStatus(name, status));
                    }
                    Err(e) => {
                        pending
                            .lock()
                            .await
                            .push(AsyncResult::Error(ServerId::nil(), e));
                    }
                }
            }
        });
    }

    pub fn minecraft_backup(&self, server_name: &str, backup_name: &str, backup_dir: &str) {
        let mc = self.minecraft_managers.clone();
        let name = server_name.to_string();
        let bname = backup_name.to_string();
        let bdir = backup_dir.to_string();
        let pending = self.pending_results.clone();

        self.runtime.spawn(async move {
            let managers = mc.lock().await;
            if let Some(mgr) = managers.get(&name) {
                match mgr.create_backup(&bname, &bdir).await {
                    Ok(info) => {
                        pending
                            .lock()
                            .await
                            .push(AsyncResult::BackupCreated(name, info.name));
                    }
                    Err(e) => {
                        pending
                            .lock()
                            .await
                            .push(AsyncResult::Error(ServerId::nil(), e));
                    }
                }
            }
        });
    }

    pub fn minecraft_install_plugin(&self, server_name: &str, slug: &str) {
        let mc = self.minecraft_managers.clone();
        let name = server_name.to_string();
        let plugin_slug = slug.to_string();
        let pending = self.pending_results.clone();

        self.runtime.spawn(async move {
            let managers = mc.lock().await;
            if let Some(mgr) = managers.get(&name) {
                match mgr.install_plugin(&plugin_slug).await {
                    Ok(()) => {
                        pending
                            .lock()
                            .await
                            .push(AsyncResult::PluginInstalled(
                                name,
                                format!("Plugin {} instalado", plugin_slug),
                            ));
                    }
                    Err(e) => {
                        pending
                            .lock()
                            .await
                            .push(AsyncResult::Error(ServerId::nil(), e));
                    }
                }
            }
        });
    }

    // ─── Docker ───────────────────────────────────────────────────

    pub fn docker_list_containers(&self, server_id: ServerId) {
        let docker_mgrs = self.docker_managers.clone();
        let pending = self.pending_results.clone();
        let sid = server_id;

        self.runtime.spawn(async move {
            let managers = docker_mgrs.lock().await;
            if let Some(mgr) = managers.get(&sid) {
                match mgr.list_containers(true).await {
                    Ok(containers) => {
                        pending
                            .lock()
                            .await
                            .push(AsyncResult::DockerContainers(sid, containers));
                    }
                    Err(e) => {
                        pending
                            .lock()
                            .await
                            .push(AsyncResult::Error(sid, e));
                    }
                }
            }
        });
    }

    // ─── Reports ──────────────────────────────────────────────────

    pub fn generate_report(
        &self,
        data: sm_reports::ReportData,
        format: sm_reports::ReportFormat,
    ) {
        let reports = self.reports.clone();
        let pending = self.pending_results.clone();

        self.runtime.spawn(async move {
            match reports.generate(&data, &format) {
                Ok(bytes) => {
                    pending
                        .lock()
                        .await
                        .push(AsyncResult::ReportGenerated(
                            data.title,
                            bytes,
                        ));
                }
                Err(e) => {
                    pending
                        .lock()
                        .await
                        .push(AsyncResult::Error(ServerId::nil(), e));
                }
            }
        });
    }

    // ─── Results ──────────────────────────────────────────────────

    pub fn poll_results(&self) -> Vec<AsyncResult> {
        let mut pending = self.pending_results.blocking_lock();
        let results = pending.drain(..).collect::<Vec<_>>();
        results
    }

    // ─── Vault ────────────────────────────────────────────────────

    pub fn store_credential(&self, key: &str, cred: &sm_core::types::Credential) -> Result<(), String> {
        let mut vault = self.vault.blocking_lock();
        if let Some(v) = vault.as_mut() {
            v.store(key, cred).map_err(|e| e)
        } else {
            Err("vault no inicializado".into())
        }
    }

    // ─── Scheduler ────────────────────────────────────────────────

    pub fn schedule_task(&self, job: sm_core::traits::JobDefinition) -> sm_core::id::JobId {
        let scheduler = self.scheduler.clone();
        self.runtime
            .block_on(async move { scheduler.lock().await.schedule(job) })
    }

    pub fn list_tasks(&self) -> Vec<sm_core::traits::Job> {
        let scheduler = self.scheduler.clone();
        self.runtime
            .block_on(async move { scheduler.lock().await.list().into_iter().cloned().collect() })
    }
}

impl Default for AppContext {
    fn default() -> Self {
        Self::new()
    }
}
