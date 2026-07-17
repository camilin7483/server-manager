use rusqlite::Connection;
use tracing::info;

const MIGRATIONS: &[(&str, &str)] = &[
    (
        "001_init",
        r#"
        CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS servers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            group_id TEXT,
            host TEXT NOT NULL,
            port INTEGER NOT NULL DEFAULT 22,
            protocol TEXT NOT NULL DEFAULT 'Ssh',
            username TEXT,
            auth_method TEXT NOT NULL DEFAULT 'Password',
            credential_data TEXT,
            status TEXT NOT NULL DEFAULT 'Offline',
            os_type TEXT,
            os_flavor TEXT,
            profile_type TEXT,
            tags TEXT,
            notes TEXT DEFAULT '',
            metadata TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            last_connected_at TEXT
        );

        CREATE TABLE IF NOT EXISTS connections (
            id TEXT PRIMARY KEY,
            server_id TEXT NOT NULL,
            protocol TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'Disconnected',
            started_at TEXT NOT NULL DEFAULT (datetime('now')),
            ended_at TEXT,
            bytes_sent INTEGER DEFAULT 0,
            bytes_received INTEGER DEFAULT 0,
            FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            level TEXT NOT NULL DEFAULT 'Info',
            message TEXT NOT NULL,
            source TEXT NOT NULL DEFAULT 'system',
            server_id TEXT,
            FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS metrics (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            server_id TEXT NOT NULL,
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            cpu_percent REAL NOT NULL DEFAULT 0,
            memory_used_bytes INTEGER NOT NULL DEFAULT 0,
            memory_total_bytes INTEGER NOT NULL DEFAULT 0,
            disk_used_bytes INTEGER NOT NULL DEFAULT 0,
            disk_total_bytes INTEGER NOT NULL DEFAULT 0,
            network_rx_bytes INTEGER NOT NULL DEFAULT 0,
            network_tx_bytes INTEGER NOT NULL DEFAULT 0,
            swap_used_bytes INTEGER NOT NULL DEFAULT 0,
            swap_total_bytes INTEGER NOT NULL DEFAULT 0,
            load_1m REAL DEFAULT 0,
            load_5m REAL DEFAULT 0,
            load_15m REAL DEFAULT 0,
            uptime_seconds INTEGER DEFAULT 0,
            process_count INTEGER DEFAULT 0,
            FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS jobs (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT DEFAULT '',
            cron_expr TEXT NOT NULL,
            action_type TEXT NOT NULL,
            action_data TEXT,
            server_id TEXT,
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS job_runs (
            id TEXT PRIMARY KEY,
            job_id TEXT NOT NULL,
            started_at TEXT NOT NULL DEFAULT (datetime('now')),
            finished_at TEXT,
            status TEXT NOT NULL DEFAULT 'Running',
            output TEXT,
            exit_code INTEGER,
            FOREIGN KEY (job_id) REFERENCES jobs(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS ssh_keys (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            key_type TEXT NOT NULL,
            public_key TEXT NOT NULL,
            private_key_path TEXT NOT NULL,
            bits INTEGER NOT NULL DEFAULT 256,
            comment TEXT DEFAULT '',
            fingerprint TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS plugins (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            version TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            manifest_data TEXT,
            loaded_at TEXT
        );

        CREATE TABLE IF NOT EXISTS audit_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            actor TEXT NOT NULL DEFAULT 'system',
            action TEXT NOT NULL,
            resource TEXT NOT NULL,
            details TEXT,
            ip_address TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_metrics_server_time ON metrics(server_id, timestamp);
        CREATE INDEX IF NOT EXISTS idx_logs_server_time ON logs(server_id, timestamp);
        CREATE INDEX IF NOT EXISTS idx_job_runs_job ON job_runs(job_id);
        "#,
    ),
    (
        "002_tags_fts",
        r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS servers_fts USING fts5(name, host, tags, notes, content='servers', content_rowid='rowid');
        "#,
    ),
];

pub fn run(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let current_version: u32 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap_or(0);

    for (idx, (name, sql)) in MIGRATIONS.iter().enumerate() {
        let version = idx as u32 + 1;

        if current_version >= version {
            continue;
        }

        info!("Aplicando migración {}: {}", version, name);
        conn.execute_batch(sql)?;
        conn.pragma_update(None, "user_version", version)?;
    }

    Ok(())
}
