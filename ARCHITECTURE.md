# Arquitectura

Server Manager usa una arquitectura de **workspace modular** con 11 crates independientes.

## Principios

1. **Separación estricta** — UI, core, networking, storage, plugins, monitor, etc. son crates separados.
2. **Traits como contratos** — las interfaces entre módulos se definen mediante traits en `sm-core`.
3. **Event-driven** — comunicación desacoplada mediante un bus de eventos.
4. **Configuración externa** — todo parametrizable vía TOML.
5. **Persistencia SQLite** — modelo relacional con migraciones versionadas.
6. **Plugin API** — extensibilidad sin modificar el core.

## Diagrama de dependencias

```
src/main.rs (binary)
  └── sm-ui (GUI)
        ├── sm-core ─────────────── (sin dependencias internas)
        ├── sm-config ──────────── (→ sm-core)
        ├── sm-db ──────────────── (→ sm-core)
        ├── sm-net ─────────────── (→ sm-core)
        ├── sm-security ────────── (→ sm-core)
        ├── sm-monitor ─────────── (→ sm-core)
        ├── sm-plugins ─────────── (→ sm-core)
        ├── sm-storage ─────────── (→ sm-core)
        ├── sm-automation ──────── (→ sm-core, sm-db)
        └── sm-updater ─────────── (→ sm-core, sm-config)
```

## Flujo de datos

```
Usuario → UI (egui) → Traits (core) → Implementaciones (net/db/security)
                                              ↓
EventBus ← Eventos ← Resultados ← ssh/disk/monitor/...
```

## Configuración

Archivo TOML en `~/.config/com.server-manager.server-manager/config.toml`:

```toml
[general]
data_dir = ""
language = "en"

[ui]
theme = "dark"
font_size = 14.0

[servers]
default_ssh_port = 22
connection_timeout_secs = 30

[monitor]
enabled = true
interval_ms = 5000
```

## Base de datos

SQLite con WAL mode activado. Esquema versionado con migraciones incrementales.

Tablas: `servers`, `connections`, `logs`, `metrics`, `jobs`, `job_runs`, `ssh_keys`, `plugins`, `audit_log`.

## Seguridad

- Credenciales cifradas con AES-256-GCM
- Claves derivadas mediante SHA-256 + salt
- Zeroization de datos sensibles en memoria
- Vault de credenciales con export/import cifrado
