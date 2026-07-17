use russh::client;
use russh::keys::load_secret_key;
use sm_core::types::Credential;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

pub struct SshClient {
    session: Arc<Mutex<Option<client::Handle<SshHandler>>>>,
    connected: Arc<Mutex<bool>>,
}

#[derive(Default)]
struct SshHandler {
    banner: Option<String>,
}

impl SshClient {
    pub fn new() -> Self {
        Self {
            session: Arc::new(Mutex::new(None)),
            connected: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn connect(
        &self,
        host: &str,
        port: u16,
        username: &str,
        credential: &Credential,
    ) -> Result<(), String> {
        let config = client::Config::default();
        let config = Arc::new(config);
        let sh = SshHandler::default();

        let addr = format!("{}:{}", host, port);
        let mut conn = client::connect(config, addr, sh)
            .await
            .map_err(|e| format!("error de conexión SSH: {}", e))?;

        let authenticated = match credential.auth_method {
            sm_core::types::AuthMethod::Key | sm_core::types::AuthMethod::KeyWithPassphrase => {
                let key_path = credential
                    .private_key_path
                    .as_deref()
                    .ok_or_else(|| "ruta de clave privada no especificada".to_string())?;
                let key = load_secret_key(Path::new(key_path), credential.passphrase.as_deref())
                    .map_err(|e| format!("error al cargar clave: {}", e))?;
                conn.authenticate_publickey(username, Arc::new(key))
                    .await
                    .map_err(|e| format!("autenticación SSH fallida: {}", e))?
            }
            sm_core::types::AuthMethod::Password => {
                let password = credential
                    .password
                    .as_deref()
                    .ok_or_else(|| "contraseña no especificada".to_string())?;
                conn.authenticate_password(username, password)
                    .await
                    .map_err(|e| format!("autenticación SSH fallida: {}", e))?
            }
            _ => {
                return Err("método de autenticación no soportado por SSH".into());
            }
        };

        if authenticated {
            info!("Conectado a {} via SSH como {}", host, username);
            let mut session = self.session.lock().await;
            *session = Some(conn);
            let mut c = self.connected.lock().await;
            *c = true;
            Ok(())
        } else {
            Err("autenticación rechazada".into())
        }
    }

    pub async fn execute(&self, command: &str) -> Result<SshOutput, String> {
        let session = self.session.lock().await;
        let conn = session
            .as_ref()
            .ok_or_else(|| "no hay sesión SSH activa".to_string())?;

        let mut channel = conn
            .channel_open_session()
            .await
            .map_err(|e| format!("error al abrir canal: {}", e))?;

        channel
            .exec(true, command.as_bytes())
            .await
            .map_err(|e| format!("error al ejecutar: {}", e))?;

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let mut exit_code: i32 = -1;

        loop {
            let msg = channel
                .wait()
                .await
                .ok_or_else(|| "canal cerrado inesperadamente".to_string())?;

            match msg {
                russh::ChannelMsg::Data { data } => {
                    stdout.extend_from_slice(&data);
                }
                russh::ChannelMsg::ExtendedData { data, .. } => {
                    stderr.extend_from_slice(&data);
                }
                russh::ChannelMsg::ExitStatus { exit_status } => {
                    exit_code = exit_status as i32;
                }
                russh::ChannelMsg::Eof | russh::ChannelMsg::Close => break,
                _ => {}
            }
        }

        Ok(SshOutput {
            stdout: String::from_utf8_lossy(&stdout).into_owned(),
            stderr: String::from_utf8_lossy(&stderr).into_owned(),
            exit_code,
        })
    }

    pub async fn disconnect(&self) {
        let mut session = self.session.lock().await;
        if let Some(conn) = session.take() {
            let _ = conn.disconnect(russh::Disconnect::ByApplication, "", "en").await;
        }
        let mut c = self.connected.lock().await;
        *c = false;
        info!("Sesión SSH cerrada");
    }

    pub async fn is_connected(&self) -> bool {
        *self.connected.lock().await
    }
}

#[derive(Debug, Clone)]
pub struct SshOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

#[async_trait::async_trait]
impl client::Handler for SshHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &russh_keys::key::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }

    async fn auth_banner(
        &mut self,
        banner: &str,
        _session: &mut client::Session,
    ) -> Result<(), Self::Error> {
        debug!("SSH banner: {}", banner);
        Ok(())
    }
}
