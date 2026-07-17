use egui::{self, Color32, RichText, Ui};
use super::super::theme::Theme;

#[derive(Clone)]
pub struct ServerGroup {
    pub id: String,
    pub name: String,
    pub color: Color32,
    pub collapsed: bool,
    pub server_ids: Vec<usize>,
}

pub struct ServerList {
    pub servers: Vec<ServerEntry>,
    pub groups: Vec<ServerGroup>,
    pub selected: Option<usize>,
    pub search: String,
    pub show_add: bool,
    pub editing_group: Option<String>,
    pub new_group_name: String,
    // Add server form fields
    pub new_name: String,
    pub new_host: String,
    pub new_port: String,
    pub new_user: String,
    pub new_password: String,
    pub new_os: String,
}

#[derive(Clone)]
pub struct ServerEntry {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub online: bool,
    pub os: String,
    pub tags: Vec<String>,
    pub group_id: Option<String>,
}

impl Default for ServerList {
    fn default() -> Self {
        Self {
            servers: vec![
                ServerEntry {
                    name: "Web Production".into(), host: "192.168.1.10".into(),
                    port: 22, user: "admin".into(), password: String::new(), online: true,
                    os: "Ubuntu 24.04".into(),
                    tags: vec!["web".into(), "production".into()],
                    group_id: Some("production".into()),
                },
                ServerEntry {
                    name: "DB Primary".into(), host: "192.168.1.20".into(),
                    port: 22, user: "dba".into(), password: String::new(), online: true,
                    os: "Debian 12".into(),
                    tags: vec!["db".into(), "production".into()],
                    group_id: Some("production".into()),
                },
                ServerEntry {
                    name: "Dev Server".into(), host: "10.0.0.5".into(),
                    port: 22, user: "dev".into(), password: String::new(), online: false,
                    os: "Arch Linux".into(),
                    tags: vec!["dev".into()],
                    group_id: Some("development".into()),
                },
            ],
            groups: vec![
                ServerGroup {
                    id: "production".into(),
                    name: "Producción".into(),
                    color: Color32::from_rgb(80, 200, 110),
                    collapsed: false,
                    server_ids: vec![0, 1],
                },
                ServerGroup {
                    id: "development".into(),
                    name: "Desarrollo".into(),
                    color: Color32::from_rgb(120, 180, 255),
                    collapsed: true,
                    server_ids: vec![2],
                },
            ],
            selected: Some(0),
            search: String::new(),
            show_add: false,
            editing_group: None,
            new_group_name: String::new(),
            new_name: String::new(),
            new_host: String::new(),
            new_port: "22".to_string(),
            new_user: "root".to_string(),
            new_password: String::new(),
            new_os: "Ubuntu".to_string(),
        }
    }
}

