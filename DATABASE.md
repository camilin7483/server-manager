# Base de Datos

## Motor

SQLite 3.x con rusqlite. Modo WAL activado por defecto.

## Ubicación

`~/.local/share/com.server-manager.server-manager/server-manager.db`

## Migraciones

Las migraciones se ejecutan automáticamente al abrir la base de datos. El esquema se versiona mediante `PRAGMA user_version`.

### Versiones

| Versión | Descripción |
|---|---|
| 1 | Esquema inicial: servers, connections, logs, metrics, jobs, job_runs, ssh_keys, plugins, audit_log |
| 2 | Índices FTS5 para búsqueda |

## Esquema

### servers
| Columna | Tipo | Descripción |
|---|---|---|
| id | TEXT PK | UUID del servidor |
| name | TEXT | Nombre descriptivo |
| group_id | TEXT | Grupo opcional |
| host | TEXT | Dirección IP/hostname |
| port | INTEGER | Puerto de conexión |
| protocol | TEXT | SSH, RDP, HTTP, etc. |
| username | TEXT | Usuario de conexión |
| auth_method | TEXT | Password, Key, Agent |
| credential_data | TEXT | Credencial cifrada (JSON) |
| status | TEXT | Online, Offline, Error |
| profile_type | TEXT | Perfil aplicado |

### metrics
| Columna | Descripción |
|---|---|
| server_id | FK a servers |
| timestamp | Momento de la muestra |
| cpu_percent | Uso de CPU |
| memory_used_bytes | RAM usada |
| disk_used_bytes | Disco usado |
| network_rx_bytes | Red recibida |
| network_tx_bytes | Red transmitida |

### jobs
| Columna | Descripción |
|---|---|
| id | UUID |
| name | Nombre de la tarea |
| cron_expr | Expresión cron |
| action_type | RunCommand, BackupServer, etc. |
| enabled | Booleano |

### audit_log
Registro de auditoría para todas las operaciones sensibles.
