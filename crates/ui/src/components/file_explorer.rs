use egui::{self, Color32, RichText, Ui, Vec2};
use super::super::theme::Theme;
use sm_core::id::ServerId;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct FsNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: u64,
    pub permissions: String,
    pub children: Vec<FsNode>,
    pub expanded: bool,
    pub loaded: bool,
}

#[derive(Clone)]
pub struct FileOperation {
    pub operation: FileOp,
    pub source: String,
    pub target: String,
    pub progress: f32,
    pub done: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FileOp {
    Download,
    Upload,
    Copy,
    Move,
    Delete,
}

pub struct FileBrowser {
    pub root_nodes: Vec<FsNode>,
    pub current_path: String,
    pub selected_path: Option<String>,
    pub operations: Vec<FileOperation>,
    pub show_context_menu: bool,
    pub context_menu_pos: Option<egui::Pos2>,
    pub new_item_name: String,
    pub show_new_item_input: bool,
    pub server_id: Option<ServerId>,
}

impl Default for FileBrowser {
    fn default() -> Self {
        Self {
            root_nodes: vec![
                FsNode {
                    name: "/".into(), path: "/".into(), is_dir: true,
                    size: 0, modified: 0, permissions: "drwxr-xr-x".into(),
                    children: vec![
                        FsNode { name: "etc".into(), path: "/etc".into(), is_dir: true, size: 0, modified: 0, permissions: "drwxr-xr-x".into(), children: vec![], expanded: false, loaded: false },
                        FsNode { name: "home".into(), path: "/home".into(), is_dir: true, size: 0, modified: 0, permissions: "drwxr-xr-x".into(), children: vec![], expanded: false, loaded: false },
                        FsNode { name: "var".into(), path: "/var".into(), is_dir: true, size: 0, modified: 0, permissions: "drwxr-xr-x".into(), children: vec![], expanded: false, loaded: false },
                        FsNode { name: "tmp".into(), path: "/tmp".into(), is_dir: true, size: 0, modified: 0, permissions: "drwxrwxrwt".into(), children: vec![], expanded: false, loaded: false },
                        FsNode { name: "opt".into(), path: "/opt".into(), is_dir: true, size: 0, modified: 0, permissions: "drwxr-xr-x".into(), children: vec![], expanded: false, loaded: false },
                        FsNode { name: "usr".into(), path: "/usr".into(), is_dir: true, size: 0, modified: 0, permissions: "drwxr-xr-x".into(), children: vec![], expanded: false, loaded: false },
                    ],
                    expanded: true,
                    loaded: true,
                },
            ],
            current_path: "/".into(),
            selected_path: None,
            operations: Vec::new(),
            show_context_menu: false,
            context_menu_pos: None,
            new_item_name: String::new(),
            show_new_item_input: false,
            server_id: None,
        }
    }
}

impl FileBrowser {
    pub fn show(&mut self, ui: &mut Ui, theme: &Theme) {
        // Breadcrumb + actions
        ui.horizontal(|ui| {
            self.render_breadcrumb(ui, theme);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("\u{2b06} Upload").clicked() {
                    self.add_operation("/tmp/upload.txt", "/tmp/upload.txt", FileOp::Upload);
                }
                if ui.small_button("\u{1f4c1} New Dir").clicked() {
                    self.show_new_item_input = true;
                }
                if ui.small_button("\u{21bb}").on_hover_text("Refresh").clicked() {
                    // Refresh current path
                }
            });
        });
        ui.separator();

        // New item input
        if self.show_new_item_input {
            ui.horizontal(|ui| {
                ui.label("Name:");
                if ui.text_edit_singleline(&mut self.new_item_name).lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                {
                    let path = format!("{}/{}", self.current_path.trim_end_matches('/'), self.new_item_name.trim());
                    self.add_node(&path, true, 0);
                    self.new_item_name.clear();
                    self.show_new_item_input = false;
                }
            });
            ui.separator();
        }

