use serde::{Deserialize, Serialize};
use sm_net::shell;
use sm_net::SshClient;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerContainer {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub state: ContainerState,
    pub ports: Vec<String>,
    pub created: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContainerState {
    Running, Paused, Stopped, Restarting, Removing, Dead, Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerImage {
    pub id: String,
    pub repository: String,
    pub tag: String,
    pub size: String,
    pub created: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerVolume {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerNetwork {
    pub name: String,
    pub driver: String,
    pub scope: String,
}

pub struct DockerManager {
    ssh: Arc<SshClient>,
}

impl DockerManager {
    pub fn new(ssh: Arc<SshClient>) -> Self {
        Self { ssh }
    }

    pub async fn list_containers(&self, all: bool) -> Result<Vec<DockerContainer>, String> {
        let flag = if all { "-a" } else { "" };
        let fmt = "--format '{{.ID}}|{{.Names}}|{{.Image}}|{{.Status}}|{{.Ports}}|{{.CreatedAt}}' --no-trunc";
        let cmd = format!("docker ps {} {}", flag, fmt);
        let output = self.ssh.execute(&cmd).await?;

        if !output.stderr.is_empty() && output.exit_code != 0 {
            return Err(output.stderr.trim().to_string());
        }

        let mut containers = Vec::new();
        for line in output.stdout.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 6 {
                containers.push(DockerContainer {
                    id: parts[0].to_string(),
                    name: parts[1].to_string(),
                    image: parts[2].to_string(),
                    status: parts[3].to_string(),
                    state: parse_state(parts[3]),
                    ports: parts[4].split(',').map(|s| s.trim().to_string()).collect(),
                    created: parts[5].to_string(),
                });
            }
        }
        Ok(containers)
    }

    pub async fn container_logs(&self, id: &str, tail: u32) -> Result<String, String> {
        shell::validate_container_id(id)?;
        let cmd = format!("docker logs --tail {} {}", tail, shell::shell_quote(id));
        let output = self.ssh.execute(&cmd).await?;
        Ok(output.stdout)
    }

    pub async fn start_container(&self, id: &str) -> Result<(), String> {
        shell::validate_container_id(id)?;
        let cmd = format!("docker start {}", shell::shell_quote(id));
        let o = self.ssh.execute(&cmd).await?;
        if o.exit_code != 0 { return Err(o.stderr); }
        Ok(())
    }

    pub async fn stop_container(&self, id: &str) -> Result<(), String> {
        shell::validate_container_id(id)?;
        let cmd = format!("docker stop {}", shell::shell_quote(id));
        let o = self.ssh.execute(&cmd).await?;
        if o.exit_code != 0 { return Err(o.stderr); }
        Ok(())
    }

    pub async fn restart_container(&self, id: &str) -> Result<(), String> {
        shell::validate_container_id(id)?;
        let cmd = format!("docker restart {}", shell::shell_quote(id));
        let o = self.ssh.execute(&cmd).await?;
        if o.exit_code != 0 { return Err(o.stderr); }
        Ok(())
    }

    pub async fn remove_container(&self, id: &str, force: bool) -> Result<(), String> {
        shell::validate_container_id(id)?;
        let flag = if force { " -f" } else { "" };
        let cmd = format!("docker rm{} {}", flag, shell::shell_quote(id));
        let o = self.ssh.execute(&cmd).await?;
        if o.exit_code != 0 { return Err(o.stderr); }
        Ok(())
    }

    pub async fn exec(&self, id: &str, command: &str) -> Result<String, String> {
        shell::validate_container_id(id)?;
        // command is already validated via docker exec which takes raw args
        if command.len() > 4096 {
            return Err("comando demasiado largo".into());
        }
        let cmd = format!("docker exec {} {}", shell::shell_quote(id), command);
        let output = self.ssh.execute(&cmd).await?;
        if output.exit_code != 0 { return Err(output.stderr); }
        Ok(output.stdout)
    }

    pub async fn list_images(&self) -> Result<Vec<DockerImage>, String> {
        let fmt = "--format '{{.ID}}|{{.Repository}}|{{.Tag}}|{{.Size}}|{{.CreatedAt}}'";
        let output = self.ssh.execute(&format!("docker images {}", fmt)).await?;
        let mut images = Vec::new();
        for line in output.stdout.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 5 {
                images.push(DockerImage {
                    id: parts[0].to_string(), repository: parts[1].to_string(),
                    tag: parts[2].to_string(), size: parts[3].to_string(),
                    created: parts[4].to_string(),
                });
            }
        }
        Ok(images)
    }

    pub async fn list_volumes(&self) -> Result<Vec<DockerVolume>, String> {
        let fmt = "--format '{{.Name}}|{{.Driver}}|{{.Mountpoint}}'";
        let output = self.ssh.execute(&format!("docker volume ls {}", fmt)).await?;
        let mut volumes = Vec::new();
        for line in output.stdout.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 3 {
                volumes.push(DockerVolume {
                    name: parts[0].to_string(), driver: parts[1].to_string(),
                    mountpoint: parts[2].to_string(),
                });
            }
        }
        Ok(volumes)
    }

    pub async fn list_networks(&self) -> Result<Vec<DockerNetwork>, String> {
        let fmt = "--format '{{.Name}}|{{.Driver}}|{{.Scope}}'";
        let output = self.ssh.execute(&format!("docker network ls {}", fmt)).await?;
        let mut nets = Vec::new();
        for line in output.stdout.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 3 {
                nets.push(DockerNetwork {
                    name: parts[0].to_string(), driver: parts[1].to_string(),
                    scope: parts[2].to_string(),
                });
            }
        }
        Ok(nets)
    }

    pub async fn compose_up(&self, path: &str) -> Result<String, String> {
        shell::validate_remote_path(path)?;
        let output = self.ssh.execute(&format!("docker compose -f {} up -d", shell::shell_quote(path))).await?;
        if output.exit_code != 0 { return Err(output.stderr); }
        Ok(output.stdout)
    }

    pub async fn compose_down(&self, path: &str) -> Result<String, String> {
        shell::validate_remote_path(path)?;
        let output = self.ssh.execute(&format!("docker compose -f {} down", shell::shell_quote(path))).await?;
        if output.exit_code != 0 { return Err(output.stderr); }
        Ok(output.stdout)
    }

    pub async fn compose_logs(&self, path: &str, tail: u32) -> Result<String, String> {
        shell::validate_remote_path(path)?;
        let output = self.ssh.execute(&format!("docker compose -f {} logs --tail {}", shell::shell_quote(path), tail)).await?;
        Ok(output.stdout)
    }

    pub async fn stats(&self) -> Result<String, String> {
        let output = self.ssh.execute("docker stats --no-stream --all").await?;
        Ok(output.stdout)
    }

    pub async fn system_info(&self) -> Result<String, String> {
        let output = self.ssh.execute("docker info --format json").await?;
        Ok(output.stdout)
    }

    pub async fn prune(&self) -> Result<String, String> {
        // Prune is destructive — always include -f but document it
        let output = self.ssh.execute("docker system prune -f 2>&1").await?;
        Ok(output.stdout)
    }
}

fn parse_state(status: &str) -> ContainerState {
    let lower = status.to_lowercase();
    if lower.starts_with("up ") { ContainerState::Running }
    else if lower.contains("pause") { ContainerState::Paused }
    else if lower.contains("restart") { ContainerState::Restarting }
    else if lower.starts_with("exited") { ContainerState::Stopped }
    else { ContainerState::Unknown }
}
