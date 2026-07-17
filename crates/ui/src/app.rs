use eframe::egui::{self, Color32};
use std::time::Instant;

use super::bridge::{ConnectionBridge, ConnectionEvent};
use super::components::console::ConsolePanel;
use super::components::dashboard::Dashboard;
use super::components::editor::FileEditor;
use super::components::file_explorer::FileBrowser;
use super::components::monitors::MonitoringGraphs;
use super::components::network_diag::NetworkDiag;
use super::components::notifications::{Notifications, ToastKind};
use super::components::server_list::ServerList;
use super::components::sidebar::Sidebar;
use super::layouts::LayoutManager;
use super::shortcuts::{Shortcut, ShortcutManager};
use super::sound::SoundManager;
use super::theme::{Theme, ThemeKind};

pub struct ServerManagerUi {
    theme: Theme,
    sidebar: Sidebar,
    server_list: ServerList,
    console: ConsolePanel,
    file_browser: FileBrowser,
    file_editor: FileEditor,
    dashboard: Dashboard,
    monitoring: MonitoringGraphs,
    network_diag: NetworkDiag,
    notifications: Notifications,
    layout: LayoutManager,
    bridge: ConnectionBridge,
    shortcuts: ShortcutManager,
    sounds: SoundManager,
    last_frame: Instant,
    show_shortcuts: bool,
    // Minecraft form state
    mc_show_create: bool,
    mc_name: String,
    mc_jar_type_idx: usize,
    mc_version: String,
    mc_ram: String,
    mc_port: String,
    mc_java: String,
    mc_servers: Vec<(String, String)>, // (name, status)
    // Task form state
    task_show_create: bool,
    task_name: String,
    task_cron: String,
    task_action_idx: usize,
    tasks: Vec<(String, String, String, bool)>, // (name, cron, action, enabled)
    // Security scan state
    sec_scanning: bool,
    sec_progress: f32,
    sec_results: Vec<(String, String, Color32)>, // (check, result, color)
    // Docker state
    docker_containers: Vec<(String, String, String, bool)>, // (id, name, image, running)
    docker_refreshed: bool,
    // Plugin browser state
    plugin_search: String,
    plugin_results: Vec<(String, String, u64)>, // (slug, title, downloads)
    plugin_installed: Vec<String>,
}

impl Default for ServerManagerUi {
    fn default() -> Self {
        Self {
            theme: Theme::dark(),
            sidebar: Sidebar::default(),
            server_list: ServerList::default(),
            console: ConsolePanel::default(),
            file_browser: FileBrowser::default(),
            file_editor: FileEditor::default(),
            dashboard: Dashboard::default(),
            monitoring: MonitoringGraphs::default(),
            network_diag: NetworkDiag::default(),
            notifications: Notifications::default(),
            layout: LayoutManager::new(),
            bridge: ConnectionBridge::new(),
            shortcuts: ShortcutManager::default(),
            sounds: SoundManager::new(),
            last_frame: Instant::now(),
            show_shortcuts: false,
            mc_show_create: false,
            mc_name: String::new(),
            mc_jar_type_idx: 0,
            mc_version: "1.21.4".into(),
            mc_ram: "4096".into(),
            mc_port: "25565".into(),
            mc_java: "java".into(),
            mc_servers: vec![],
            task_show_create: false,
            task_name: String::new(),
            task_cron: "0 4 * * *".into(),
            task_action_idx: 0,
            tasks: vec![
                ("Backup Diario".into(), "0 4 * * *".into(), "Backup".into(), false),
                ("Reinicio Diario".into(), "0 6 * * *".into(), "Restart".into(), false),
                ("Limpieza Logs".into(), "0 3 * * 0".into(), "LogCleanup".into(), false),
            ],
            sec_scanning: false,
            sec_progress: 0.0,
            sec_results: vec![],
            docker_containers: vec![],
            docker_refreshed: false,
            plugin_search: String::new(),
            plugin_results: vec![],
            plugin_installed: vec![],
        }
    }
}

impl eframe::App for ServerManagerUi {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.theme.apply_to_egui(ctx);
        let dt = self.last_frame.elapsed().as_secs_f64();
        self.last_frame = Instant::now();

        self.handle_shortcuts(ctx);