        // Context menu
        if self.show_context_menu {
            egui::Area::new("file_context_menu".into())
                .fixed_pos(self.context_menu_pos.unwrap_or_default())
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.set_min_width(150.0);
                        if ui.button("\u{1f4cb} Copy").clicked() {
                            self.context_action(FileOp::Copy);
                            self.show_context_menu = false;
                        }
                        if ui.button("\u{2702} Move").clicked() {
                            self.context_action(FileOp::Move);
                            self.show_context_menu = false;
                        }
                        if ui.button("\u{2b07} Download").clicked() {
                            self.context_action(FileOp::Download);
                            self.show_context_menu = false;
                        }
                        ui.separator();
                        if ui.button("\u{1f5d1} Delete").clicked() {
                            self.context_action(FileOp::Delete);
                            self.show_context_menu = false;
                        }
                    });
                });
        }

        // Clone state for rendering to avoid borrow conflicts
        let nodes = self.root_nodes.clone();
        let selected = self.selected_path.clone();

        // File tree
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for node in &nodes {
                    self.render_node(ui, theme, &node, 0);
                }
            });

        // Progress bar for operations
        for op in &self.operations {
            if !op.done {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("{:?} {} → {}", op.operation, op.source, op.target))
                        .small()
                        .color(theme.colors.text_secondary));
                    ui.add(egui::ProgressBar::new(op.progress).desired_width(100.0));
                });
            }
        }
    }

    fn render_breadcrumb(&self, ui: &mut Ui, theme: &Theme) {
        let parts: Vec<&str> = self.current_path.split('/').filter(|s| !s.is_empty()).collect();
        ui.label("/");
        for (i, part) in parts.iter().enumerate() {
            if ui.link(RichText::new(*part).color(theme.colors.accent_primary)).clicked() {
                // Navigate to this path
            }
            if i < parts.len() - 1 {
                ui.label("/");
            }
        }
    }

    fn render_node(&mut self, ui: &mut Ui, theme: &Theme, node: &FsNode, depth: usize) {
        let mut node = node.clone();
        let indent = depth as f32 * 16.0;
        ui.add_space(indent);

        let icon = if node.is_dir {
            if node.expanded { "\u{1f4c2}" } else { "\u{1f4c1}" }
        } else {
            file_icon(&node.name)
        };

        let selected = self.selected_path.as_deref() == Some(&node.path);

        ui.horizontal(|ui| {
            if node.is_dir {
                let arrow = if node.expanded { "▾" } else { "▸" };
                if ui.selectable_label(false,
                    RichText::new(format!("{}{} {}", " ".repeat(depth), arrow, icon))
                        .color(theme.colors.text_secondary))
                    .clicked()
                {
                    // Toggle node expansion
                    self.toggle_node(&node.path);
                }
            } else {
                ui.label(format!("{}{} {}", " ".repeat(depth), "", icon));
            }

            let response = ui.selectable_label(
                selected,
                RichText::new(&node.name).color(if selected {
                    Color32::WHITE
                } else {
                    theme.colors.text_primary
                }),
            );

            if response.clicked() {
                if node.is_dir {
                    self.current_path = node.path.clone();
                    self.toggle_node(&node.path);
                }
                self.selected_path = Some(node.path.clone());
            }

            if response.secondary_clicked() {
                self.selected_path = Some(node.path.clone());
                self.show_context_menu = true;
                self.context_menu_pos = response.rect.left_bottom().into();
            }

            // Size and permissions
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if !node.is_dir {
                    ui.label(
                        RichText::new(human_size(node.size))
                            .small()
                            .color(theme.colors.text_muted),
                    );
                }
                ui.label(
                    RichText::new(&node.permissions)
                        .small()
                        .color(theme.colors.text_muted),
                );
            });
        });

        // Render children if expanded
        if node.expanded && !node.children.is_empty() {
            for child in &node.children {
                self.render_node(ui, theme, child, depth + 1);
            }
        }
    }

    fn toggle_node(&mut self, path: &str) {
        fn toggle_recursive(nodes: &mut Vec<FsNode>, path: &str) -> bool {
            for node in nodes.iter_mut() {
                if node.path == path {
                    node.expanded = !node.expanded;
                    // Populate with mock children if not loaded
                    if node.expanded && !node.loaded {
                        node.children = mock_children(path);
                        node.loaded = true;
                    }
                    return true;
                }
                if toggle_recursive(&mut node.children, path) {
                    return true;
                }
            }
            false
        }
        toggle_recursive(&mut self.root_nodes, path);
    }

    pub fn add_node(&mut self, path: &str, is_dir: bool, size: u64) {
        let parent_path = Path::new(path).parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "/".into());
        let name = Path::new(path).file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".into());

        fn add_recursive(nodes: &mut Vec<FsNode>, parent: &str, name: &str, is_dir: bool, size: u64, full_path: &str) -> bool {
            for node in nodes.iter_mut() {
                if node.path == parent {
                    if !node.children.iter().any(|c| c.name == name) {
                        node.children.push(FsNode {
                            name: name.to_string(),
                            path: full_path.to_string(),
                            is_dir,
                            size,
                            modified: chrono::Utc::now().timestamp() as u64,
                            permissions: if is_dir { "drwxr-xr-x".into() } else { "-rw-r--r--".into() },
                            children: vec![],
                            expanded: false,
                            loaded: false,
                        });
                    }
                    return true;
                }
                if add_recursive(&mut node.children, parent, name, is_dir, size, full_path) {
                    return true;
                }
            }
            false
        }
        add_recursive(&mut self.root_nodes, &parent_path, &name, is_dir, size, path);
    }

    fn context_action(&mut self, op: FileOp) {
        let path = self.selected_path.clone();
        if let Some(ref path) = path {
            self.add_operation(path, path, op);
        }
    }

    fn add_operation(&mut self, source: &str, target: &str, op: FileOp) {
        self.operations.push(FileOperation {
            operation: op,
            source: source.to_string(),
            target: target.to_string(),
            progress: 0.0,
            done: false,
        });
    }
}

