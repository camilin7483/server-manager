# Roadmap

## Fase 1 — Fundación (COMPLETADO v0.1.0)

- [x] Workspace modular de 11 crates
- [x] Core types, traits, errors, events
- [x] Configuración TOML con hot-reload
- [x] SQLite + migraciones
- [x] Cliente SSH async (Russh)
- [x] Network discovery
- [x] Cifrado AES-256-GCM + vault
- [x] Monitor de sistema local
- [x] Gestión de archivos
- [x] Programador de tareas (cron)
- [x] UI modular con temas Dark/Light
- [x] Sidebar, dashboard, consola
- [x] Tests de integración (6)

## Fase 2 — Conexiones & Gestión (COMPLETADO v0.2.0)

- [x] SessionManager con pool de sesiones SSH
- [x] SshKeyManager (generar, importar, exportar)
- [x] SftpClient básico sobre SSH
- [x] Perfiles predefinidos (14 perfiles)
- [x] DockerManager (contenedores, imágenes, compose)
- [x] MinecraftManager (Paper, Purpur, Fabric, Forge)
- [x] ConnectionBridge UI ↔ SSH async
- [x] UI con botones Conectar/Desconectar
- [x] Tests expandidos (12 tests)

## Fase 3 — UI Avanzada (PRÓXIMO)

- [ ] Terminal ANSI multisesión
- [ ] SFTP browser gráfico en UI
- [ ] Drag & drop de archivos remotos
- [ ] Editor de archivos integrado
- [ ] Panel de monitoreo con gráficas en tiempo real
- [ ] Dashboard personalizable con widgets
- [ ] Notificaciones toast
- [ ] Atajos de teclado globales

## Fase 4 — Gestión de Servidores

- [ ] Configuración automática desde perfiles
- [ ] Gestión de servicios (systemd)
- [ ] Inventario de software instalado
- [ ] Actualizaciones pendientes
- [ ] Grupos visuales con colores
- [ ] Búsqueda avanzada (FTS5)
- [ ] Import/export de configuración

## Fase 5 — Web & Databases

- [ ] Virtual hosts (Nginx, Apache, Caddy)
- [ ] Let's Encrypt / SSL automatizado
- [ ] Gestión de bases de datos (PostgreSQL, MySQL)
- [ ] PHP-FPM, Node.js, Bun, Python backends

## Fase 6 — Seguridad & Auditoría

- [ ] RBAC (roles y permisos)
- [ ] Logs de auditoría en UI
- [ ] Escaneo de seguridad (puertos, certificados)
- [ ] Alertas configurables
- [ ] Firewall manager (UFW, nftables)

## Fase 7 — Avanzado

- [ ] Monitoreo con gráficas históricas
- [ ] API de plugins funcional
- [ ] Reportes HTML/PDF/JSON
- [ ] Auto-updater con rollback
- [ ] Kubernetes (básico)
- [ ] Clustering de servidores