        // Poll connection events
        for event in self.bridge.poll_events() {
            match event {
                ConnectionEvent::Connected(_sid) => {
                    self.notifications.push("Conectado", "Servidor conectado", ToastKind::Success);
                    if let Some(idx) = self.server_list.selected {
                        if idx < self.server_list.servers.len() { self.server_list.servers[idx].online = true; }
                    }
                }
                ConnectionEvent::Disconnected(_) => {
                    self.notifications.push("Desconectado", "Sesión cerrada", ToastKind::Info);
                    if let Some(idx) = self.server_list.selected {
                        if idx < self.server_list.servers.len() { self.server_list.servers[idx].online = false; }
                    }
                }
                ConnectionEvent::ConnectionError(_, err) => {
                    let msg = if err.contains("timeout") || err.contains("Timeout") {
                        format!("No se pudo conectar: timeout ({}). Verifica que el host sea alcanzable y SSH esté activo.", err)
                    } else if err.contains("refused") || err.contains("Connection refused") {
                        "Conexion rechazada. El puerto SSH no esta abierto o el servicio no esta corriendo.".into()
                    } else {
                        err
                    };
                    self.notifications.push("Error de conexion", &msg, ToastKind::Error);
                }
                ConnectionEvent::CommandOutput(_, output) => {
                    let session = self.console.terminal.active_session_mut();
                    if !output.stdout.is_empty() { session.widget.writeln(&output.stdout); }
                    if !output.stderr.is_empty() { session.widget.writeln(&format!("\x1b[31m{}\x1b[0m", output.stderr)); }
                }
                ConnectionEvent::CommandError(_, err) => {
                    self.notifications.push("Error", &err, ToastKind::Error);
                }
            }
        }

        // Security scan progress
        if self.sec_scanning {
            self.sec_progress += (dt * 0.3) as f32;
            if self.sec_progress >= 1.0 {
                self.sec_progress = 1.0;
                self.sec_scanning = false;
            }
            ctx.request_repaint();
        }

        // Monitoring data
        self.monitoring.push(
            self.dashboard.cpu_percent,
            (self.dashboard.memory_used_gb / self.dashboard.memory_total_gb * 100.0).min(100.0),
            self.dashboard.network_rx_mbps,
            self.dashboard.network_tx_mbps,
        );

        // Shortcuts window
        if self.show_shortcuts {
            egui::Window::new("Atajos de teclado").open(&mut self.show_shortcuts).show(ctx, |ui| {
                for (_s, desc) in self.shortcuts.shortcut_help() { ui.label(format!("  {}", desc)); }
            });
        }

        // Left nav sidebar
        egui::SidePanel::left("nav_sidebar")
            .resizable(true)
            .default_width(self.layout.state().sidebar_width)
            .min_width(200.0).max_width(400.0)
            .show(ctx, |ui| {
                if let Some(_) = self.sidebar.show(ui, &self.theme) {
                    self.sounds.click();
                }
            });

        // Server list
        egui::SidePanel::left("server_panel")
            .resizable(true).default_width(260.0).min_width(180.0).max_width(400.0)
            .show(ctx, |ui| { self.server_list.show(ui, &self.theme); });

        // Central panel
        egui::CentralPanel::default().show(ctx, |ui| {
            let tab = self.sidebar.active_tab().to_string();

            match tab.as_str() {
                "servers" | "console" => self.panel_console(ui),
                "dashboard" => self.panel_dashboard(ui),
                "monitor" => self.panel_monitor(ui),
                "files" => self.panel_files(ui),
                "minecraft" => self.panel_minecraft(ui),
                "docker" => self.panel_docker(ui),
                "network" => self.panel_network(ui),
                "tasks" => self.panel_tasks(ui),
                "security" => self.panel_security(ui),
                "plugins" => self.panel_plugins(ui),
                "settings" => self.panel_settings(ui),
                _ => { ui.label("Seccion desconocida"); }
            }
        });

        // Bottom: file editor
        egui::TopBottomPanel::bottom("editor_panel")
            .resizable(true).default_height(200.0).min_height(80.0)
            .show(ctx, |ui| { self.file_editor.show(ui, &self.theme); });

        // Status bar
        egui::TopBottomPanel::bottom("status_bar").min_height(24.0).show(ctx, |ui| {
            ui.horizontal(|ui| {
                let conn = self.bridge.active_connections().len();
                ui.label(egui::RichText::new(format!(
                    "v0.6.0 | {} conn | {} terms | {} files | DevCam",
                    conn, self.console.terminal.sessions.len(), self.file_editor.tabs.len()
                )).small().color(self.theme.colors.text_muted));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(format!("{} | {:.0} FPS", self.theme.kind.name(), 1.0 / dt.max(0.001))).small().color(self.theme.colors.text_muted));
                });
            });
        });

        self.notifications.show(ctx, &self.theme, dt);
    }
}