fn file_icon(name: &str) -> &'static str {
    let ext = Path::new(name).extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match ext {
        "rs" | "go" | "py" | "js" | "ts" | "cpp" | "c" | "h" | "java" | "kt" | "swift" => "\u{1f4bb}",
        "toml" | "json" | "yaml" | "yml" | "xml" | "ini" | "cfg" => "\u{2699}",
        "md" | "txt" | "log" | "readme" => "\u{1f4c4}",
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "ico" | "webp" => "\u{1f5bc}",
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => "\u{1f4e6}",
        "sh" | "bash" | "zsh" => "\u{1f4dc}",
        "pdf" => "\u{1f4d5}",
        "sql" | "db" | "sqlite" => "\u{1f5c4}",
        "service" | "timer" => "\u{2699}",
        _ => "\u{1f4c4}",
    }
}

fn human_size(bytes: u64) -> String {
    if bytes < 1024 { format!("{} B", bytes) }
    else if bytes < 1024 * 1024 { format!("{:.1} KB", bytes as f64 / 1024.0) }
    else if bytes < 1024 * 1024 * 1024 { format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0)) }
    else { format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0)) }
}

fn mock_children(path: &str) -> Vec<FsNode> {
    match path {
        "/etc" => vec![
            FsNode { name: "hostname".into(), path: "/etc/hostname".into(), is_dir: false, size: 32, modified: 0, permissions: "-rw-r--r--".into(), children: vec![], expanded: false, loaded: true },
            FsNode { name: "hosts".into(), path: "/etc/hosts".into(), is_dir: false, size: 256, modified: 0, permissions: "-rw-r--r--".into(), children: vec![], expanded: false, loaded: true },
            FsNode { name: "nginx".into(), path: "/etc/nginx".into(), is_dir: true, size: 0, modified: 0, permissions: "drwxr-xr-x".into(), children: vec![], expanded: false, loaded: false },
            FsNode { name: "ssh".into(), path: "/etc/ssh".into(), is_dir: true, size: 0, modified: 0, permissions: "drwxr-xr-x".into(), children: vec![], expanded: false, loaded: false },
        ],
        "/home" => vec![
            FsNode { name: "admin".into(), path: "/home/admin".into(), is_dir: true, size: 0, modified: 0, permissions: "drwx------".into(), children: vec![], expanded: false, loaded: false },
        ],
        "/var" => vec![
            FsNode { name: "log".into(), path: "/var/log".into(), is_dir: true, size: 0, modified: 0, permissions: "drwxr-xr-x".into(), children: vec![], expanded: false, loaded: false },
            FsNode { name: "www".into(), path: "/var/www".into(), is_dir: true, size: 0, modified: 0, permissions: "drwxr-xr-x".into(), children: vec![], expanded: false, loaded: false },
        ],
        _ => vec![],
    }
}
