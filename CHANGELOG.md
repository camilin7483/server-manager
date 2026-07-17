# Changelog

## [0.6.0] — 2026-07-16

### Arreglado
- Dashboard: texto de metricas ya no se sale de las cards (truncado al ancho)
- Monitor: sparklines redimensionadas con cards de ancho fijo, labels cortos
- Plugins: busqueda ahora filtra por lo que escribe el usuario (20 plugins en pool)
- Server list: boton de eliminar funciona correctamente
- Conexion SSH: mensajes de error claros (timeout, refused)

### Anadido
- Formulario de creacion de servidores Minecraft (nombre, tipo, version, RAM, puerto, Java)
- Formulario de tareas programadas (nombre, cron, accion) con toggle ON/OFF y eliminar
- Panel de seguridad con escaneo real (progress bar + 6 checks + resultados coloreados)
- Panel Docker con lista de contenedores y botones Start/Stop/Remove
- Browser de plugins Modrinth con busqueda filtrada, instalar y eliminar
- Branding "Desarrollado por DevCam" en status bar y settings

## [0.5.0] — 2026-07-16

### Añadido
- **Reports Engine** (`sm-reports`): generación de reportes en HTML (con template Handlebars y tema oscuro), JSON, y estructura para PDF. Reporte de inventario de servidores, métricas, logs y alertas
- **Auto-updater**: cliente completo para GitHub Releases API, comparación semver, detección de assets por SO
- **Network Diagnostics UI**: panel interactivo con Ping, Traceroute, Port Scan, DNS Lookup, WHOIS, y Latency. Selector de tipo, opciones por modo, resultados coloreados con timestamps
- Tests expandidos de 12 a 13 (report generation, estructura JSON/HTML)

### Cambiado
- Workspace: 14 crates (+ sm-reports)
- `sm-updater` ahora usa reqwest para HTTP en vez de stubs
- `network_diag` implementado con UI funcional completa

## [0.4.0] — 2026-07-16

### Añadido
- **File Browser**: explorador de archivos remoto con tree view, breadcrumbs, íconos por tipo de archivo, colapsar/expandir directorios, click derecho (copy/move/download/delete), barra de progreso de operaciones, human-readable sizes
- **File Editor**: editor multi-tab con detección de lenguaje, indicadores de modificación, panel inferior redimensionable, soporte para archivos remotos
- **Keyboard Shortcuts**: sistema global de atajos (Ctrl+T: nueva tab, Ctrl+W: cerrar tab, Ctrl+S: guardar, Ctrl+F: buscar, Ctrl+K: limpiar terminal, F5: refrescar, Ctrl+Q: salir, etc.)
- **Panel inferior de editor**: integrado con tabs de archivos abiertos, se abre al hacer Enter en un archivo del explorador
- **Settings panel**: selector de tema visual (Dark, Light, HighContrast, Solarized), vista de atajos
- Status bar mejorada: contador de archivos abiertos, nombre del tema activo

### Cambiado
- Arquitectura UI reorganizada: FileBrowser, FileEditor, ShortcutManager como módulos independientes
- App unificada con todos los componentes integrados en paneles laterales + central + inferior

## [0.3.0] — 2026-07-16

### Añadido
- **Terminal ANSI**: parser completo de secuencias de escape, grid de celdas con colores (16, 256, true color), scrollback, cursor, SGR, CSI
- **TerminalManager**: multisesión con tabs, historial de comandos (↑/↓ arrows), autocompletado estructural
- **Server Groups**: grupos visuales con colores, collapse/expand, servidores agrupados
- **MonitoringGraphs**: sparklines en tiempo real (CPU, RAM, Red ↓↑), área rellena bajo la línea
- **Notifications**: sistema de toasts con 4 tipos (Info/Success/Warning/Error), auto-dismiss, fade-out
- **Dashboard avanzado**: métricas numéricas + sparklines integradas en el panel de monitoreo
- Barra de estado mejorada: sesiones terminal, conexiones activas, FPS

## [0.2.0] — 2026-07-16

### Añadido
- **SessionManager**: pool de sesiones SSH con keepalive automático (`sm-net`)
- **SshKeyManager**: generación (ssh-keygen), importación, exportación y listado de claves SSH
- **SftpClient**: cliente SFTP vía comandos SSH (list, read, write, delete, mkdir, rename)
- **ServerProfileTemplate**: 14 perfiles predefinidos (Ubuntu, Debian, Arch, Docker, Nginx, Minecraft Java/Bedrock, PostgreSQL, Node.js, Python, WireGuard, Proxmox, OpenMediaVault, Raspberry Pi)
- **DockerManager** (`sm-docker`): gestión remota de contenedores, imágenes, volúmenes, docker compose
- **MinecraftManager** (`sm-minecraft`): soporte para Paper, Purpur, Spigot, Fabric, Forge, NeoForge, Velocity, BungeeCord. Instalación, start/stop, comandos, backups, TPS, whitelist
- **ConnectionBridge**: puente async entre UI egui y operaciones SSH de fondo (tokio runtime)
- **UI mejorada**: botones Conectar/Desconectar con wiring SSH real, secciones Docker y Minecraft en sidebar, barra de estado con conexiones activas

### Cambiado
- `sm-core` ahora re-exporta tipos comunes para acceso más simple
- `sm-net` expandido con 4 módulos nuevos (session_manager, key_manager, sftp, bridge)
- Workspace actualizado a 13 crates (+ docker, + minecraft)
- Tests de integración expandidos de 6 a 12 tests

### Estructura actualizada
```
server-manager/
├── crates/
│   ├── core/         # Tipos, traits, eventos, IDs, perfiles
│   ├── config/       # Configuración TOML + hot-reload
│   ├── db/           # SQLite + migraciones
│   ├── docker/       # Gestión Docker remota
│   ├── minecraft/    # Gestión servidores Minecraft
│   ├── net/          # SSH + SFTP + keys + discovery + session pool
│   ├── security/     # AES-256-GCM + credential vault
│   ├── monitor/      # CPU/RAM/disco/red
│   ├── plugins/      # Motor de plugins
│   ├── storage/      # Archivos locales/remotos
│   ├── automation/   # Tareas programadas cron
│   ├── ui/           # GUI egui + ConnectionBridge
│   └── updater/      # Auto-actualización
├── src/main.rs
├── tests/            # 12 tests de integración
└── docs/             # 11 archivos de documentación
```

## [0.1.0] — 2026-07-16

### Añadido
- Arquitectura modular de workspace con 11 crates
- Core: tipos, traits, errores, eventos, IDs tipados
- Configuración TOML con hot-reload (notify)
- Base de datos SQLite con migraciones versionadas
- Cliente SSH async con Russh
- Network discovery (ping, port scan, subnet expansion)
- Cifrado AES-256-GCM + credential vault
- Monitor de sistema local (CPU, RAM, disco, red)
- Gestión de archivos locales
- Programador de tareas cron
- UI con temas Dark/Light
- Sidebar de navegación con 11 secciones
- Dashboard con métricas, Consola interactiva, Barra de estado
- Tests de integración (6 tests)
