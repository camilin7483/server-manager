mod logstream;
mod multi;
mod rcon;

pub use logstream::LogStreamer;
pub use multi::{MultiMinecraftManager, PluginUpdateInfo};
pub use rcon::RconClient;

use serde::{Deserialize, Serialize};
use sm_net::shell;
use sm_net::SshClient;
use std::sync::Arc;
use tracing::{error, info};

// ─── Types ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftServer {
    pub name: String,
    pub directory: String,
    pub jar_type: MinecraftJar,
    pub version: String,
    pub java_path: String,
    pub min_ram_mb: u64,
    pub max_ram_mb: u64,
    pub port: u16,
    pub online_mode: bool,
    pub difficulty: String,
    pub motd: String,
    pub max_players: u32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MinecraftJar {
    Vanilla, Paper, Purpur, Spigot, Fabric, Forge, NeoForge,
    Velocity, BungeeCord,
}

impl MinecraftJar {
    pub fn all() -> &'static [Self] {
        &[Self::Paper, Self::Purpur, Self::Vanilla, Self::Fabric, Self::Forge, Self::NeoForge, Self::Spigot, Self::Velocity, Self::BungeeCord]
    }

    pub fn name(&self) -> &'static str {
        match self { Self::Vanilla=>"Vanilla",Self::Paper=>"Paper",Self::Purpur=>"Purpur",Self::Spigot=>"Spigot",Self::Fabric=>"Fabric",Self::Forge=>"Forge",Self::NeoForge=>"NeoForge",Self::Velocity=>"Velocity",Self::BungeeCord=>"BungeeCord" }
    }

    pub fn is_modded(&self) -> bool { matches!(self, Self::Fabric | Self::Forge | Self::NeoForge) }

    pub fn supports_plugins(&self) -> bool { matches!(self, Self::Paper | Self::Purpur | Self::Spigot) }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftPlugin { pub name: String, pub version: String, pub enabled: bool, pub file: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftMod { pub name: String, pub version: String, pub loader: String, pub file: String, pub source: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftStatus { pub online: bool, pub version: String, pub players_online: u32, pub max_players: u32, pub players: Vec<String>, pub tps: Option<f64>, pub mspt: Option<f64>, pub uptime: String, pub ram_used_mb: u64, pub cpu_percent: f64 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo { pub local_ip: String, pub public_ip: String, pub port_open: bool, pub connection_address: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo { pub name: String, pub path: String, pub size_bytes: u64, pub created_at: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask { pub id: String, pub name: String, pub cron: String, pub action: TaskAction, pub enabled: bool }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskAction { Restart, Backup, Command(String), LogCleanup }

// ─── Manager ──────────────────────────────────────────────────────

pub struct MinecraftManager {
    ssh: Arc<SshClient>,
    server_dir: String,
}

impl MinecraftManager {
    pub fn new(ssh: Arc<SshClient>, server_dir: &str) -> Self { Self { ssh, server_dir: server_dir.to_string() } }

    // ─── Creation ─────────────────────────────────────────────────

    pub async fn create(&self, server: &MinecraftServer) -> Result<(), String> {
        shell::validate_server_name(&server.name)?;
        let dir = &self.server_dir;

        let url = self.get_download_url(&server.jar_type, &server.version).await?;
        let jar = format!("server-{}.jar", server.version);

        let cmds = vec![
            format!("mkdir -p {}", shell::shell_quote(dir)),
            format!("cd {} && curl -fsSL -o {} --max-time 300 {}", shell::shell_quote(dir), shell::shell_quote(&jar), shell::shell_quote(&url)),
            format!("echo 'eula=true' > {}/eula.txt", shell::shell_quote(dir)),
        ];
        for cmd in &cmds { let o = self.ssh.execute(cmd).await?; if o.exit_code != 0 { return Err(o.stderr); } }

        self.generate_properties(server).await?;
        info!("Servidor Minecraft creado: {} ({})", server.name, server.version);
        Ok(())
    }

    async fn get_download_url(&self, jar: &MinecraftJar, version: &str) -> Result<String, String> {
        shell::validate_server_name(version).map_err(|_| "versión inválida".to_string())?;
        match jar {
            MinecraftJar::Paper  => Ok(format!("https://api.papermc.io/v2/projects/paper/versions/{}/builds/latest/downloads/paper-{}-latest.jar", version, version)),
            MinecraftJar::Purpur => Ok(format!("https://api.purpurmc.org/v2/purpur/{}/latest/download", version)),
            MinecraftJar::Velocity => Ok(format!("https://api.papermc.io/v2/projects/velocity/versions/{}/builds/latest/downloads/velocity-{}-latest.jar", version, version)),
            MinecraftJar::Fabric => Ok(format!("https://meta.fabricmc.net/v2/versions/loader/{}/0.16.10/1.0.1/server/jar", version)),
            MinecraftJar::Vanilla => Ok(format!("https://piston-data.mojang.com/v1/objects/{}/server.jar", self.get_vanilla_hash(version).await?)),
            _ => Err(format!("descarga automática de {} no soportada aún", jar.name())),
        }
    }

    async fn get_vanilla_hash(&self, version: &str) -> Result<String, String> {
        let url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
        let client = reqwest::Client::new();
        let manifest: serde_json::Value = client.get(url).send().await.map_err(|e| format!("error de red: {}",e))?.json().await.map_err(|e| format!("json: {}",e))?;
        for v in manifest["versions"].as_array().ok_or("no versions")? {
            if v["id"].as_str() == Some(version) {
                let vurl = v["url"].as_str().ok_or("no url")?;
                let detail: serde_json::Value = client.get(vurl).send().await.map_err(|e| format!("error: {}",e))?.json().await.map_err(|e| format!("json: {}",e))?;
                return detail["downloads"]["server"]["sha1"].as_str().map(|s|s.to_string()).ok_or("no sha1".into());
            }
        }
        Err(format!("versión {} no encontrada en Mojang", version))
    }

    async fn generate_properties(&self, srv: &MinecraftServer) -> Result<(), String> {
        let dir = shell::shell_quote(&self.server_dir);
        let props = format!(
            "server-port={}\nonline-mode={}\ndifficulty={}\nmotd={}\nmax-players={}\nenable-command-block=true\nspawn-protection=0\nview-distance=10\n",
            srv.port, srv.online_mode, srv.difficulty, srv.motd, srv.max_players
        );
        let cmd = format!("cat > {}/server.properties << 'EOPROPS'\n{}EOPROPS", dir, props);
        let o = self.ssh.execute(&cmd).await?;
        if o.exit_code != 0 { return Err(o.stderr); }
        Ok(())
    }

    // ─── Control ──────────────────────────────────────────────────

    pub async fn start(&self, jar: &str, max_ram_mb: u64) -> Result<(), String> {
        shell::validate_filename(jar)?;
        let dir = shell::shell_quote(&self.server_dir);
        let cmd = format!("cd {} && screen -dmS mcserver java -Xms512M -Xmx{}M -jar {} nogui", dir, max_ram_mb, shell::shell_quote(jar));
        let o = self.ssh.execute(&cmd).await?;
        if o.exit_code != 0 { return Err(o.stderr); }
        info!("Minecraft iniciado: {}", jar);
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), String> { self.send_command("stop").await }

    pub async fn restart(&self, jar: &str, max_ram_mb: u64) -> Result<(), String> { self.stop().await; tokio::time::sleep(std::time::Duration::from_secs(5)).await; self.start(jar, max_ram_mb).await }

    pub async fn kill(&self) -> Result<(), String> { self.ssh.execute("screen -S mcserver -X quit 2>/dev/null; pkill -f 'server-' 2>/dev/null; true").await?; Ok(()) }

    pub async fn delete_server(&self) -> Result<(), String> {
        self.kill().await?;
        let dir = shell::shell_quote(&self.server_dir);
        if dir == "'/'" || dir == "''" || dir == "'/home'" { return Err("ruta protegida".into()); }
        let o = self.ssh.execute(&format!("rm -rf {}", dir)).await?;
        if o.exit_code != 0 { return Err(o.stderr); }
        Ok(())
    }

    // ─── Console ──────────────────────────────────────────────────

    pub async fn send_command(&self, command: &str) -> Result<(), String> {
        shell::validate_mc_command(command)?;
        let cmd = format!("screen -S mcserver -X stuff '{}\\n'", command.replace('\'', "'\\''"));
        self.ssh.execute(&cmd).await?;
        Ok(())
    }

    pub async fn get_recent_logs(&self, lines: u32) -> Result<String, String> {
        let dir = shell::shell_quote(&self.server_dir);
        let o = self.ssh.execute(&format!("tail -{} {}/logs/latest.log 2>/dev/null || echo 'no logs'", lines, dir)).await?;
        Ok(o.stdout)
    }

    pub async fn get_log(&self) -> Result<String, String> { self.get_recent_logs(100).await }

    // ─── Status ───────────────────────────────────────────────────

    pub async fn status(&self) -> Result<MinecraftStatus, String> {
        let dir = shell::shell_quote(&self.server_dir);
        let running = self.ssh.execute("screen -ls 2>/dev/null | grep mcserver || true").await?;
        let online = !running.stdout.trim().is_empty();

        if !online { return Ok(MinecraftStatus { online: false, version: String::new(), players_online: 0, max_players: 0, players: vec![], tps: None, mspt: None, uptime: "offline".into(), ram_used_mb: 0, cpu_percent: 0.0 }); }

        let log = self.ssh.execute(&format!("tail -50 {}/logs/latest.log 2>/dev/null", dir)).await?.stdout;
        let mut players = Vec::new();
        let mut version = String::new();
        let mut max_players = 0u32;
        for line in log.lines() {
            if line.contains("players online:") { if let Some(p) = line.split(':').nth(1) { for n in p.split(',') { let n=n.trim(); if !n.is_empty() { players.push(n.to_string()); }}}}
            if line.contains("Starting minecraft server version") { if let Some(v) = line.rsplit(' ').next() { version = v.trim().to_string(); }}
            if line.contains("max_players") { max_players = 20; }
        }
        let tps = self.get_tps().await;
        let ram = self.ssh.execute("ps -o rss= -C java 2>/dev/null | awk '{s+=$1} END {print s}'").await?.stdout.trim().parse::<u64>().unwrap_or(0) / 1024;

        Ok(MinecraftStatus { online, version, players_online: players.len() as u32, max_players, players, tps, mspt: None, uptime: "running".into(), ram_used_mb: ram, cpu_percent: 0.0 })
    }

    pub async fn get_tps(&self) -> Option<f64> {
        let dir = shell::shell_quote(&self.server_dir);
        let log = self.ssh.execute(&format!("grep -i 'tps' {}/logs/latest.log 2>/dev/null | tail -5", dir)).await.ok()?.stdout;
        for line in log.lines() { for part in line.split_whitespace() { let v=part.trim_end_matches(',').trim_end_matches('s'); if let Ok(t)=v.parse::<f64>() { if t>0.0 && t<=20.0 { return Some(t); }}}}
        None
    }

    // ─── Files ────────────────────────────────────────────────────

    pub async fn list_files(&self, subdir: &str) -> Result<Vec<RemoteFile>, String> {
        let safe = if subdir.is_empty() { self.server_dir.clone() } else { format!("{}/{}", self.server_dir, subdir) };
        let path = shell::validate_remote_path(&safe)?;
        let dir = shell::shell_quote(&path);
        let o = self.ssh.execute(&format!("ls -la --time-style=+%s {}", dir)).await?;
        let mut files = Vec::new();
        for line in o.stdout.lines().skip(1) {
            if line.is_empty() || line.starts_with("total ") { continue; }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 7 {
                files.push(RemoteFile { name: parts[6..].join(" "), path: format!("{}/{}", path, parts[6..].join(" ")), is_dir: parts[0].starts_with('d'), size: parts[4].parse().unwrap_or(0), modified: parts[5].parse().unwrap_or(0), permissions: parts[0].to_string() });
            }
        }
        Ok(files)
    }

    pub async fn read_file(&self, rel_path: &str) -> Result<String, String> {
        shell::validate_filename(rel_path.split('/').last().unwrap_or(""))?;
        let safe = shell::validate_remote_path(&format!("{}/{}", self.server_dir, rel_path))?;
        let o = self.ssh.execute(&format!("cat {}", shell::shell_quote(&safe))).await?;
        if o.exit_code != 0 { return Err(o.stderr); }
        Ok(o.stdout)
    }

    pub async fn write_file(&self, rel_path: &str, content: &str) -> Result<(), String> {
        shell::validate_filename(rel_path.split('/').last().unwrap_or(""))?;
        let safe = shell::validate_remote_path(&format!("{}/{}", self.server_dir, rel_path))?;
        let encoded = base64_encode(content.as_bytes());
        let o = self.ssh.execute(&format!("echo {} | base64 -d > {}", shell::shell_quote(&encoded), shell::shell_quote(&safe))).await?;
        if o.exit_code != 0 { return Err(o.stderr); }
        Ok(())
    }

    pub async fn delete_file(&self, rel_path: &str) -> Result<(), String> {
        let safe = shell::validate_remote_path(&format!("{}/{}", self.server_dir, rel_path))?;
        let o = self.ssh.execute(&format!("rm -rf {}", shell::shell_quote(&safe))).await?;
        if o.exit_code != 0 { return Err(o.stderr); }
        Ok(())
    }

    pub async fn mkdir(&self, rel_path: &str) -> Result<(), String> {
        let safe = shell::validate_remote_path(&format!("{}/{}", self.server_dir, rel_path))?;
        let o = self.ssh.execute(&format!("mkdir -p {}", shell::shell_quote(&safe))).await?;
        if o.exit_code != 0 { return Err(o.stderr); }
        Ok(())
    }

    // ─── Plugins (Modrinth API) ───────────────────────────────────

    pub async fn search_plugin(&self, query: &str) -> Result<Vec<ModrinthProject>, String> {
        let url = format!("https://api.modrinth.com/v2/search?query={}&limit=10&facets=[[\"categories:paper\"],[\"project_type:mod\"]]", urlencoding(query));
        let client = reqwest::Client::new();
        let resp: serde_json::Value = client.get(&url).header("User-Agent","server-manager/0.5").send().await.map_err(|e| format!("error: {}",e))?.json().await.map_err(|e| format!("json: {}",e))?;
        let mut projects = Vec::new();
        for hit in resp["hits"].as_array().unwrap_or(&vec![]) {
            projects.push(ModrinthProject {
                slug: hit["slug"].as_str().unwrap_or("").into(),
                title: hit["title"].as_str().unwrap_or("").into(),
                description: hit["description"].as_str().unwrap_or("").into(),
                downloads: hit["downloads"].as_u64().unwrap_or(0),
                versions: hit["versions"].as_array().map(|v|v.len()).unwrap_or(0),
            });
        }
        Ok(projects)
    }

    pub async fn install_plugin(&self, slug: &str) -> Result<(), String> {
        shell::validate_server_name(slug)?;
        let dir = shell::shell_quote(&format!("{}/plugins", self.server_dir));
        let _ = self.ssh.execute(&format!("mkdir -p {}", dir)).await?;
        let url = format!("https://api.modrinth.com/v2/project/{}/version", slug);
        let client = reqwest::Client::new();
        let versions: serde_json::Value = client.get(&url).header("User-Agent","server-manager/0.5").send().await.map_err(|e|format!("error: {}",e))?.json().await.map_err(|e|format!("json: {}",e))?;
        let latest = versions.as_array().and_then(|a|a.first()).ok_or("no versions")?;
        let dl_url = latest["files"].as_array().and_then(|a|a.first()).and_then(|f|f["url"].as_str()).ok_or("no download url")?;
        let fname = latest["files"].as_array().and_then(|a|a.first()).and_then(|f|f["filename"].as_str()).unwrap_or("plugin.jar");
        let cmd = format!("cd {} && curl -fsSL -o {} --max-time 120 {}", dir, shell::shell_quote(fname), shell::shell_quote(dl_url));
        let o = self.ssh.execute(&cmd).await?;
        if o.exit_code != 0 { return Err(o.stderr); }
        info!("Plugin instalado: {}", slug);
        Ok(())
    }

    pub async fn list_plugins(&self) -> Result<Vec<MinecraftPlugin>, String> {
        let dir = shell::shell_quote(&format!("{}/plugins", self.server_dir));
        let o = self.ssh.execute(&format!("ls -1 {}", dir)).await?;
        Ok(o.stdout.lines().filter(|l| l.ends_with(".jar")).map(|l| MinecraftPlugin { name: l.trim_end_matches(".jar").to_string(), version: "?".into(), enabled: true, file: l.to_string() }).collect())
    }

    pub async fn remove_plugin(&self, name: &str) -> Result<(), String> {
        shell::validate_filename(name)?;
        let dir = shell::shell_quote(&format!("{}/plugins", self.server_dir));
        let file = if name.ends_with(".jar") { name.to_string() } else { format!("{}.jar", name) };
        let o = self.ssh.execute(&format!("rm -f {}/{}", dir, shell::shell_quote(&file))).await?;
        if o.exit_code != 0 { return Err(o.stderr); }
        Ok(())
    }

    // ─── Mods ─────────────────────────────────────────────────────

    pub async fn list_mods(&self) -> Result<Vec<MinecraftMod>, String> {
        let dir = shell::shell_quote(&format!("{}/mods", self.server_dir));
        let o = self.ssh.execute(&format!("ls -1 {}", dir)).await?;
        Ok(o.stdout.lines().filter(|l| l.ends_with(".jar")).map(|l| MinecraftMod { name: l.to_string(), version: "?".into(), loader: "?".into(), file: l.to_string(), source: "manual".into() }).collect())
    }

    // ─── Backups ──────────────────────────────────────────────────

    pub async fn create_backup(&self, name: &str, backup_dir: &str) -> Result<BackupInfo, String> {
        self.send_command("save-all").await?;
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let safe_backup = shell::validate_remote_path(backup_dir)?;
        let date = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let fname = format!("{}_{}.tar.gz", name, date);
        let cmd = format!("mkdir -p {} && cd {} && tar czf {}/{} . 2>/dev/null", shell::shell_quote(&safe_backup), shell::shell_quote(&self.server_dir), shell::shell_quote(&safe_backup), shell::shell_quote(&fname));
        let o = self.ssh.execute(&cmd).await?;
        if o.exit_code != 0 { return Err(o.stderr); }
        let sz = self.ssh.execute(&format!("stat -c%s {}/{}", shell::shell_quote(&safe_backup), shell::shell_quote(&fname))).await?.stdout.trim().parse().unwrap_or(0);
        Ok(BackupInfo { name: fname.clone(), path: format!("{}/{}", safe_backup, fname), size_bytes: sz, created_at: date })
    }

    pub async fn list_backups(&self, backup_dir: &str) -> Result<Vec<BackupInfo>, String> {
        let safe = shell::validate_remote_path(backup_dir)?;
        let o = self.ssh.execute(&format!("ls -la --time-style=+%s {}", shell::shell_quote(&safe))).await?;
        let mut bk = Vec::new();
        for line in o.stdout.lines().skip(1) {
            if line.is_empty() || line.starts_with("total ") { continue; }
            let p: Vec<&str> = line.split_whitespace().collect();
            if p.len() >= 7 { bk.push(BackupInfo { name: p[6..].join(" "), path: format!("{}/{}", safe, p[6..].join(" ")), size_bytes: p[4].parse().unwrap_or(0), created_at: p[5].to_string() }); }
        }
        Ok(bk)
    }

    pub async fn restore_backup(&self, backup_path: &str) -> Result<(), String> {
        self.stop().await?;
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let safe = shell::validate_remote_path(backup_path)?;
        let dir = shell::shell_quote(&self.server_dir);
        let o = self.ssh.execute(&format!("rm -rf {}/* && tar xzf {} -C {} 2>/dev/null", dir, shell::shell_quote(&safe), dir)).await?;
        if o.exit_code != 0 { return Err(o.stderr); }
        Ok(())
    }

    pub async fn delete_backup(&self, backup_path: &str) -> Result<(), String> {
        let safe = shell::validate_remote_path(backup_path)?;
        self.ssh.execute(&format!("rm -f {}", shell::shell_quote(&safe))).await?;
        Ok(())
    }

    // ─── Remote Access ────────────────────────────────────────────

    pub async fn get_network_info(&self, port: u16) -> Result<NetworkInfo, String> {
        let local = self.ssh.execute("hostname -I 2>/dev/null | awk '{print $1}' || ip -4 addr show | grep -oP '(?<=inet\\s)\\d+(\\.\\d+){3}' | head -1").await?.stdout.trim().to_string();
        let public = self.ssh.execute("curl -fsSL --max-time 5 ifconfig.me 2>/dev/null || echo 'unknown'").await?.stdout.trim().to_string();
        let port_check = self.ssh.execute(&format!("ss -tlnp 2>/dev/null | grep ':{}' || true", port)).await?.stdout;
        let addr = if !public.is_empty() && public != "unknown" { format!("{}:{}", public, port) } else { format!("{}:{}", local, port) };

        Ok(NetworkInfo { local_ip: local, public_ip: public, port_open: !port_check.trim().is_empty(), connection_address: addr })
    }

    // ─── Scheduled Tasks ──────────────────────────────────────────

    pub fn default_tasks(directory: &str) -> Vec<ScheduledTask> {
        vec![
            ScheduledTask { id: "daily-backup".into(), name: "Backup Diario".into(), cron: "0 4 * * *".into(), action: TaskAction::Backup, enabled: false },
            ScheduledTask { id: "daily-restart".into(), name: "Reinicio Diario".into(), cron: "0 6 * * *".into(), action: TaskAction::Restart, enabled: false },
            ScheduledTask { id: "log-cleanup".into(), name: "Limpieza de Logs".into(), cron: "0 3 * * 0".into(), action: TaskAction::LogCleanup, enabled: false },
        ]
    }
}

// ─── Helpers ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteFile { pub name: String, pub path: String, pub is_dir: bool, pub size: u64, pub modified: u64, pub permissions: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthProject { pub slug: String, pub title: String, pub description: String, pub downloads: u64, pub versions: usize }

fn base64_encode(data: &[u8]) -> String { use base64::Engine; base64::engine::general_purpose::STANDARD.encode(data) }

fn urlencoding(s: &str) -> String { urlencoding::encode(s).into_owned() }
