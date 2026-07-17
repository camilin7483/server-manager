use rusqlite::Connection;
use std::path::{Path, PathBuf};
use tracing::info;

use super::migrations;

pub struct DatabaseConfig {
    pub path: PathBuf,
    pub wal_mode: bool,
    pub busy_timeout_ms: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("server-manager.db"),
            wal_mode: true,
            busy_timeout_ms: 5000,
        }
    }
}

pub struct Database {
    connection: Connection,
    pub path: PathBuf,
}

impl Database {
    pub fn open(config: DatabaseConfig) -> Result<Self, Box<dyn std::error::Error>> {
        if let Some(parent) = config.path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&config.path)?;

        if config.wal_mode {
            conn.pragma_update(None, "journal_mode", "WAL")?;
        }
        conn.pragma_update(None, "busy_timeout", config.busy_timeout_ms as i64)?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        info!("Base de datos abierta en {}", config.path.display());

        let mut db = Self {
            connection: conn,
            path: config.path,
        };

        db.run_migrations()?;

        Ok(db)
    }

    pub fn open_at(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        Self::open(DatabaseConfig {
            path: path.as_ref().to_path_buf(),
            ..Default::default()
        })
    }

    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    pub fn connection_mut(&mut self) -> &mut Connection {
        &mut self.connection
    }

    fn run_migrations(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        migrations::run(&self.connection)?;
        Ok(())
    }

    pub fn schema_version(&self) -> Result<u32, Box<dyn std::error::Error>> {
        let version: u32 = self
            .connection
            .pragma_query_value(None, "user_version", |row| row.get(0))?;
        Ok(version)
    }

    pub fn vacuum(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.connection.execute_batch("VACUUM")?;
        Ok(())
    }

    pub fn close(self) -> Result<(), (Connection, rusqlite::Error)> {
        self.connection.close()
    }
}
