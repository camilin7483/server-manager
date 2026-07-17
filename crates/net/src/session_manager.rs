use sm_core::id::ServerId;
use sm_core::types::ConnectionInfo;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use super::client::{SshClient, SshOutput};

pub struct SessionManager {
    sessions: RwLock<HashMap<ServerId, Arc<SshClient>>>,
    connect_timeout: Duration,
}

impl SessionManager {
    pub fn new(timeout_secs: u64) -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            connect_timeout: Duration::from_secs(timeout_secs),
        }
    }

    pub async fn connect(
        &self,
        server_id: ServerId,
        info: &ConnectionInfo,
    ) -> Result<(), String> {
        let client = Arc::new(SshClient::new());

        let username = info
            .credential
            .username
            .as_deref()
            .ok_or_else(|| "username requerido".to_string())?;

        let connect_fut = client.connect(
            &info.host,
            info.port,
            username,
            &info.credential,
        );

        match tokio::time::timeout(self.connect_timeout, connect_fut).await {
            Ok(Ok(())) => {
                info!("Sesión establecida para servidor {:?}", server_id);
                self.sessions
                    .write()
                    .await
                    .insert(server_id, client.clone());
                Ok(())
            }
            Ok(Err(e)) => {
                error!("Error conectando a {:?}: {}", server_id, e);
                Err(e)
            }
            Err(_) => {
                error!("Timeout conectando a {:?}", server_id);
                Err(format!(
                    "timeout de conexión ({} segundos)",
                    self.connect_timeout.as_secs()
                ))
            }
        }
    }

    pub async fn disconnect(&self, server_id: ServerId) {
        if let Some(client) = self.sessions.write().await.remove(&server_id) {
            client.disconnect().await;
            info!("Sesión cerrada para servidor {:?}", server_id);
        }
    }

    pub async fn disconnect_all(&self) {
        let sessions = {
            let mut guard = self.sessions.write().await;
            std::mem::take(&mut *guard)
        };
        for (server_id, client) in sessions {
            client.disconnect().await;
            info!("Sesión cerrada para {:?}", server_id);
        }
    }

    pub async fn execute(
        &self,
        server_id: ServerId,
        command: &str,
    ) -> Result<SshOutput, String> {
        let sessions = self.sessions.read().await;
        let client = sessions
            .get(&server_id)
            .ok_or_else(|| format!("no hay sesión activa para {:?}", server_id))?;

        if !client.is_connected().await {
            return Err(format!("sesión desconectada para {:?}", server_id));
        }

        client.execute(command).await
    }

    pub async fn is_connected(&self, server_id: ServerId) -> bool {
        self.sessions
            .read()
            .await
            .get(&server_id)
            .map(|c| {
                let c = c.clone();
                tokio::task::spawn(async move { c.is_connected().await })
            })
            .is_some()
    }

    pub async fn active_sessions(&self) -> Vec<ServerId> {
        self.sessions.read().await.keys().copied().collect()
    }

    pub async fn session_count(&self) -> usize {
        self.sessions.read().await.len()
    }

    pub async fn run_keepalive(&self, server_id: ServerId, interval_secs: u64) {
        let sessions = self.sessions.read().await;
        let client = match sessions.get(&server_id) {
            Some(c) => c.clone(),
            None => return,
        };
        drop(sessions);

        loop {
            tokio::time::sleep(Duration::from_secs(interval_secs)).await;

            if !client.is_connected().await {
                warn!("Keepalive: sesión {:?} desconectada", server_id);
                self.disconnect(server_id).await;
                break;
            }

            match client.execute("echo keepalive").await {
                Ok(_) => {}
                Err(e) => {
                    warn!("Keepalive {:?} falló: {}", server_id, e);
                    self.disconnect(server_id).await;
                    break;
                }
            }
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new(30)
    }
}
