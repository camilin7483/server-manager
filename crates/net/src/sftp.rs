use crate::shell;
use crate::client::SshClient;
use std::sync::Arc;

pub struct SftpClient {
    ssh: Arc<SshClient>,
    root_dir: String,
}

#[derive(Debug, Clone)]
pub struct RemoteFile {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: u64,
    pub permissions: String,
}

impl SftpClient {
    pub fn new(ssh: Arc<SshClient>, root_dir: impl Into<String>) -> Self {
        Self { ssh, root_dir: root_dir.into() }
    }

    fn safe_path(&self, path: &str) -> Result<String, String> {
        let validated = shell::validate_remote_path(path)?;
        // Ensure path stays within root_dir
        if !validated.starts_with(&self.root_dir) && validated != "/" {
            return Err(format!("acceso fuera del directorio raíz: {}", validated));
        }
        Ok(validated)
    }

    pub async fn list(&self, path: &str) -> Result<Vec<RemoteFile>, String> {
        let safe = self.safe_path(path)?;
        let cmd = format!("ls -la --time-style=+%s {}", shell::shell_quote(&safe));
        let output = self.ssh.execute(&cmd).await?;
        if output.exit_code != 0 { return Err(output.stderr); }

        let mut files = Vec::new();
        for line in output.stdout.lines().skip(1) {
            if line.is_empty() || line.starts_with("total ") { continue; }
            if let Some(file) = parse_ls_line(line) {
                files.push(file);
            }
        }
        Ok(files)
    }

    pub async fn read(&self, path: &str) -> Result<Vec<u8>, String> {
        let safe = self.safe_path(path)?;
        let cmd = format!("cat {}", shell::shell_quote(&safe));
        let output = self.ssh.execute(&cmd).await?;
        if output.exit_code != 0 { return Err(output.stderr); }
        Ok(output.stdout.into_bytes())
    }

    pub async fn write(&self, path: &str, data: &[u8]) -> Result<(), String> {
        let safe = self.safe_path(path)?;
        let encoded = base64_encode(data);
        let cmd = format!("echo {} | base64 -d > {}", shell::shell_quote(&encoded), shell::shell_quote(&safe));
        let output = self.ssh.execute(&cmd).await?;
        if output.exit_code != 0 { return Err(output.stderr); }
        Ok(())
    }

    pub async fn delete(&self, path: &str) -> Result<(), String> {
        let safe = self.safe_path(path)?;
        // For delete, explicitly reject rm -rf on /
        if safe == "/" || safe.is_empty() {
            return Err("no se puede eliminar el directorio raíz".into());
        }
        let cmd = format!("rm -rf {}", shell::shell_quote(&safe));
        let output = self.ssh.execute(&cmd).await?;
        if output.exit_code != 0 { return Err(output.stderr); }
        Ok(())
    }

    pub async fn mkdir(&self, path: &str) -> Result<(), String> {
        let safe = self.safe_path(path)?;
        let cmd = format!("mkdir -p {}", shell::shell_quote(&safe));
        let output = self.ssh.execute(&cmd).await?;
        if output.exit_code != 0 { return Err(output.stderr); }
        Ok(())
    }

    pub async fn rename(&self, from: &str, to: &str) -> Result<(), String> {
        let safe_from = self.safe_path(from)?;
        let safe_to = self.safe_path(to)?;
        let cmd = format!("mv {} {}", shell::shell_quote(&safe_from), shell::shell_quote(&safe_to));
        let output = self.ssh.execute(&cmd).await?;
        if output.exit_code != 0 { return Err(output.stderr); }
        Ok(())
    }

    pub async fn exists(&self, path: &str) -> Result<bool, String> {
        let safe = self.safe_path(path)?;
        let cmd = format!("test -e {}", shell::shell_quote(&safe));
        let output = self.ssh.execute(&cmd).await?;
        Ok(output.exit_code == 0)
    }
}

fn parse_ls_line(line: &str) -> Option<RemoteFile> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 7 { return None; }
    let permissions = parts[0].to_string();
    let is_dir = permissions.starts_with('d');
    let size: u64 = parts[4].parse().unwrap_or(0);
    let modified: u64 = parts[5].parse().unwrap_or(0);
    let name = parts[6..].join(" ");

    Some(RemoteFile { name, path: String::new(), is_dir, size, modified, permissions })
}

fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}
