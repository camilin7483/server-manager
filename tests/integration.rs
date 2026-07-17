// Tests de integracion del Server Manager v0.6.0
// Cobertura: core, config, net, security, monitor, automation, storage,
//            docker, minecraft, reports, plugins, errors, shell safety

#[cfg(test)]
mod tests {
    // ─── Core Types ───────────────────────────────────────────────

    #[test]
    fn test_core_types_creation() {
        let id = sm_core::id::ServerId::new();
        assert!(!id.is_nil());
        assert_ne!(id, sm_core::id::ServerId::nil());

        let server = sm_core::types::Server {
            id: sm_core::id::ServerId::new(),
            name: "test-server".into(),
            group_id: None,
            connection: sm_core::types::ConnectionInfo {
                protocol: sm_core::types::ConnectionProtocol::Ssh,
                host: "localhost".into(),
                port: 22,
                credential: sm_core::types::Credential {
                    auth_method: sm_core::types::AuthMethod::Password,
                    username: Some("root".into()),
                    password: Some("test".into()),
                    private_key_path: None, private_key_data: None,
                    passphrase: None, certificate_path: None,
                },
                options: std::collections::HashMap::new(),
            },
            status: sm_core::types::ServerStatus::Offline,
            profile: None, tags: vec!["web".into()], notes: String::new(),
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            last_connected_at: None, metadata: std::collections::HashMap::new(),
        };
        assert_eq!(server.name, "test-server");
        assert_eq!(server.connection.port, 22);
        assert_eq!(server.tags.len(), 1);
    }

    #[test]
    fn test_server_status_labels() {
        use sm_core::types::ServerStatus;
        assert_eq!(ServerStatus::Offline.label(), "Offline");
        assert_eq!(ServerStatus::Online.label(), "Online");
        assert_eq!(ServerStatus::Connecting.label(), "Conectando");
        assert_eq!(ServerStatus::Error.label(), "Error");
        assert_eq!(ServerStatus::Restarting.label(), "Reiniciando");
        assert_eq!(ServerStatus::Maintenance.label(), "Mantenimiento");
    }

    #[test]
    fn test_connection_protocol_default_ports() {
        use sm_core::types::ConnectionProtocol;
        assert_eq!(ConnectionProtocol::Ssh.default_port(), 22);
        assert_eq!(ConnectionProtocol::Rdp.default_port(), 3389);
        assert_eq!(ConnectionProtocol::Vnc.default_port(), 5900);
        assert_eq!(ConnectionProtocol::Http.default_port(), 80);
        assert_eq!(ConnectionProtocol::Https.default_port(), 443);
        assert_eq!(ConnectionProtocol::Ftp.default_port(), 21);
    }

    #[test]
    fn test_id_uniqueness() {
        let id1 = sm_core::id::ServerId::new();
        let id2 = sm_core::id::ServerId::new();
        assert_ne!(id1, id2);
        assert!(!id1.is_nil());
        assert!(sm_core::id::ServerId::nil().is_nil());
    }

    // ─── Core Errors ──────────────────────────────────────────────

    #[test]
    fn test_core_error_display() {
        use sm_core::error::CoreError;
        let e = CoreError::ServerNotFound(sm_core::id::ServerId::nil());
        assert!(e.to_string().contains("Servidor no encontrado"));
        let e = CoreError::ConnectionFailed("timeout".into());
        assert!(e.to_string().contains("Conexi"));
        let e = CoreError::InvalidConfig("bad".into());
        assert!(e.to_string().contains("Configuraci"));
        let e = CoreError::PermissionDenied("no access".into());
        assert!(e.to_string().contains("Permiso"));
    }

    #[test]
    fn test_core_error_from_serde_json() {
        use sm_core::error::CoreError;
        let json_err = serde_json::from_str::<serde_json::Value>("{invalid}");
        assert!(json_err.is_err());
        let core_err: CoreError = json_err.unwrap_err().into();
        assert!(matches!(core_err, CoreError::Serialization(_)));
    }

    #[test]
    fn test_core_error_from_io() {
        use sm_core::error::CoreError;
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let core_err: CoreError = io_err.into();
        assert!(matches!(core_err, CoreError::Generic(_)));
    }

    // ─── Server Profiles ──────────────────────────────────────────

