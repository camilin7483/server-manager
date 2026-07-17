# Server Manager

**Plataforma profesional de administracion de infraestructura** — SSH, Docker, Minecraft, monitoreo y mas en una sola app.

Desarrollado por **DevCam**

---

## Que es

Server Manager es una aplicacion de escritorio escrita en Rust que permite administrar decenas de servidores desde una unica interfaz grafica. Combina funcionalidades de Termius, Portainer, Cockpit y Docker Desktop en un solo panel unificado.

## Caracteristicas principales

### Gestor de servidores
- Crear, editar y eliminar servidores
- Conexiones SSH reales con autenticacion por password y claves
- Grupos visuales con colores (Produccion, Desarrollo, etc.)
- Tags personalizables (#web, #db, #production)
- Busqueda por nombre, host o tags

### Terminal ANSI
- Emulador de terminal completo con colores ANSI (16, 256, true color)
- Multisesion con pestañas (tabs)
- Historial de comandos (flechas arriba/abajo)
- Scrollback buffer de 10,000 lineas
- Cursor parpadeante
- Soporte para secuencias CSI/SGR

### Explorador de archivos
- Tree view con iconos por tipo de archivo
- Expandir/colapsar directorios
- Menu contextual (copiar, mover, eliminar, descargar)
- Editor de texto integrado multi-tab
- Breadcrumbs de navegacion

### Docker
- Listar contenedores, imagenes, volumenes y redes
- Iniciar, detener, reiniciar y eliminar contenedores
- Docker Compose (up/down/logs)
- Estadisticas de uso de recursos
- Prune del sistema

### Minecraft
- Crear servidores: Paper, Purpur, Vanilla, Fabric, Forge, NeoForge, Spigot, Velocity, BungeeCord
- Descarga automatica del JAR desde las APIs oficiales
- Aceptar EULA automaticamente
- Generar server.properties
- Iniciar, detener, reiniciar, matar proceso
- Consola en tiempo real via RCON o screen
- Instalar plugins desde Modrinth
- Buscar, instalar, actualizar y eliminar plugins
- Backups manuales y programados
- Deteccion de IP local y publica
- Verificacion de puerto abierto
- Monitoreo: CPU, RAM, TPS, jugadores, uptime

### Monitoreo
- Graficas en tiempo real: CPU, RAM, disco, red
- Sparklines con area rellena
- 60 puntos de historial
- Lista de procesos con CPU y memoria

### Diagnostico de red
- Ping
- Traceroute
- Port Scan
- DNS Lookup (A, AAAA, MX, TXT, NS, CNAME)
- WHOIS
- Latencia

### Seguridad
- Cifrado AES-256-GCM para credenciales
- Credential vault con export/import cifrado
- Zeroization de datos sensibles
- Escaneo de seguridad (firewall, puertos, SSL, usuarios, sesiones)
- Logs de auditoria
- Validacion anti path-traversal
- Sanitizacion anti command-injection

### Tareas programadas
- Crear tareas con expresiones cron
- Acciones: backup, reinicio, actualizar paquetes, limpiar logs, comandos personalizados
- Activar/desactivar tareas
- Tabla con nombre, cron, accion y estado

### Plugins
- Buscar en Modrinth (20+ plugins en pool)
- Instalar y eliminar plugins
- Lista de plugins instalados
- Sistema extensible de plugins (Page, Widget, Driver, Monitor, Automation, Theme)

### Reportes
- Generacion de reportes en HTML (con template Handlebars y tema oscuro)
- Exportacion JSON
- Estructura para PDF
- Datos: inventario de servidores, metricas, logs, alertas

### Interfaz
- Tema oscuro y claro (tambien Alto Contraste y Solarized)
- Sidebar con iconos y barra de acento
- Notificaciones toast (Info, Success, Warning, Error) con fade-out
- Barra de estado con conexiones activas, sesiones, FPS
- 23 atajos de teclado globales
- **Sonidos tipo Minecraft al clickear botones** (sintetizados en runtime)

## Stack tecnologico

| Componente | Tecnologia |
|---|---|
| Lenguaje | Rust 2021 |
| UI | egui 0.30 / eframe |
| Async | Tokio |
| SSH | Russh 0.44 (Rust nativo, sin C deps) |
| Base de datos | SQLite (rusqlite) |
| Configuracion | TOML |
| Cifrado | AES-256-GCM, Argon2 |
| Sonidos | rodio (sintesis WAV en runtime) |
| HTTP | reqwest |
| Logging | tracing |

## Arquitectura

14 crates modulares con separacion estricta de responsabilidades:

```
server-manager/
├── crates/
│   ├── core/         # Tipos, traits, errores, eventos, IDs, perfiles
│   ├── config/       # Config TOML + hot-reload con notify
│   ├── db/           # SQLite + migraciones versionadas
│   ├── net/          # SSH, SFTP, discovery, keys, shell safety
│   ├── security/     # AES-256-GCM + credential vault + zeroization
│   ├── monitor/      # CPU/RAM/disco/red (sysinfo)
│   ├── plugins/      # Motor + registro de plugins
│   ├── storage/      # Gestor de archivos locales
│   ├── automation/   # Tareas programadas con cron
│   ├── docker/       # Gestion Docker remota via SSH
│   ├── minecraft/    # Minecraft + RCON + multi-server + log streaming
│   ├── reports/      # Reportes HTML/JSON/PDF (Handlebars)
│   ├── ui/           # GUI egui + AppContext + terminal ANSI + sonidos
│   └── updater/      # Auto-updater via GitHub Releases API
├── src/main.rs       # Entry point
├── tests/            # 13 tests de integracion
└── docs/             # Documentacion completa
```

### Flujo de datos

```
Usuario → UI (egui) → AppContext → SSH/Docker/Minecraft/...
                              ↓
                      EventBus ← Eventos ← Resultados
```

### Seguridad

Todas las operaciones que construyen comandos shell pasan por `sm_net::shell`:

- `shell_quote()` — escapa argumentos con comillas simples
- `validate_container_id()` — solo hex, max 64 chars
- `validate_remote_path()` — bloquea path traversal (..) y paths peligrosos (/etc/shadow, /proc)
- `validate_mc_command()` — bloquea caracteres de control
- `validate_server_name()` — solo alfanumerico, guiones y guiones bajos

## Instalacion

### Desde codigo fuente

```bash
git clone https://github.com/camilin7483/server-manager.git
cd server-manager
cargo build --release
./target/release/server-manager
```

### Debug (rapido)

```bash
cargo run
```

### Requisitos

- Rust 1.80+ (edition 2021)
- Linux: libxcb, libxkbcommon, wayland, fontconfig
- Audio: ALSA o PulseAudio (para sonidos)

#### Arch Linux
```bash
sudo pacman -S libxcb libxkbcommon wayland fontconfig
```

#### Ubuntu/Debian
```bash
sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libfontconfig-dev
```

## Uso

1. **Abrir la app** — busca "Server Manager" en tu menu de aplicaciones o ejecuta el binario
2. **Anadir servidor** — click "+ Anadir" en el panel izquierdo, completa host, puerto, usuario y password
3. **Conectar** — selecciona el servidor y click "Conectar"
4. **Terminal** — usa la consola ANSI para ejecutar comandos
5. **Minecraft** — tab "Minecraft" para crear y administrar servidores
6. **Docker** — tab "Docker" para gestionar contenedores
7. **Monitoreo** — tab "Monitor" para ver CPU/RAM/disco/red en tiempo real

## Atajos de teclado

| Atajo | Accion |
|---|---|
| Ctrl+T | Nueva pestaña terminal |
| Ctrl+W | Cerrar pestaña |
| Ctrl+S | Guardar |
| Ctrl+F | Buscar |
| Ctrl+L | Foco en input de comando |
| Ctrl+K | Limpiar terminal |
| Ctrl+Enter | Conectar |
| Ctrl+D | Desconectar |
| Ctrl+Q | Salir |
| F5 | Refrescar conexion |
| Ctrl+C/V/X | Copiar/Pegar/Cortar |
| Ctrl+Z | Deshacer |
| Ctrl+Plus/Minus | Zoom in/out |

## Tests

```bash
cargo test
```

13 tests de integracion cubren: core types, config, network discovery, monitor, task scheduler, storage, server profiles, crypto, credential vault, reports, minecraft.

## Licencia

MIT — Desarrollado por **DevCam**
