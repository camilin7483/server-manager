use rusqlite::{params, Connection};
use tracing::debug;

use super::models::SmServer;

pub struct ServerRepository<'a> {
    conn: &'a Connection,
}

impl<'a> ServerRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn insert(&self, server: &SmServer) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "INSERT INTO servers (id, name, group_id, host, port, protocol, username,
             auth_method, credential_data, status, os_type, os_flavor, profile_type,
             tags, notes, metadata, created_at, updated_at, last_connected_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
             ?15, ?16, ?17, ?18, ?19)",
            params![
                server.id,
                server.name,
                server.group_id,
                server.host,
                server.port,
                server.protocol,
                server.username,
                server.auth_method,
                server.credential_data,
                server.status,
                server.os_type,
                server.os_flavor,
                server.profile_type,
                server.tags,
                server.notes,
                server.metadata,
                server.created_at,
                server.updated_at,
                server.last_connected_at,
            ],
        )?;
        Ok(())
    }

    pub fn update(&self, server: &SmServer) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE servers SET name=?1, group_id=?2, host=?3, port=?4, protocol=?5,
             username=?6, auth_method=?7, credential_data=?8, status=?9,
             os_type=?10, os_flavor=?11, profile_type=?12, tags=?13, notes=?14,
             metadata=?15, updated_at=?16, last_connected_at=?17
             WHERE id=?18",
            params![
                server.name,
                server.group_id,
                server.host,
                server.port,
                server.protocol,
                server.username,
                server.auth_method,
                server.credential_data,
                server.status,
                server.os_type,
                server.os_flavor,
                server.profile_type,
                server.tags,
                server.notes,
                server.metadata,
                server.updated_at,
                server.last_connected_at,
                server.id,
            ],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<(), rusqlite::Error> {
        self.conn
            .execute("DELETE FROM servers WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<SmServer>, rusqlite::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, group_id, host, port, protocol, username,
                      auth_method, credential_data, status, os_type, os_flavor,
                      profile_type, tags, notes, metadata, created_at, updated_at,
                      last_connected_at FROM servers ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            Ok(SmServer {
                id: row.get(0)?,
                name: row.get(1)?,
                group_id: row.get(2)?,
                host: row.get(3)?,
                port: row.get::<_, i64>(4)? as u16,
                protocol: row.get(5)?,
                username: row.get(6)?,
                auth_method: row.get(7)?,
                credential_data: row.get(8)?,
                status: row.get(9)?,
                os_type: row.get(10)?,
                os_flavor: row.get(11)?,
                profile_type: row.get(12)?,
                tags: row.get(13)?,
                notes: row.get(14).unwrap_or_default(),
                metadata: row.get(15)?,
                created_at: row.get(16)?,
                updated_at: row.get(17)?,
                last_connected_at: row.get(18)?,
            })
        })?;
        let mut servers = Vec::new();
        for server in rows {
            servers.push(server?);
        }
        debug!("Cargados {} servidores de la base de datos", servers.len());
        Ok(servers)
    }

    pub fn get(&self, id: &str) -> Result<Option<SmServer>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, group_id, host, port, protocol, username,
             auth_method, credential_data, status, os_type, os_flavor,
             profile_type, tags, notes, metadata, created_at, updated_at,
             last_connected_at FROM servers WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(SmServer {
                id: row.get(0)?,
                name: row.get(1)?,
                group_id: row.get(2)?,
                host: row.get(3)?,
                port: row.get::<_, i64>(4)? as u16,
                protocol: row.get(5)?,
                username: row.get(6)?,
                auth_method: row.get(7)?,
                credential_data: row.get(8)?,
                status: row.get(9)?,
                os_type: row.get(10)?,
                os_flavor: row.get(11)?,
                profile_type: row.get(12)?,
                tags: row.get(13)?,
                notes: row.get(14).unwrap_or_default(),
                metadata: row.get(15)?,
                created_at: row.get(16)?,
                updated_at: row.get(17)?,
                last_connected_at: row.get(18)?,
            })
        })?;
        match rows.next() {
            Some(Ok(server)) => Ok(Some(server)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }
}