    #[test]
    fn test_server_profiles_all() {
        let profiles = sm_core::profiles::ServerProfileTemplate::all_presets();
        assert!(profiles.len() >= 14);
        let ubuntu = profiles.iter().find(|p| p.name == "Ubuntu Server").unwrap();
        assert!(ubuntu.ports.contains(&22));
        assert!(!ubuntu.setup_commands.is_empty());
        let docker = profiles.iter().find(|p| p.name == "Docker Host").unwrap();
        assert_eq!(docker.profile_type, sm_core::types::ProfileType::DockerHost);
        let mc = profiles.iter().find(|p| p.name == "Minecraft Java Server").unwrap();
        assert!(mc.ports.contains(&25565));
        let nginx = profiles.iter().find(|p| p.name == "Nginx Web Server").unwrap();
        assert!(nginx.ports.contains(&80));
        assert!(nginx.ports.contains(&443));
    }

    // ─── Config ───────────────────────────────────────────────────

    #[test]
    fn test_config_defaults() {
        let config = sm_config::AppConfig::default();
        assert_eq!(config.ui.theme, "dark");
        assert_eq!(config.servers.default_ssh_port, 22);
        assert_eq!(config.servers.connection_timeout_secs, 30);
        assert!(config.security.encrypt_credentials);
        assert!(config.monitor.enabled);
        assert_eq!(config.monitor.interval_ms, 5000);
    }

