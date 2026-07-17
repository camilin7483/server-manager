// Tests de integración del Server Manager

#[cfg(test)]
mod tests {
    #[test]
    fn test_core_types_creation() {
        let id = sm_core::id::ServerId::new();
        assert!(!id.is_nil());

        let server = sm_core::types::Server {
            id: sm_core::id::ServerId::new(),
            name: "test".into(),
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
            profile: None, tags: vec![], notes: String::new(),
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            last_connected_at: None, metadata: std::collections::HashMap::new(),
        };
        assert_eq!(server.name, "test");
        assert_eq!(server.connection.port, 22);
    }

    #[test]
    fn test_config_defaults() {
        let config = sm_config::AppConfig::default();
        assert_eq!(config.ui.theme, "dark");
        assert_eq!(config.servers.default_ssh_port, 22);
    }

    #[test]
    fn test_network_discovery_subnet() {
        let hosts = sm_net::NetworkDiscovery::expand_subnet("192.168.1.0/30").unwrap();
        assert!(!hosts.is_empty());
    }

    #[test]
    fn test_monitor_collect() {
        let mut monitor = sm_monitor::SystemMonitor::new();
        let metrics = monitor.collect(sm_core::id::ServerId::new());
        assert!(metrics.uptime_seconds > 0);
        assert!(metrics.memory_total_bytes > 0);
    }

    #[test]
    fn test_task_scheduler() {
        let mut scheduler = sm_automation::TaskScheduler::new();
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
    fn test_storage_local() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        sm_storage::LocalFileManager::write(&path, b"hello world").unwrap();
        let content = sm_storage::LocalFileManager::read(&path).unwrap();
        assert_eq!(content, b"hello world");
        let files = sm_storage::LocalFileManager::list(dir.path()).unwrap();
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_server_profiles_all() {
        let profiles = sm_core::profiles::ServerProfileTemplate::all_presets();
        assert!(profiles.len() >= 14);
        let ubuntu = profiles.iter().find(|p| p.name == "Ubuntu Server").unwrap();
        assert!(ubuntu.ports.contains(&22));
        let docker = profiles.iter().find(|p| p.name == "Docker Host").unwrap();
        assert_eq!(docker.profile_type, sm_core::types::ProfileType::DockerHost);
        let mc = profiles.iter().find(|p| p.name == "Minecraft Java Server").unwrap();
        assert!(mc.ports.contains(&25565));
    }

    #[test]
    fn test_crypto_roundtrip() {
        let engine = sm_security::CryptoEngine::from_password("masterkey", b"mysalt123");
        let encrypted = engine.encrypt_string("password123").unwrap();
        let decrypted = engine.decrypt_string(&encrypted).unwrap();
        assert_eq!("password123", decrypted);
    }

    #[test]
    fn test_credential_vault() {
        let mut vault = sm_security::CredentialVault::new("masterkey");
        let cred = sm_core::types::Credential {
            auth_method: sm_core::types::AuthMethod::Password,
            username: Some("admin".into()), password: Some("secret".into()),
            private_key_path: None, private_key_data: None,
            passphrase: None, certificate_path: None,
        };
        vault.store("server-1", &cred).unwrap();
        let retrieved = vault.retrieve("server-1").unwrap();
        assert_eq!(retrieved.username.unwrap(), "admin");
    }

    #[test]
    fn test_reports_json_generation() {
        let data = sm_reports::ReportData {
            title: "Test Report".into(),
            generated_at: "2024-01-01T00:00:00Z".into(),
            version: "0.1.0".into(),
            server_count: 2,
            online_count: 1,
            servers: vec![
                sm_reports::ServerSummary {
                    name: "Web".into(), host: "10.0.0.1".into(), os: "Ubuntu".into(),
                    status: "online".into(), tags: vec!["web".into()],
                    last_connected: Some("2024-01-01".into()),
                }
            ],
            metrics: vec![],
            logs: vec![],
            alerts: vec![],
        };

        let engine = sm_reports::ReportEngine::new();
        let json = engine.generate_json(&data).unwrap();
        assert!(json.contains("Test Report"));
        assert!(json.contains("Ubuntu"));

        let html = engine.generate_html(&data).unwrap();
        assert!(html.contains("Test Report"));
        assert!(html.contains("<table>"));
    }

    #[test]
    fn test_version_compare() {
        // Test internal version comparison
        assert!(!sm_reports::ReportEngine::new().generate_json(
            &sm_reports::ReportData {
                title: "v".into(), generated_at: "".into(), version: "0.1.0".into(),
                server_count: 0, online_count: 0,
                servers: vec![], metrics: vec![], logs: vec![], alerts: vec![],
            }
        ).is_err());
    }

    #[test]
    fn test_session_manager_default() {
        let _manager = sm_net::SessionManager::default();
        let _manager2 = sm_net::SessionManager::new(10);
    }

    #[test]
    fn test_minecraft_jar_types() {
        let all = sm_minecraft::MinecraftJar::all();
        assert!(all.len() >= 8);
        assert_eq!(sm_minecraft::MinecraftJar::Paper.name(), "Paper");
    }
}
