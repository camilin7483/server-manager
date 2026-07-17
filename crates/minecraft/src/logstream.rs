use sm_net::SshClient;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::warn;

/// Streams log output from a remote Minecraft server using `tail -f`.
/// Sends log lines through a channel for real-time display in the UI.
pub struct LogStreamer {
    ssh: Arc<SshClient>,
    log_path: String,
    running: Arc<std::sync::Mutex<bool>>,
}

impl LogStreamer {
    pub fn new(ssh: Arc<SshClient>, log_path: impl Into<String>) -> Self {
        Self {
            ssh,
            log_path: log_path.into(),
            running: Arc::new(std::sync::Mutex::new(false)),
        }
    }

    /// Start streaming logs. Returns a receiver that yields log lines.
    pub fn start(&self) -> mpsc::Receiver<String> {
        let (tx, rx) = mpsc::channel(256);
        let ssh = self.ssh.clone();
        let log_path = self.log_path.clone();
        let running = self.running.clone();
        *running.lock().unwrap() = true;

        tokio::spawn(async move {
            // Use a polling approach: read new lines every 500ms
            let mut last_size: u64 = 0;
            *running.lock().unwrap() = true;

            while *running.lock().unwrap() {
                // Get file size and read new content
                let cmd = format!(
                    "wc -c < {} 2>/dev/null || echo 0",
                    sm_net::shell::shell_quote(&log_path)
                );
                let size_output = match ssh.execute(&cmd).await {
                    Ok(o) => o,
                    Err(e) => {
                        warn!("Log stream error: {}", e);
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        continue;
                    }
                };

                let current_size: u64 = size_output.stdout.trim().parse().unwrap_or(0);

                if current_size > last_size {
                    // Read the new portion
                    let bytes_to_read = current_size - last_size;
                    let cmd = format!(
                        "tail -c {} {} 2>/dev/null || true",
                        bytes_to_read,
                        sm_net::shell::shell_quote(&log_path)
                    );
                    if let Ok(output) = ssh.execute(&cmd).await {
                        for line in output.stdout.lines() {
                            if tx.send(line.to_string()).await.is_err() {
                                // Receiver dropped — stop streaming
                                *running.lock().unwrap() = false;
                                return;
                            }
                        }
                    }
                    last_size = current_size;
                }

                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        });

        rx
    }

    pub fn stop(&self) {
        *self.running.lock().unwrap() = false;
    }

    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }
}