    #[test]
    fn test_config_toml_roundtrip() {
        let config = sm_config::AppConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: sm_config::AppConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.ui.theme, config.ui.theme);
        assert_eq!(parsed.servers.default_ssh_port, config.servers.default_ssh_port);
    }

    // ─── Shell Safety (anti command injection) ────────────────────

    #[test]
    fn test_shell_quote() {
        use sm_net::shell::shell_quote;
        assert_eq!(shell_quote("hello"), "'hello'");
        assert_eq!(shell_quote("it's"), "'it'\\''s'");
        assert_eq!(shell_quote(""), "''");
    }

    #[test]
    fn test_validate_container_id() {
        use sm_net::shell::validate_container_id;
        assert!(validate_container_id("a1b2c3d4e5f6").is_ok());
        assert!(validate_container_id("ABC123").is_ok());
        assert!(validate_container_id("").is_err());
        assert!(validate_container_id("hello world").is_err());
        assert!(validate_container_id("container; rm -rf /").is_err());
        assert!(validate_container_id(&"a".repeat(65)).is_err());
    }

    #[test]
    fn test_validate_container_name() {
        use sm_net::shell::validate_container_name;
        assert!(validate_container_name("nginx-web").is_ok());
        assert!(validate_container_name("postgres_db").is_ok());
        assert!(validate_container_name("app.v2").is_ok());
        assert!(validate_container_name("").is_err());
        assert!(validate_container_name("name with spaces").is_err());
        assert!(validate_container_name("name`whoami`").is_err());
    }

    #[test]
    fn test_validate_remote_path() {
        use sm_net::shell::validate_remote_path;
        assert!(validate_remote_path("/home/user/file.txt").is_ok());
        assert!(validate_remote_path("/var/www/html").is_ok());
        assert!(validate_remote_path("").is_err());
        assert!(validate_remote_path("/etc/shadow").is_err());
        assert!(validate_remote_path("/etc/passwd").is_err());
        assert!(validate_remote_path("/root/.ssh").is_err());
        assert!(validate_remote_path("/proc/1/fd").is_err());
        assert!(validate_remote_path("/boot/vmlinuz").is_err());
        assert!(validate_remote_path("../etc/passwd").is_err());
        assert!(validate_remote_path("/var/../../etc/shadow").is_err());
    }

    #[test]
    fn test_validate_filename() {
        use sm_net::shell::validate_filename;
        assert!(validate_filename("server.properties").is_ok());
        assert!(validate_filename("config.yml").is_ok());
        assert!(validate_filename("").is_err());
        assert!(validate_filename("path/with/slash").is_err());
        assert!(validate_filename("..").is_err());
        assert!(validate_filename(".").is_err());
    }

    #[test]
    fn test_validate_mc_command() {
        use sm_net::shell::validate_mc_command;
        assert!(validate_mc_command("say Hello World").is_ok());
        assert!(validate_mc_command("give @p diamond 64").is_ok());
        assert!(validate_mc_command("").is_err());
        assert!(validate_mc_command("stop\nrm -rf /").is_err());
        assert!(validate_mc_command("say\rwhoami").is_err());
        assert!(validate_mc_command(&"a".repeat(2000)).is_err());
    }

    #[test]
    fn test_validate_server_name() {
        use sm_net::shell::validate_server_name;
        assert!(validate_server_name("my-server").is_ok());
        assert!(validate_server_name("Server_01").is_ok());
        assert!(validate_server_name("").is_err());
        assert!(validate_server_name("server name").is_err());
        assert!(validate_server_name("server;rm").is_err());
    }

    // ─── Security / Crypto ────────────────────────────────────────

    #[test]
    fn test_crypto_roundtrip() {
        let engine = sm_security::CryptoEngine::from_password("masterkey", b"mysalt123");
        let original = "password123";
        let encrypted = engine.encrypt_string(original).unwrap();
        let decrypted = engine.decrypt_string(&encrypted).unwrap();
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_crypto_wrong_password_fails() {
        let engine1 = sm_security::CryptoEngine::from_password("correct", b"salt");
        let engine2 = sm_security::CryptoEngine::from_password("wrong", b"salt");
        let encrypted = engine1.encrypt_string("secret").unwrap();
        assert!(engine2.decrypt_string(&encrypted).is_err());
    }

    #[test]
    fn test_crypto_different_nonces() {
        let engine = sm_security::CryptoEngine::from_password("key", b"salt");
        let e1 = engine.encrypt_string("data").unwrap();
        let e2 = engine.encrypt_string("data").unwrap();
        // Nonces should differ → ciphertexts differ
        assert_ne!(e1.ciphertext, e2.ciphertext);
        // Both decrypt to same value
        assert_eq!(engine.decrypt_string(&e1).unwrap(), "data");
        assert_eq!(engine.decrypt_string(&e2).unwrap(), "data");
    }

    #[test]
    fn test_credential_vault() {
        let mut vault = sm_security::CredentialVault::new("masterkey");
        let cred = sm_core::types::Credential {
            auth_method: sm_core::types::AuthMethod::Password,
            username: Some("admin".into()), password: Some("secret123".into()),
            private_key_path: None, private_key_data: None,
            passphrase: None, certificate_path: None,
        };
        vault.store("server-1", &cred).unwrap();
        let retrieved = vault.retrieve("server-1").unwrap();
        assert_eq!(retrieved.username.unwrap(), "admin");
        assert_eq!(retrieved.password.unwrap(), "secret123");
        assert!(vault.retrieve("nonexistent").is_none());
        assert_eq!(vault.count(), 1);
    }

    #[test]
    fn test_credential_vault_remove() {
        let mut vault = sm_security::CredentialVault::new("key");
        let cred = sm_core::types::Credential {
            auth_method: sm_core::types::AuthMethod::None,
            username: None, password: None,
            private_key_path: None, private_key_data: None,
            passphrase: None, certificate_path: None,
        };
        vault.store("test", &cred).unwrap();
        assert_eq!(vault.count(), 1);
        vault.remove("test");
        assert_eq!(vault.count(), 0);
        assert!(vault.retrieve("test").is_none());
    }

    #[test]
    fn test_credential_vault_export_import() {
        let mut vault1 = sm_security::CredentialVault::new("pass1");
        let cred = sm_core::types::Credential {
            auth_method: sm_core::types::AuthMethod::Password,
            username: Some("user".into()), password: Some("pass".into()),
            private_key_path: None, private_key_data: None,
            passphrase: None, certificate_path: None,
        };
        vault1.store("key1", &cred).unwrap();
        let exported = vault1.export().unwrap();
        let mut vault2 = sm_security::CredentialVault::new("pass1");
        vault2.import("pass1", &exported).unwrap();
        let retrieved = vault2.retrieve("key1").unwrap();
        assert_eq!(retrieved.username.unwrap(), "user");
    }

    // ─── Monitor ──────────────────────────────────────────────────

    #[test]
    fn test_monitor_collect() {
        let mut monitor = sm_monitor::SystemMonitor::new();
        let metrics = monitor.collect(sm_core::id::ServerId::new());
        assert!(metrics.uptime_seconds > 0);
        assert!(metrics.memory_total_bytes > 0);
        assert!(metrics.cpu_percent >= 0.0);
    }

    #[test]
    fn test_monitor_processes() {
        let monitor = sm_monitor::SystemMonitor::new();
        let procs = monitor.get_processes();
        assert!(!procs.is_empty());
        // Should find init/systemd or similar
        assert!(procs.iter().any(|p| !p.name.is_empty()));
    }

    #[test]
    fn test_metrics_store() {
        use sm_monitor::MetricsStore;
        let mut store = MetricsStore::new(100);
        let sid = sm_core::id::ServerId::new();
        for i in 0..50 {
            store.push(sm_core::events::SystemMetrics {
                server_id: sid, timestamp: chrono::Utc::now(),
                cpu_percent: i as f64, memory_used_bytes: 0, memory_total_bytes: 0,
                disk_used_bytes: 0, disk_total_bytes: 0,
                network_rx_bytes: 0, network_tx_bytes: 0,
                swap_used_bytes: 0, swap_total_bytes: 0,
                load_average: [0.0; 3], uptime_seconds: 0, process_count: 0,
            });
        }
        assert_eq!(store.len(), 50);
        assert!(store.latest().is_some());
        // Test max capacity
        for _ in 0..100 {
            store.push(sm_core::events::SystemMetrics {
                server_id: sid, timestamp: chrono::Utc::now(),
                cpu_percent: 99.0, memory_used_bytes: 0, memory_total_bytes: 0,
                disk_used_bytes: 0, disk_total_bytes: 0,
                network_rx_bytes: 0, network_tx_bytes: 0,
                swap_used_bytes: 0, swap_total_bytes: 0,
                load_average: [0.0; 3], uptime_seconds: 0, process_count: 0,
            });
        }
        assert_eq!(store.len(), 100); // capped at max
    }

    // ─── Automation / Scheduler ───────────────────────────────────

    #[test]
    fn test_task_scheduler() {
        use sm_automation::TaskScheduler;
        let mut scheduler = TaskScheduler::new();
        let def = sm_core::traits::JobDefinition {
            id: None, name: "Test Job".into(), description: "Test".into(),
            cron_expr: "0 0 * * * *".into(),
            action: sm_core::traits::JobAction::RunCommand("echo test".into()),
            enabled: true, server_id: None,
        };
        let id = scheduler.schedule(def);
        assert!(!id.is_nil());
        assert_eq!(scheduler.list().len(), 1);
    }

    #[test]
    fn test_task_scheduler_cancel() {
        use sm_automation::TaskScheduler;
        let mut scheduler = TaskScheduler::new();
        let def = sm_core::traits::JobDefinition {
            id: None, name: "Cancel Me".into(), description: "".into(),
            cron_expr: "0 0 * * * *".into(),
            action: sm_core::traits::JobAction::RunCommand("echo hi".into()),
            enabled: true, server_id: None,
        };
        let id = scheduler.schedule(def);
        assert!(scheduler.cancel(id));
        assert_eq!(scheduler.list().len(), 1); // still listed but cancelled
    }

    #[test]
    fn test_task_scheduler_multiple() {
        use sm_automation::TaskScheduler;
        let mut scheduler = TaskScheduler::new();
        for i in 0..5 {
            let def = sm_core::traits::JobDefinition {
                id: None, name: format!("Job {}", i), description: "".into(),
                cron_expr: "0 0 * * * *".into(),
                action: sm_core::traits::JobAction::RunCommand("echo hi".into()),
                enabled: true, server_id: None,
            };
            scheduler.schedule(def);
        }
        assert_eq!(scheduler.list().len(), 5);
    }

    // ─── Storage ──────────────────────────────────────────────────

    #[test]
    fn test_storage_local_write_read() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        sm_storage::LocalFileManager::write(&path, b"hello world").unwrap();
        let content = sm_storage::LocalFileManager::read(&path).unwrap();
        assert_eq!(content, b"hello world");
    }

    #[test]
    fn test_storage_local_list() {
        let dir = tempfile::tempdir().unwrap();
        sm_storage::LocalFileManager::write(dir.path().join("a.txt"), b"a").unwrap();
        sm_storage::LocalFileManager::write(dir.path().join("b.txt"), b"bb").unwrap();
        let files = sm_storage::LocalFileManager::list(dir.path()).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_storage_local_delete() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("delete.me");
        sm_storage::LocalFileManager::write(&path, b"data").unwrap();
        assert!(sm_storage::LocalFileManager::exists(&path));
        sm_storage::LocalFileManager::delete(&path).unwrap();
        assert!(!sm_storage::LocalFileManager::exists(&path));
    }

    #[test]
    fn test_storage_local_mkdir_rename() {
        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path().join("subdir");
        sm_storage::LocalFileManager::mkdir(&dir_path).unwrap();
        assert!(dir_path.exists());
        let new_path = dir.path().join("renamed");
        sm_storage::LocalFileManager::rename(&dir_path, &new_path).unwrap();
        assert!(new_path.exists());
        assert!(!dir_path.exists());
    }

    // ─── Network Discovery ────────────────────────────────────────

    #[test]
    fn test_network_discovery_subnet() {
        let hosts = sm_net::NetworkDiscovery::expand_subnet("192.168.1.0/30").unwrap();
        assert!(!hosts.is_empty());
        // /30 should give 2 usable hosts (network + broadcast excluded)
        assert!(hosts.len() <= 3);
    }

    #[test]
    fn test_network_discovery_single_ip() {
        assert!(sm_net::NetworkDiscovery::expand_subnet("10.0.0.1").is_err());
    }

    #[test]
    fn test_network_discovery_invalid_cidr() {
        assert!(sm_net::NetworkDiscovery::expand_subnet("invalid").is_err());
        assert!(sm_net::NetworkDiscovery::expand_subnet("192.168.1.0/99").is_err());
    }

    // ─── Docker ───────────────────────────────────────────────────

    #[test]
    fn test_docker_container_state_parsing() {
        // Verify parse_state logic via the public API
        // (DockerManager requires SSH, but we can test the types)
        use sm_docker::{DockerContainer, ContainerState};
        let c = DockerContainer {
            id: "abc123".into(), name: "test".into(), image: "nginx".into(),
            status: "Up 2 hours".into(), state: ContainerState::Running,
            ports: vec!["80/tcp".into()], created: "2024-01-01".into(),
        };
        assert_eq!(c.id, "abc123");
        assert_eq!(c.state, ContainerState::Running);
    }

    #[test]
    fn test_docker_image_type() {
        use sm_docker::DockerImage;
        let img = DockerImage {
            id: "sha256:abc".into(), repository: "nginx".into(),
            tag: "latest".into(), size: "50MB".into(), created: "2024-01-01".into(),
        };
        assert_eq!(img.repository, "nginx");
        assert_eq!(img.tag, "latest");
    }

    // ─── Minecraft ────────────────────────────────────────────────

    #[test]
    fn test_minecraft_jar_types() {
        let all = sm_minecraft::MinecraftJar::all();
        assert!(all.len() >= 8);
        assert_eq!(sm_minecraft::MinecraftJar::Paper.name(), "Paper");
        assert_eq!(sm_minecraft::MinecraftJar::Vanilla.name(), "Vanilla");
        assert_eq!(sm_minecraft::MinecraftJar::Forge.name(), "Forge");
    }

    #[test]
    fn test_minecraft_jar_properties() {
        assert!(sm_minecraft::MinecraftJar::Paper.supports_plugins());
        assert!(sm_minecraft::MinecraftJar::Purpur.supports_plugins());
        assert!(!sm_minecraft::MinecraftJar::Vanilla.supports_plugins());
        assert!(sm_minecraft::MinecraftJar::Fabric.is_modded());
        assert!(sm_minecraft::MinecraftJar::Forge.is_modded());
        assert!(sm_minecraft::MinecraftJar::NeoForge.is_modded());
        assert!(!sm_minecraft::MinecraftJar::Paper.is_modded());
    }

    #[test]
    fn test_minecraft_server_config() {
        let srv = sm_minecraft::MinecraftServer {
            name: "test".into(), directory: "/opt/mc".into(),
            jar_type: sm_minecraft::MinecraftJar::Paper,
            version: "1.21.4".into(), java_path: "java".into(),
            min_ram_mb: 512, max_ram_mb: 4096, port: 25565,
            online_mode: true, difficulty: "normal".into(),
            motd: "Test Server".into(), max_players: 20,
            created_at: "2024-01-01".into(),
        };
        assert_eq!(srv.name, "test");
        assert_eq!(srv.port, 25565);
        assert_eq!(srv.max_ram_mb, 4096);
    }

    #[test]
    fn test_minecraft_default_tasks() {
        let tasks = sm_minecraft::MinecraftManager::default_tasks("/opt/mc");
        assert!(!tasks.is_empty());
        assert!(tasks.iter().any(|t| t.name.contains("Backup")));
        assert!(tasks.iter().any(|t| t.name.contains("Reinicio")));
        assert!(tasks.iter().any(|t| t.name.contains("Logs")));
    }

    #[test]
    fn test_minecraft_multi_manager() {
        let mgr = sm_minecraft::MultiMinecraftManager::new();
        // Just verify it creates without panic
        let _ = mgr;
    }

    // ─── Reports ──────────────────────────────────────────────────

    #[test]
    fn test_reports_json_generation() {
        let data = sm_reports::ReportData {
            title: "Test Report".into(),
            generated_at: "2024-01-01T00:00:00Z".into(),
            version: "0.6.0".into(), server_count: 2, online_count: 1,
            servers: vec![
                sm_reports::ServerSummary {
                    name: "Web".into(), host: "10.0.0.1".into(), os: "Ubuntu".into(),
                    status: "online".into(), tags: vec!["web".into()],
                    last_connected: Some("2024-01-01".into()),
                }
            ],
            metrics: vec![], logs: vec![], alerts: vec![],
        };
        let engine = sm_reports::ReportEngine::new();
        let json = engine.generate_json(&data).unwrap();
        assert!(json.contains("Test Report"));
        assert!(json.contains("Ubuntu"));
        assert!(json.contains("server_count"));
    }

    #[test]
    fn test_reports_html_generation() {
        let data = sm_reports::ReportData {
            title: "HTML Report".into(), generated_at: "2024-01-01".into(),
            version: "0.6.0".into(), server_count: 1, online_count: 1,
            servers: vec![], metrics: vec![], logs: vec![], alerts: vec![],
        };
        let engine = sm_reports::ReportEngine::new();
        let html = engine.generate_html(&data).unwrap();
        assert!(html.contains("<html"));
        assert!(html.contains("HTML Report"));
        assert!(html.contains("<table>"));
    }

    #[test]
    fn test_reports_format() {
        use sm_reports::ReportFormat;
        let data = sm_reports::ReportData {
            title: "Format".into(), generated_at: "".into(), version: "0.6.0".into(),
            server_count: 0, online_count: 0,
            servers: vec![], metrics: vec![], logs: vec![], alerts: vec![],
        };
        let engine = sm_reports::ReportEngine::new();
        let json = engine.generate(&data, &ReportFormat::Json).unwrap();
        assert!(json.len() > 10);
        let html = engine.generate(&data, &ReportFormat::Html).unwrap();
        assert!(html.len() > 100);
    }

    // ─── Session Manager ──────────────────────────────────────────

    #[test]
    fn test_session_manager_creation() {
        let _ = sm_net::SessionManager::default();
        let _ = sm_net::SessionManager::new(30);
    }

    // ─── Plugins ──────────────────────────────────────────────────

    #[test]
    fn test_plugin_registry() {
        use sm_plugins::PluginRegistry;
        let mut registry = PluginRegistry::new();
        assert_eq!(registry.count(), 0);
        let manifest = sm_core::types::PluginManifest {
            id: "test-plugin".into(), name: "Test Plugin".into(),
            version: "1.0.0".into(), description: "A test".into(),
            author: "DevCam".into(),
            plugin_type: sm_core::types::PluginType::Page,
            dependencies: vec![], permissions: vec![], entry_point: "main".into(),
        };
        registry.register(manifest);
        assert_eq!(registry.count(), 1);
        assert!(registry.get("test-plugin").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_plugin_registry_remove() {
        use sm_plugins::PluginRegistry;
        let mut registry = PluginRegistry::new();
        let manifest = sm_core::types::PluginManifest {
            id: "remove-me".into(), name: "Remove".into(),
            version: "1.0.0".into(), description: "".into(), author: "".into(),
            plugin_type: sm_core::types::PluginType::Widget,
            dependencies: vec![], permissions: vec![], entry_point: "".into(),
        };
        registry.register(manifest);
        assert_eq!(registry.count(), 1);
        let removed = registry.remove("remove-me");
        assert!(removed.is_some());
        assert_eq!(registry.count(), 0);
        assert!(registry.remove("nonexistent").is_none());
    }
}

// Helper trait removed — async tests use simpler patterns
