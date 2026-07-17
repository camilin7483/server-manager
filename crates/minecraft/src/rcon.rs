use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

/// RCON client for bidirectional Minecraft server console communication.
/// Implements the Source RCON protocol (used by Minecraft).
pub struct RconClient {
    host: String,
    port: u16,
    password: String,
    stream: Arc<Mutex<Option<TcpStream>>>,
    connected: Arc<Mutex<bool>>,
}

const RCON_AUTH: i32 = 3;
const RCON_AUTH_RESPONSE: i32 = 2;
const RCON_EXEC_COMMAND: i32 = 2;
const RCON_EXEC_RESPONSE: i32 = 0;
const RCON_PID: i32 = 0;

impl RconClient {
    pub fn new(host: impl Into<String>, password: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            password: password.into(),
            stream: Arc::new(Mutex::new(None)),
            connected: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn connect(&self) -> Result<(), String> {
        let addr = format!("{}:{}", self.host, self.port);
        let stream = TcpStream::connect(&addr)
            .await
            .map_err(|e| format!("RCON connect error: {}", e))?;

        *self.stream.lock().await = Some(stream);
        *self.connected.lock().await = true;

        // Authenticate
        self.send_packet(RCON_AUTH, &self.password).await?;
        let response = self.read_packet().await?;

        if response.id == -1 {
            *self.connected.lock().await = false;
            return Err("RCON authentication failed".into());
        }

        info!("RCON conectado a {}", addr);
        Ok(())
    }

    pub async fn disconnect(&self) {
        *self.stream.lock().await = None;
        *self.connected.lock().await = false;
        info!("RCON desconectado");
    }

    pub async fn is_connected(&self) -> bool {
        *self.connected.lock().await
    }

    pub async fn send_command(&self, command: &str) -> Result<(), String> {
        if !self.is_connected().await {
            return Err("RCON no conectado".into());
        }

        // Validate command
        if command.is_empty() || command.len() > 1400 {
            return Err("comando inválido (vacío o demasiado largo)".into());
        }

        self.send_packet(RCON_EXEC_COMMAND, command).await?;

        // Read response (may be multiple packets for long output)
        let mut output = String::new();
        loop {
            match tokio::time::timeout(
                std::time::Duration::from_secs(5),
                self.read_packet(),
            )
            .await
            {
                Ok(Ok(response)) => {
                    if response.id == RCON_PID {
                        output.push_str(&response.body);
                        if response.body.len() < 1000 {
                            break;
                        }
                    }
                }
                Ok(Err(e)) => {
                    warn!("RCON read error: {}", e);
                    break;
                }
                Err(_) => {
                    // Timeout — we have partial output
                    break;
                }
            }
        }

        Ok(())
    }

    pub async fn send_command_with_response(&self, command: &str) -> Result<String, String> {
        if !self.is_connected().await {
            return Err("RCON no conectado".into());
        }

        if command.is_empty() || command.len() > 1400 {
            return Err("comando inválido".into());
        }

        self.send_packet(RCON_EXEC_COMMAND, command).await?;

        let mut output = String::new();
        loop {
            match tokio::time::timeout(
                std::time::Duration::from_secs(5),
                self.read_packet(),
            )
            .await
            {
                Ok(Ok(response)) => {
                    if response.id == RCON_PID {
                        output.push_str(&response.body);
                        if response.body.len() < 1000 {
                            break;
                        }
                    }
                }
                _ => break,
            }
        }

        Ok(output)
    }

    async fn send_packet(&self, packet_type: i32, body: &str) -> Result<(), String> {
        let mut stream = self.stream.lock().await;
        let stream = stream
            .as_mut()
            .ok_or("RCON no conectado")?;

        let body_bytes = body.as_bytes();
        let length = 10 + body_bytes.len() as i32; // 4 (size) + 4 (id) + 2 (type + padding) + body

        let mut packet = Vec::with_capacity(length as usize + 4);
        packet.extend_from_slice(&length.to_le_bytes());
        packet.extend_from_slice(&RCON_PID.to_le_bytes());
        packet.extend_from_slice(&packet_type.to_le_bytes());
        packet.extend_from_slice(body_bytes);
        packet.push(0); // null terminator for body
        packet.push(0); // null terminator for packet

        stream
            .write_all(&packet)
            .await
            .map_err(|e| format!("RCON write error: {}", e))?;

        Ok(())
    }

    async fn read_packet(&self) -> Result<RconPacket, String> {
        let mut stream = self.stream.lock().await;
        let stream = stream
            .as_mut()
            .ok_or("RCON no conectado")?;

        // Read length (4 bytes)
        let mut length_buf = [0u8; 4];
        stream
            .read_exact(&mut length_buf)
            .await
            .map_err(|e| format!("RCON read length error: {}", e))?;
        let length = i32::from_le_bytes(length_buf);

        if length < 10 || length > 4096 {
            return Err(format!("RCON packet length invalid: {}", length));
        }

        // Read the rest of the packet
        let mut data = vec![0u8; length as usize];
        stream
            .read_exact(&mut data)
            .await
            .map_err(|e| format!("RCON read data error: {}", e))?;

        let id = i32::from_le_bytes(data[0..4].try_into().unwrap());
        let packet_type = i32::from_le_bytes(data[4..8].try_into().unwrap());

        // Body is everything after type, minus the two null terminators
        let body_end = data.len().saturating_sub(2);
        let body = String::from_utf8_lossy(&data[8..body_end]).to_string();

        Ok(RconPacket {
            id,
            packet_type,
            body,
        })
    }
}

struct RconPacket {
    id: i32,
    packet_type: i32,
    body: String,
}
