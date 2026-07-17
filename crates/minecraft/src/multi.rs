use serde::{Deserialize, Serialize};
use sm_net::shell;
use sm_net::SshClient;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use crate::*;

/// Multi-server Minecraft manager.
/// Manages multiple Minecraft servers on potentially different SSH connections.
pub struct MultiMinecraftManager {
    servers: Arc<Mutex<HashMap<String, MinecraftServerInstance>>>,
}

pub struct MinecraftServerInstance {
    pub config: crate::MinecraftServer,
    pub manager: Arc<crate::MinecraftManager>,
    pub rcon: Option<Arc<RconClient>>,
    pub log_stream_active: bool,
}

impl MultiMinecraftManager {
    pub fn new() -> Self {
        Self {
            servers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn add_server(
        &self,
        name: &str,
        config: crate::MinecraftServer,
        ssh: Arc<SshClient>,
    ) -> Result<(), String> {
        shell::validate_server_name(name)?;
        let manager = Arc::new(crate::MinecraftManager::new(ssh, &config.directory));

        let instance = MinecraftServerInstance {
            config,
            manager,
            rcon: None,
            log_stream_active: false,
        };

        self.servers.lock().await.insert(name.to_string(), instance);
        info!("Servidor Minecraft '{}' registrado", name);
        Ok(())
    }

    pub async fn remove_server(&self, name: &str) -> Result<(), String> {
        let mut servers = self.servers.lock().await;
        if servers.remove(name).is_some() {
            info!("Servidor Minecraft '{}' removido", name);
            Ok(())
        } else {
            Err(format!("servidor '{}' no encontrado", name))
        }
    }

    pub async fn list_servers(&self) -> Vec<String> {
        self.servers.lock().await.keys().cloned().collect()
    }

    pub async fn get_server(&self, name: &str) -> Option<crate::MinecraftServer> {
        self.servers
            .lock()
            .await
            .get(name)
            .map(|i| i.config.clone())
    }

    pub async fn create_server(&self, name: &str) -> Result<(), String> {
        let servers = self.servers.lock().await;
        if let Some(instance) = servers.get(name) {
            instance.manager.create(&instance.config).await
        } else {
            Err(format!("servidor '{}' no encontrado", name))
        }
    }

    pub async fn start_server(&self, name: &str) -> Result<(), String> {
        let servers = self.servers.lock().await;
        if let Some(instance) = servers.get(name) {
            let jar = format!("server-{}.jar", instance.config.version);
            instance
                .manager
                .start(&jar, instance.config.max_ram_mb)
                .await
        } else {
            Err(format!("servidor '{}' no encontrado", name))
        }
    }

    pub async fn stop_server(&self, name: &str) -> Result<(), String> {
        let servers = self.servers.lock().await;
        if let Some(instance) = servers.get(name) {
            instance.manager.stop().await
        } else {
            Err(format!("servidor '{}' no encontrado", name))
        }
    }

    pub async fn restart_server(&self, name: &str) -> Result<(), String> {
        let servers = self.servers.lock().await;
        if let Some(instance) = servers.get(name) {
            let jar = format!("server-{}.jar", instance.config.version);
            instance
                .manager
                .restart(&jar, instance.config.max_ram_mb)
                .await
        } else {
            Err(format!("servidor '{}' no encontrado", name))
        }
    }

    pub async fn kill_server(&self, name: &str) -> Result<(), String> {
        let servers = self.servers.lock().await;
        if let Some(instance) = servers.get(name) {
            instance.manager.kill().await
        } else {
            Err(format!("servidor '{}' no encontrado", name))
        }
    }

    pub async fn send_command(&self, name: &str, command: &str) -> Result<(), String> {
        let servers = self.servers.lock().await;
        if let Some(instance) = servers.get(name) {
            // Try RCON first if available
            if let Some(rcon) = &instance.rcon {
                if rcon.is_connected().await {
                    return rcon.send_command(command).await;
                }
            }
            // Fall back to screen stuff
            instance.manager.send_command(command).await
        } else {
            Err(format!("servidor '{}' no encontrado", name))
        }
    }

    pub async fn get_status(&self, name: &str) -> Result<crate::MinecraftStatus, String> {
        let servers = self.servers.lock().await;
        if let Some(instance) = servers.get(name) {
            instance.manager.status().await
        } else {
            Err(format!("servidor '{}' no encontrado", name))
        }
    }

    pub async fn get_logs(&self, name: &str, lines: u32) -> Result<String, String> {
        let servers = self.servers.lock().await;
        if let Some(instance) = servers.get(name) {
            instance.manager.get_recent_logs(lines).await
        } else {
            Err(format!("servidor '{}' no encontrado", name))
        }
    }

    pub async fn create_backup(
        &self,
        name: &str,
        backup_name: &str,
        backup_dir: &str,
    ) -> Result<crate::BackupInfo, String> {
        let servers = self.servers.lock().await;
        if let Some(instance) = servers.get(name) {
            instance
                .manager
                .create_backup(backup_name, backup_dir)
                .await
        } else {
            Err(format!("servidor '{}' no encontrado", name))
        }
    }

    pub async fn list_backups(
        &self,
        name: &str,
        backup_dir: &str,
    ) -> Result<Vec<crate::BackupInfo>, String> {
        let servers = self.servers.lock().await;
        if let Some(instance) = servers.get(name) {
            instance.manager.list_backups(backup_dir).await
        } else {
            Err(format!("servidor '{}' no encontrado", name))
        }
    }

    pub async fn search_plugins(
        &self,
        _name: &str,
        query: &str,
    ) -> Result<Vec<crate::ModrinthProject>, String> {
        // Use any available manager to search (search is remote-independent)
        let servers = self.servers.lock().await;
        if let Some((_, instance)) = servers.iter().next() {
            instance.manager.search_plugin(query).await
        } else {
            Err("no hay servidores Minecraft configurados".into())
        }
    }

    pub async fn install_plugin(&self, name: &str, slug: &str) -> Result<(), String> {
        let servers = self.servers.lock().await;
        if let Some(instance) = servers.get(name) {
            instance.manager.install_plugin(slug).await
        } else {
            Err(format!("servidor '{}' no encontrado", name))
        }
    }

    pub async fn list_plugins(&self, name: &str) -> Result<Vec<crate::MinecraftPlugin>, String> {
        let servers = self.servers.lock().await;
        if let Some(instance) = servers.get(name) {
            instance.manager.list_plugins().await
        } else {
            Err(format!("servidor '{}' no encontrado", name))
        }
    }

    pub async fn remove_plugin(&self, name: &str, plugin_name: &str) -> Result<(), String> {
        let servers = self.servers.lock().await;
        if let Some(instance) = servers.get(name) {
            instance.manager.remove_plugin(plugin_name).await
        } else {
            Err(format!("servidor '{}' no encontrado", name))
        }
    }

    pub async fn get_network_info(&self, name: &str) -> Result<crate::NetworkInfo, String> {
        let servers = self.servers.lock().await;
        if let Some(instance) = servers.get(name) {
            instance
                .manager
                .get_network_info(instance.config.port)
                .await
        } else {
            Err(format!("servidor '{}' no encontrado", name))
        }
    }

    /// Check if a plugin needs updating by comparing installed vs latest Modrinth version
    pub async fn check_plugin_updates(
        &self,
        name: &str,
    ) -> Result<Vec<PluginUpdateInfo>, String> {
        let servers = self.servers.lock().await;
        if let Some(instance) = servers.get(name) {
            let installed = instance.manager.list_plugins().await?;
            let mut updates = Vec::new();
            for plugin in &installed {
                // Query Modrinth for latest version
                let slug = &plugin.name;
                let url = format!("https://api.modrinth.com/v2/project/{}/version", slug);
                let client = reqwest::Client::new();
                if let Ok(resp) = client
                    .get(&url)
                    .header("User-Agent", "server-manager/0.5")
                    .send()
                    .await
                {
                    if let Ok(versions) = resp.json::<serde_json::Value>().await {
                        if let Some(latest) = versions.as_array().and_then(|a| a.first()) {
                            let latest_version = latest["version_number"]
                                .as_str()
                                .unwrap_or("?")
                                .to_string();
                            if latest_version != plugin.version {
                                updates.push(PluginUpdateInfo {
                                    name: plugin.name.clone(),
                                    current_version: plugin.version.clone(),
                                    latest_version,
                                    slug: slug.to_string(),
                                });
                            }
                        }
                    }
                }
            }
            Ok(updates)
        } else {
            Err(format!("servidor '{}' no encontrado", name))
        }
    }

    /// Enable RCON for a server
    pub async fn enable_rcon(
        &self,
        name: &str,
        password: &str,
        port: u16,
    ) -> Result<(), String> {
        let mut servers = self.servers.lock().await;
        if let Some(instance) = servers.get_mut(name) {
            // Enable RCON in server.properties
            instance
                .manager
                .write_file("server.properties", &format!(
                    "enable-rcon=true\nrcon.port={}\nrcon.password={}\n",
                    port, password
                ))
                .await?;

            let rcon = Arc::new(RconClient::new(
                instance.config.directory.clone(),
                password.to_string(),
                port,
            ));
            instance.rcon = Some(rcon);
            info!("RCON habilitado para servidor '{}'", name);
            Ok(())
        } else {
            Err(format!("servidor '{}' no encontrado", name))
        }
    }
}

impl Default for MultiMinecraftManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUpdateInfo {
    pub name: String,
    pub current_version: String,
    pub latest_version: String,
    pub slug: String,
}