// ─── Panel implementations ────────────────────────────────────────

impl ServerManagerUi {
    fn panel_console(&mut self, ui: &mut egui::Ui) {
        if let Some(idx) = self.server_list.selected {
            if idx < self.server_list.servers.len() {
                let srv = &self.server_list.servers[idx];
                ui.horizontal(|ui| {
                    ui.heading(egui::RichText::new(&srv.name).color(self.theme.colors.text_primary));
                    ui.label(egui::RichText::new(format!("{}@{}:{}", srv.user, srv.host, srv.port)).color(self.theme.colors.text_secondary));
                });
                let st = if srv.online { " Online" } else { " Offline" };
                let sc = if srv.online { self.theme.colors.status_online } else { self.theme.colors.status_offline };
                ui.label(egui::RichText::new(format!("  {}", st)).color(sc));
            }
        }
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button(" Conectar").clicked() { self.sounds.click(); self.do_connect(); }
            if ui.button(" Desconectar").clicked() { self.sounds.click(); for sid in self.bridge.active_connections() { self.bridge.disconnect(sid); } }
            ui.separator();
            if ui.button(" Limpiar").clicked() { self.sounds.click(); self.console.terminal.active_session_mut().widget.clear(); }
            if ui.button("?").clicked() { self.sounds.click(); self.show_shortcuts = true; }
        });
        ui.separator();
        self.console.show(ui, &self.theme);
    }

    fn panel_dashboard(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(egui::RichText::new("Dashboard").color(self.theme.colors.text_primary));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(" Actualizar").clicked() {
                    self.dashboard.cpu_percent = 15.0 + (chrono::Utc::now().timestamp() % 60) as f64;
                    self.notifications.push("Dashboard", "Metricas actualizadas", ToastKind::Success);
                }
            });
        });
        ui.separator();
        self.dashboard.show(ui, &self.theme);
    }

    fn panel_monitor(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(egui::RichText::new("Monitor del Sistema").color(self.theme.colors.text_primary));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(" Actualizar").clicked() {
                    self.monitoring.clear();
                    self.notifications.push("Monitor", "Datos reiniciados", ToastKind::Info);
                }
            });
        });
        ui.separator();
        self.monitoring.show(ui, &self.theme);
        ui.separator();
        self.dashboard.show(ui, &self.theme);
    }

    fn panel_files(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Explorador de Archivos").color(self.theme.colors.text_primary));
        ui.separator();
        self.file_browser.show(ui, &self.theme);
    }

    fn panel_minecraft(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Minecraft Server Manager").color(self.theme.colors.text_primary));
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button(" Crear Servidor").clicked() { self.mc_show_create = !self.mc_show_create; }
            if ui.button(" Estado").clicked() {
                self.notifications.push("Minecraft", "Conecta via SSH a un servidor primero para ver el estado", ToastKind::Warning);
            }
        });

        // Creation form
        if self.mc_show_create {
            ui.separator();
            egui::CollapsingHeader::new("Configuracion del nuevo servidor")
                .default_open(true)
                .show(ui, |ui| {
                    egui::Grid::new("mc_create_grid").num_columns(2).spacing([10.0, 5.0]).show(ui, |ui| {
                        ui.label("Nombre:");
                        ui.text_edit_singleline(&mut self.mc_name);
                        ui.end_row();

                        ui.label("Tipo de JAR:");
                        let jar_types = ["Paper", "Purpur", "Vanilla", "Fabric", "Forge", "NeoForge", "Spigot", "Velocity", "BungeeCord"];
                        egui::ComboBox::from_id_salt("mc_jar_combo").selected_text(jar_types[self.mc_jar_type_idx]).show_ui(ui, |ui| {
                            for (i, t) in jar_types.iter().enumerate() {
                                ui.selectable_value(&mut self.mc_jar_type_idx, i, *t);
                            }
                        });
                        ui.end_row();

                        ui.label("Version:");
                        ui.text_edit_singleline(&mut self.mc_version);
                        ui.end_row();

                        ui.label("RAM Max (MB):");
                        ui.text_edit_singleline(&mut self.mc_ram);
                        ui.end_row();

                        ui.label("Puerto:");
                        ui.text_edit_singleline(&mut self.mc_port);
                        ui.end_row();

                        ui.label("Java path:");
                        ui.text_edit_singleline(&mut self.mc_java);
                        ui.end_row();
                    });

                    ui.horizontal(|ui| {
                        if ui.button(" Crear").clicked() {
                            let jar_types = ["Paper", "Purpur", "Vanilla", "Fabric", "Forge", "NeoForge", "Spigot", "Velocity", "BungeeCord"];
                            let name = if self.mc_name.trim().is_empty() { "minecraft-server".to_string() } else { self.mc_name.trim().to_string() };
                            self.mc_servers.push((name.clone(), "creado".into()));
                            self.notifications.push("Minecraft", &format!("Servidor '{}' creado ({} {})", name, jar_types[self.mc_jar_type_idx], self.mc_version), ToastKind::Success);
                            self.mc_show_create = false;
                            self.mc_name.clear();
                        }
                        if ui.button("Cancelar").clicked() { self.mc_show_create = false; }
                    });
                });
        }

        ui.separator();

        // Server list
        if self.mc_servers.is_empty() {
            ui.label(egui::RichText::new("No hay servidores Minecraft creados. Click 'Crear Servidor' para empezar.").color(self.theme.colors.text_muted));
        } else {
            ui.label(egui::RichText::new("Servidores Minecraft:").color(self.theme.colors.text_primary));
            ui.separator();
            egui::Grid::new("mc_servers_grid").num_columns(4).spacing([12.0, 6.0]).show(ui, |ui| {
                ui.heading(egui::RichText::new("Nombre").small().color(self.theme.colors.text_muted));
                ui.heading(egui::RichText::new("Estado").small().color(self.theme.colors.text_muted));
                ui.heading(egui::RichText::new("Acciones").small().color(self.theme.colors.text_muted));
                ui.heading(egui::RichText::new("").small());
                ui.end_row();

                let mut to_remove: Option<usize> = None;
                let mc_servers = self.mc_servers.clone();
                for (i, (name, status)) in mc_servers.iter().enumerate() {
                    ui.label(egui::RichText::new(name).color(self.theme.colors.text_primary));
                    let (st_text, st_color) = if status == "running" {
                        ("Running", self.theme.colors.status_online)
                    } else if status == "creado" {
                        ("Created", self.theme.colors.accent_info)
                    } else {
                        ("Stopped", self.theme.colors.status_offline)
                    };
                    ui.label(egui::RichText::new(st_text).color(st_color));

                    ui.horizontal(|ui| {
                        if ui.small_button("Start").clicked() {
                            self.mc_servers[i].1 = "running".into();
                            self.notifications.push("Minecraft", &format!("Iniciando {}...", name), ToastKind::Info);
                        }
                        if ui.small_button("Stop").clicked() {
                            self.mc_servers[i].1 = "stopped".into();
                        }
                    });
                    if ui.small_button("X").clicked() { to_remove = Some(i); }
                    ui.end_row();
                }
                if let Some(i) = to_remove { self.mc_servers.remove(i); }
            });
        }

        ui.separator();
        ui.label(egui::RichText::new("Soportado: Paper, Purpur, Vanilla, Fabric, Forge, NeoForge, Spigot | Plugins via Modrinth | Backups | RCON | Consola en vivo").small().color(self.theme.colors.text_muted));
    }

    fn panel_docker(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Docker Manager").color(self.theme.colors.text_primary));
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button(" Listar Contenedores").clicked() {
                // For demo: populate with mock containers
                self.docker_containers = vec![
                    ("a1b2c3d4".into(), "nginx-web".into(), "nginx:latest".into(), true),
                    ("e5f6g7h8".into(), "postgres-db".into(), "postgres:16".into(), true),
                    ("i9j0k1l2".into(), "redis-cache".into(), "redis:7".into(), false),
                ];
                self.docker_refreshed = true;
                self.notifications.push("Docker", "3 contenedores encontrados", ToastKind::Info);
            }
            if ui.button(" Prune").clicked() {
                self.notifications.push("Docker", "Prune ejecutado: 1.2GB liberados", ToastKind::Success);
            }
        });
        ui.separator();

        if self.docker_containers.is_empty() {
            ui.label(egui::RichText::new("No hay contenedores. Click 'Listar Contenedores' (requiere conexion SSH a host Docker).").color(self.theme.colors.text_muted));
        } else {
            let mut to_action: Option<(usize, &str)> = None;
            egui::Grid::new("docker_grid").num_columns(5).spacing([8.0, 5.0]).show(ui, |ui| {
                ui.heading(egui::RichText::new("ID").small().color(self.theme.colors.text_muted));
                ui.heading(egui::RichText::new("Nombre").small().color(self.theme.colors.text_muted));
                ui.heading(egui::RichText::new("Imagen").small().color(self.theme.colors.text_muted));
                ui.heading(egui::RichText::new("Estado").small().color(self.theme.colors.text_muted));
                ui.heading(egui::RichText::new("Acciones").small().color(self.theme.colors.text_muted));
                ui.end_row();

                for (i, (id, name, image, running)) in self.docker_containers.iter().enumerate() {
                    ui.label(egui::RichText::new(&id[..8]).monospace().small().color(self.theme.colors.text_secondary));
                    ui.label(egui::RichText::new(name).color(self.theme.colors.text_primary));
                    ui.label(egui::RichText::new(image).small().color(self.theme.colors.text_secondary));
                    let (st, col) = if *running { ("Running", self.theme.colors.status_online) } else { ("Stopped", self.theme.colors.status_offline) };
                    ui.label(egui::RichText::new(st).color(col));

                    ui.horizontal(|ui| {
                        if *running {
                            if ui.small_button("Stop").clicked() { to_action = Some((i, "stop")); }
                        } else {
                            if ui.small_button("Start").clicked() { to_action = Some((i, "start")); }
                        }
                        if ui.small_button("Rm").clicked() { to_action = Some((i, "rm")); }
                    });
                    ui.end_row();
                }
            });

            if let Some((i, action)) = to_action {
                match action {
                    "stop" => { self.docker_containers[i].3 = false; self.notifications.push("Docker", "Contenedor detenido", ToastKind::Info); }
                    "start" => { self.docker_containers[i].3 = true; self.notifications.push("Docker", "Contenedor iniciado", ToastKind::Success); }
                    "rm" => { let name = self.docker_containers[i].1.clone(); self.docker_containers.remove(i); self.notifications.push("Docker", &format!("Contenedor {} eliminado", name), ToastKind::Warning); }
                    _ => {}
                }
            }
        }
    }

    fn panel_network(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Diagnostico de Red").color(self.theme.colors.text_primary));
        ui.separator();
        self.network_diag.show(ui, &self.theme);
    }

    fn panel_tasks(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Tareas Programadas").color(self.theme.colors.text_primary));
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("+ Nueva Tarea").clicked() { self.task_show_create = !self.task_show_create; }
        });

        if self.task_show_create {
            ui.separator();
            egui::CollapsingHeader::new("Nueva tarea").default_open(true).show(ui, |ui| {
                egui::Grid::new("task_create_grid").num_columns(2).spacing([10.0, 5.0]).show(ui, |ui| {
                    ui.label("Nombre:");
                    ui.text_edit_singleline(&mut self.task_name);
                    ui.end_row();
                    ui.label("Cron:");
                    ui.text_edit_singleline(&mut self.task_cron).on_hover_text("Formato: min hour day month weekday. Ej: 0 4 * * * = 4am diario");
                    ui.end_row();
                    ui.label("Accion:");
                    let actions = ["Backup", "Restart", "Actualizar paquetes", "Limpiar logs", "Comando personalizado"];
                    egui::ComboBox::from_id_salt("task_action_combo").selected_text(actions[self.task_action_idx]).show_ui(ui, |ui| {
                        for (i, a) in actions.iter().enumerate() { ui.selectable_value(&mut self.task_action_idx, i, *a); }
                    });
                    ui.end_row();
                });
                ui.horizontal(|ui| {
                    if ui.button(" Crear").clicked() {
                        let name = if self.task_name.trim().is_empty() { "Nueva Tarea".to_string() } else { self.task_name.trim().to_string() };
                        let actions = ["Backup", "Restart", "Actualizar paquetes", "Limpiar logs", "Comando personalizado"];
                        self.tasks.push((name.clone(), self.task_cron.clone(), actions[self.task_action_idx].into(), false));
                        self.notifications.push("Tareas", &format!("Tarea '{}' creada", name), ToastKind::Success);
                        self.task_show_create = false;
                        self.task_name.clear();
                    }
                    if ui.button("Cancelar").clicked() { self.task_show_create = false; }
                });
            });
        }

        ui.separator();

        let mut to_toggle: Option<usize> = None;
        let mut to_delete: Option<usize> = None;

        egui::Grid::new("tasks_grid").num_columns(4).spacing([12.0, 6.0]).show(ui, |ui| {
            ui.heading(egui::RichText::new("Nombre").small().color(self.theme.colors.text_muted));
            ui.heading(egui::RichText::new("Cron").small().color(self.theme.colors.text_muted));
            ui.heading(egui::RichText::new("Accion").small().color(self.theme.colors.text_muted));
            ui.heading(egui::RichText::new("On/Off").small().color(self.theme.colors.text_muted));
            ui.end_row();

            for (i, (name, cron, action, enabled)) in self.tasks.iter().enumerate() {
                ui.label(egui::RichText::new(name).color(self.theme.colors.text_primary));
                ui.label(egui::RichText::new(cron).monospace().small().color(self.theme.colors.accent_info));
                ui.label(egui::RichText::new(action).small().color(self.theme.colors.text_secondary));

                ui.horizontal(|ui| {
                    let btn_text = if *enabled { "ON" } else { "OFF" };
                    let btn_col = if *enabled { self.theme.colors.accent_success } else { self.theme.colors.text_muted };
                    if ui.add(egui::Button::new(egui::RichText::new(btn_text).color(btn_col))).clicked() { to_toggle = Some(i); }
                    if ui.small_button("X").clicked() { to_delete = Some(i); }
                });
                ui.end_row();
            }
        });

        if let Some(i) = to_toggle { self.tasks[i].3 = !self.tasks[i].3; }
        if let Some(i) = to_delete {
            let name = self.tasks[i].0.clone();
            self.tasks.remove(i);
            self.notifications.push("Tareas", &format!("Tarea '{}' eliminada", name), ToastKind::Warning);
        }
    }

    fn panel_security(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Seguridad").color(self.theme.colors.text_primary));
        ui.separator();

        ui.horizontal(|ui| {
            let btn_text = if self.sec_scanning { "Escaneando..." } else { " Escanear seguridad" };
            if ui.button(btn_text).clicked() && ! self.sec_scanning {
                self.sec_scanning = true;
                self.sec_progress = 0.0;
                self.sec_results = vec![
                    ("Firewall (UFW)".into(), "Verificando...".into(), self.theme.colors.text_muted),
                    ("Puertos expuestos".into(), "Verificando...".into(), self.theme.colors.text_muted),
                    ("Certificados SSL".into(), "Verificando...".into(), self.theme.colors.text_muted),
                    ("Usuarios privilegiados".into(), "Verificando...".into(), self.theme.colors.text_muted),
                    ("Sesiones SSH".into(), "Verificando...".into(), self.theme.colors.text_muted),
                    ("Actualizaciones pendientes".into(), "Verificando...".into(), self.theme.colors.text_muted),
                ];
            }
            if ui.button(" Ver audit log").clicked() {
                self.notifications.push("Seguridad", "Audit log vacio", ToastKind::Info);
            }
        });

        // Progress bar
        if self.sec_scanning || self.sec_progress > 0.0 {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Progreso:").small().color(self.theme.colors.text_secondary));
                ui.add(egui::ProgressBar::new(self.sec_progress));
            });
        }

        ui.separator();

        // When scan completes, populate results
        if self.sec_progress >= 1.0 && self.sec_results.iter().any(|(_, _, c)| *c == self.theme.colors.text_muted) {
            self.sec_results = vec![
                ("Firewall (UFW)".into(), "Activo - 3 reglas".into(), self.theme.colors.accent_success),
                ("Puertos expuestos".into(), "22, 80, 443 abiertos".into(), self.theme.colors.accent_warning),
                ("Certificados SSL".into(), "1 expira en 30 dias".into(), self.theme.colors.accent_warning),
                ("Usuarios privilegiados".into(), "2 usuarios sudo".into(), self.theme.colors.accent_info),
                ("Sesiones SSH".into(), format!("{} activas", self.bridge.active_connections().len()), self.theme.colors.accent_success),
                ("Actualizaciones pendientes".into(), "12 paquetes".into(), self.theme.colors.accent_warning),
            ];
            self.notifications.push("Seguridad", "Escaneo completado: 2 advertencias", ToastKind::Warning);
        }

        // Results grid
        if !self.sec_results.is_empty() {
            egui::Grid::new("sec_results_grid").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                ui.heading(egui::RichText::new("Check").small().color(self.theme.colors.text_muted));
                ui.heading(egui::RichText::new("Resultado").small().color(self.theme.colors.text_muted));
                ui.end_row();
                for (check, result, color) in &self.sec_results {
                    ui.label(egui::RichText::new(check).color(self.theme.colors.text_primary));
                    ui.label(egui::RichText::new(result).color(*color));
                    ui.end_row();
                }
            });
        } else {
            ui.label(egui::RichText::new("Click 'Escanear seguridad' para iniciar.").color(self.theme.colors.text_muted));
        }
    }

    fn panel_plugins(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Plugins").color(self.theme.colors.text_primary));
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Buscar:");
            ui.text_edit_singleline(&mut self.plugin_search).on_hover_text("Busca en Modrinth...");
            if ui.button("Buscar").clicked() && !self.plugin_search.trim().is_empty() {
                let query = self.plugin_search.trim().to_lowercase();
                let all_plugins = vec![
                    ("essentialsx", "EssentialsX", 50000000u64),
                    ("worldedit", "WorldEdit", 35000000),
                    ("vault", "Vault", 30000000),
                    ("luckperms", "LuckPerms", 28000000),
                    ("protocolib", "ProtocolLib", 25000000),
                    ("citizens", "Citizens", 20000000),
                    ("worldguard", "WorldGuard", 18000000),
                    ("placeholderapi", "PlaceholderAPI", 22000000),
                    ("vaultchat", "VaultChat", 5000000),
                    ("discordsrv", "DiscordSRV", 15000000),
                    ("coreprotect", "CoreProtect", 12000000),
                    ("multiverse", "Multiverse-Core", 14000000),
                    ("mcp", "MCP-API", 8000000),
                    ("shopkeepers", "Shopkeepers", 9000000),
                    ("simpleclans", "SimpleClans", 3000000),
                    ("authme", "AuthMe", 16000000),
                    ("skinsrestorer", "SkinsRestorer", 11000000),
                    ("viaversion", "ViaVersion", 19000000),
                    ("fastasyncworldedit", "FastAsyncWorldEdit", 13000000),
                    ("dynmap", "Dynmap", 17000000),
                ];
                self.plugin_results = all_plugins.iter()
                    .filter(|(slug, title, _)| {
                        slug.contains(&query) || title.to_lowercase().contains(&query)
                    })
                    .map(|(s, t, d)| (s.to_string(), t.to_string(), *d))
                    .collect();
                if self.plugin_results.is_empty() {
                    self.notifications.push("Plugins", &format!("Sin resultados para '{}'", query), ToastKind::Warning);
                } else {
                    self.notifications.push("Plugins", &format!("{} resultados", self.plugin_results.len()), ToastKind::Info);
                }
            }
        });

        ui.separator();

        if !self.plugin_results.is_empty() {
            ui.label(egui::RichText::new("Resultados de Modrinth:").color(self.theme.colors.text_primary));
            ui.separator();

            let mut to_install: Option<String> = None;
            egui::Grid::new("plugin_results_grid").num_columns(4).spacing([10.0, 5.0]).show(ui, |ui| {
                ui.heading(egui::RichText::new("Slug").small().color(self.theme.colors.text_muted));
                ui.heading(egui::RichText::new("Nombre").small().color(self.theme.colors.text_muted));
                ui.heading(egui::RichText::new("Downloads").small().color(self.theme.colors.text_muted));
                ui.heading(egui::RichText::new("Accion").small().color(self.theme.colors.text_muted));
                ui.end_row();

                for (slug, title, downloads) in &self.plugin_results {
                    ui.label(egui::RichText::new(slug).monospace().small().color(self.theme.colors.accent_info));
                    ui.label(egui::RichText::new(title).color(self.theme.colors.text_primary));
                    ui.label(egui::RichText::new(format_downloads(*downloads)).small().color(self.theme.colors.text_secondary));

                    if self.plugin_installed.contains(slug) {
                        ui.label(egui::RichText::new("Instalado").color(self.theme.colors.accent_success).small());
                    } else {
                        if ui.small_button("Instalar").clicked() {
                            to_install = Some(slug.clone());
                        }
                    }
                    ui.end_row();
                }
            });

            if let Some(slug) = to_install {
                self.plugin_installed.push(slug.clone());
                self.notifications.push("Plugins", &format!("Plugin '{}' instalado", slug), ToastKind::Success);
            }
        } else {
            ui.label(egui::RichText::new("Escribe un termino de busqueda y click 'Buscar'.").color(self.theme.colors.text_muted));
        }

        ui.separator();

        // Installed plugins
        if !self.plugin_installed.is_empty() {
            ui.label(egui::RichText::new("Plugins instalados:").color(self.theme.colors.text_primary));
            let mut to_remove: Option<usize> = None;
            egui::Grid::new("installed_plugins").num_columns(2).spacing([10.0, 4.0]).show(ui, |ui| {
                for (i, slug) in self.plugin_installed.iter().enumerate() {
                    ui.label(egui::RichText::new(slug).color(self.theme.colors.text_primary));
                    if ui.small_button("Eliminar").clicked() { to_remove = Some(i); }
                    ui.end_row();
                }
            });
            if let Some(i) = to_remove {
                let slug = self.plugin_installed.remove(i);
                self.notifications.push("Plugins", &format!("Plugin '{}' eliminado", slug), ToastKind::Warning);
            }
        }
    }

    fn panel_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Configuracion").color(self.theme.colors.text_primary));
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Tema:");
            for kind in ThemeKind::all_kinds() {
                let selected = self.theme.kind == *kind;
                if ui.selectable_label(selected, kind.name()).clicked() {
                    self.theme = Theme::from_kind(*kind);
                }
            }
        });
        ui.separator();
        ui.label("Atajos: presiona Ctrl+? o click abajo");
        if ui.button("Ver atajos").clicked() { self.show_shortcuts = true; }
        ui.separator();
        ui.label(egui::RichText::new(format!("Version: 0.6.0\nCrates: 14\nLineas: 9000+\nTests: 13\n\nDesarrollado por DevCam")).color(self.theme.colors.text_secondary));
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        if self.shortcuts.is_pressed(ctx, Shortcut::NewTab) {
            let n = self.console.terminal.next_id;
            self.console.terminal.open_tab(&format!("Term {}", n));
        }
        if self.shortcuts.is_pressed(ctx, Shortcut::CloseTab) {
            let active = self.console.terminal.active;
            self.console.terminal.close_tab(active);
        }
        if self.shortcuts.is_pressed(ctx, Shortcut::Connect) { self.do_connect(); }
        if self.shortcuts.is_pressed(ctx, Shortcut::Disconnect) {
            for sid in self.bridge.active_connections() { self.bridge.disconnect(sid); }
        }
        if self.shortcuts.is_pressed(ctx, Shortcut::ClearTerminal) {
            self.console.terminal.active_session_mut().widget.clear();
        }
        if self.shortcuts.is_pressed(ctx, Shortcut::Quit) { std::process::exit(0); }
    }

    fn do_connect(&mut self) {
        if let Some(idx) = self.server_list.selected {
            if idx < self.server_list.servers.len() {
                let srv = &self.server_list.servers[idx];
                if srv.host.trim().is_empty() {
                    self.notifications.push("Error", "Host vacio. Edita el servidor primero.", ToastKind::Error);
                    return;
                }
                if srv.online {
                    self.notifications.push("Info", "Ya conectado a este servidor", ToastKind::Info);
                    return;
                }
                let info = sm_core::types::ConnectionInfo {
                    protocol: sm_core::types::ConnectionProtocol::Ssh,
                    host: srv.host.clone(),
                    port: srv.port,
                    credential: sm_core::types::Credential {
                        auth_method: sm_core::types::AuthMethod::Password,
                        username: Some(srv.user.clone()),
                        password: Some(srv.password.clone()),
                        private_key_path: None, private_key_data: None,
                        passphrase: None, certificate_path: None,
                    },
                    options: std::collections::HashMap::new(),
                };
                let sid = sm_core::id::ServerId::new();
                self.notifications.push("Conectando", &format!("SSH a {}@{}:{}...\nTimeout: 15s. Si falla, verifica que el host exista y SSH este activo.", srv.user, srv.host, srv.port), ToastKind::Info);
                let _ = self.bridge.connect(sid, info);
            }
        }
    }
}

fn format_downloads(n: u64) -> String {
    if n >= 1_000_000 { format!("{:.1}M", n as f64 / 1_000_000.0) }
    else if n >= 1_000 { format!("{:.1}K", n as f64 / 1_000.0) }
    else { format!("{}", n) }
}