impl ServerList {
    pub fn show(&mut self, ui: &mut Ui, theme: &Theme) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Servidores").color(theme.colors.text_primary));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("+ Grupo").clicked() {
                    self.editing_group = Some(String::new());
                }
                if ui.small_button("+ Añadir").clicked() {
                    self.show_add = true;
                }
            });
        });

        ui.add_space(4.0);
        ui.text_edit_singleline(&mut self.search)
            .on_hover_text("Buscar servidores...");
        ui.separator();

        // Add server form
        if self.show_add {
            egui::CollapsingHeader::new("Nuevo Servidor")
                .default_open(true)
                .show(ui, |ui| {
                    egui::Grid::new("add_server_grid")
                        .num_columns(2)
                        .spacing([8.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Nombre:");
                            ui.text_edit_singleline(&mut self.new_name);
                            ui.end_row();
                            ui.label("Host/IP:");
                            ui.text_edit_singleline(&mut self.new_host);
                            ui.end_row();
                            ui.label("Puerto:");
                            ui.text_edit_singleline(&mut self.new_port);
                            ui.end_row();
                            ui.label("Usuario:");
                            ui.text_edit_singleline(&mut self.new_user);
                            ui.end_row();
                            ui.label("Contraseña:");
                            ui.text_edit_singleline(&mut self.new_password);
                            ui.end_row();
                            ui.label("OS:");
                            ui.text_edit_singleline(&mut self.new_os);
                            ui.end_row();
                        });

                    ui.horizontal(|ui| {
                        if ui.button("\u{2714} Crear").clicked() {
                            let port = self.new_port.parse::<u16>().unwrap_or(22);
                            let name = if self.new_name.trim().is_empty() {
                                self.new_host.clone()
                            } else {
                                self.new_name.trim().to_string()
                            };
                            self.servers.push(ServerEntry {
                                name,
                                host: self.new_host.trim().to_string(),
                                port,
                                user: self.new_user.trim().to_string(),
                                password: self.new_password.clone(),
                                online: false,
                                os: self.new_os.trim().to_string(),
                                tags: vec![],
                                group_id: None,
                            });
                            self.selected = Some(self.servers.len() - 1);
                            self.show_add = false;
                            self.new_name.clear();
                            self.new_host.clear();
                            self.new_port = "22".into();
                            self.new_password.clear();
                        }
                        if ui.button("Cancelar").clicked() {
                            self.show_add = false;
                            self.new_name.clear();
                            self.new_host.clear();
                            self.new_password.clear();
                        }
                    });
                });
            ui.separator();
        }

        // Group editing modal
        if let Some(ref mut name) = self.editing_group.clone() {
            ui.horizontal(|ui| {
                ui.label("Nombre del grupo:");
                ui.text_edit_singleline(name);
                if ui.button("Crear").clicked() {
                    let color = Color32::from_rgb(
                        ((name.len() as u8).wrapping_mul(50)) % 156 + 80,
                        ((name.len() as u8).wrapping_mul(80)) % 156 + 80,
                        ((name.len() as u8).wrapping_mul(110)) % 156 + 80,
                    );
                    self.groups.push(ServerGroup {
                        id: sanitize_id(name),
                        name: name.clone(),
                        color,
                        collapsed: false,
                        server_ids: vec![],
                    });
                    self.editing_group = None;
                }
                if ui.button("Cancelar").clicked() {
                    self.editing_group = None;
                }
            });
            ui.separator();
        }

        // Render groups with their servers
        for group in self.groups.clone() {
            self.render_group(ui, theme, &group);
        }

        // Ungrouped servers
        let grouped_indices: Vec<usize> = self.groups.iter()
            .flat_map(|g| g.server_ids.iter().copied())
            .collect();

        let ungrouped: Vec<usize> = (0..self.servers.len())
            .filter(|i| !grouped_indices.contains(i))
            .collect();

        if !ungrouped.is_empty() {
            ui.label(RichText::new("Sin grupo").small().color(theme.colors.text_muted));
            for idx in ungrouped {
                self.render_server_entry(ui, theme, idx);
                ui.add_space(2.0);
            }
        }
    }

    fn render_group(&mut self, ui: &mut Ui, theme: &Theme, group: &ServerGroup) {
        let group = group.clone(); // work on copy
        let collapsed = self.groups.iter()
            .find(|g| g.id == group.id)
            .map(|g| g.collapsed)
            .unwrap_or(false);

        ui.horizontal(|ui| {
            let arrow = if collapsed { "▸" } else { "▾" };
            if ui.selectable_label(false, RichText::new(format!("{} {}", arrow, group.name))
                .color(group.color))
                .clicked()
            {
                // Toggle collapse
                if let Some(g) = self.groups.iter_mut().find(|g| g.id == group.id) {
                    g.collapsed = !g.collapsed;
                }
            }
            ui.add_space(4.0);
            ui.label(
                RichText::new(format!("({})", group.server_ids.len()))
                    .small()
                    .color(theme.colors.text_muted),
            );
        });

        if !collapsed {
            for &srv_idx in &group.server_ids {
                if srv_idx < self.servers.len() {
                    // Filter by search
                    let srv = &self.servers[srv_idx];
                    if !self.search.is_empty()
                        && !srv.name.to_lowercase().contains(&self.search.to_lowercase())
                        && !srv.host.contains(&self.search)
                    {
                        continue;
                    }
                    self.render_server_entry(ui, theme, srv_idx);
                }
            }
        }
    }

    fn render_server_entry(&mut self, ui: &mut Ui, theme: &Theme, idx: usize) {
        let server = self.servers[idx].clone();
        let selected = self.selected == Some(idx);
        let mut to_delete: Option<usize> = None;
        let card_bg = if selected { theme.colors.sidebar_active } else { theme.colors.card_bg };
        let card_w = ui.available_width();

        // Card background
        let (card_rect, _) = ui.allocate_exact_size(egui::vec2(card_w, 52.0), egui::Sense::hover());
        let painter = ui.painter().clone();
        painter.rect_filled(card_rect, egui::Rounding::same(6.0), card_bg);
        if selected {
            let bar = egui::Rect::from_min_size(card_rect.left_top(), egui::vec2(3.0, 52.0));
            painter.rect_filled(bar, egui::Rounding::same(0.0), theme.colors.accent_primary);
        }

        // Line 1: dot + name + delete button
        ui.horizontal(|ui| {
            ui.add_space(12.0);
            let dot_color = if server.online { theme.colors.status_online } else { theme.colors.status_offline };
            let (_id, space_rect) = ui.allocate_space(egui::vec2(8.0, 16.0));
            ui.painter().circle_filled(space_rect.center(), 4.0, dot_color);

            let label = if selected {
                RichText::new(&server.name).color(Color32::WHITE).strong()
            } else {
                RichText::new(&server.name).color(theme.colors.text_primary)
            };
            let response = ui.selectable_label(selected, label);
            if response.clicked() { self.selected = Some(idx); }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("\u{2718}").on_hover_text("Eliminar").clicked() { to_delete = Some(idx); }
                ui.add_space(4.0);
            });
        });

        // Line 2: OS + user@host:port + tags
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            ui.label(RichText::new(format!("{} | {}@{}:{}", server.os, server.user, server.host, server.port))
                .small().color(theme.colors.text_muted));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                for tag in &server.tags {
                    let tag_bg = if tag == "production" { theme.colors.accent_warning } else if tag == "web" { theme.colors.accent_info } else { theme.colors.accent_secondary };
                    let tag_rect = ui.allocate_exact_size(egui::vec2(tag.len() as f32 * 7.0 + 12.0, 16.0), egui::Sense::hover()).0;
                    let p = ui.painter().clone();
                    p.rect_filled(tag_rect, egui::Rounding::same(3.0), Color32::from_rgba_premultiplied(tag_bg.r(), tag_bg.g(), tag_bg.b(), 40));
                    p.text(tag_rect.center(), egui::Align2::CENTER_CENTER, format!("#{}", tag), egui::FontId::proportional(9.0), tag_bg);
                }
            });
        });

        ui.add_space(2.0);

        if let Some(i) = to_delete {
            self.servers.remove(i);
            for g in &mut self.groups {
                g.server_ids.retain(|&x| x != i);
                for id in &mut g.server_ids { if *id > i { *id -= 1; } }
            }
            if self.selected == Some(i) {
                self.selected = if self.servers.is_empty() { None } else { Some(i.saturating_sub(1)) };
            } else if let Some(s) = self.selected { if s > i { self.selected = Some(s - 1); } }
        }
    }
}

fn sanitize_id(name: &str) -> String {
    name.to_lowercase().replace(' ', "-")
}
