use egui::{self, Color32, RichText, Ui};
use super::super::theme::Theme;
use std::collections::HashMap;

#[derive(Clone)]
pub struct FileTab {
    pub id: String,
    pub title: String,
    pub path: String,
    pub content: String,
    pub modified: bool,
    pub language: String,
    pub cursor_pos: usize,
    pub scroll: f32,
    pub remote: bool,
}

pub struct FileEditor {
    pub tabs: Vec<FileTab>,
    pub active_tab: usize,
    pub next_id: usize,
}

impl Default for FileEditor {
    fn default() -> Self {
        Self {
            tabs: vec![],
            active_tab: 0,
            next_id: 0,
        }
    }
}

impl FileEditor {
    pub fn open(&mut self, path: &str, content: &str, remote: bool) {
        let id = self.next_id;
        self.next_id += 1;

        let title = path.split('/').last().unwrap_or(path).to_string();
        let language = detect_language(path);

        self.tabs.push(FileTab {
            id: id.to_string(),
            title,
            path: path.to_string(),
            content: content.to_string(),
            modified: false,
            language,
            cursor_pos: 0,
            scroll: 0.0,
            remote,
        });
        self.active_tab = self.tabs.len() - 1;
    }

    pub fn close(&mut self, idx: usize) {
        if self.tabs.is_empty() { return; }
        self.tabs.remove(idx);
        if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len().saturating_sub(1);
        }
    }

    pub fn active(&self) -> Option<&FileTab> {
        self.tabs.get(self.active_tab)
    }

    pub fn active_mut(&mut self) -> Option<&mut FileTab> {
        self.tabs.get_mut(self.active_tab)
    }

    pub fn show(&mut self, ui: &mut Ui, theme: &Theme) {
        if self.tabs.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(
                    RichText::new("No files open. Use the file browser to open files.")
                        .color(theme.colors.text_muted),
                );
            });
            return;
        }

        // Tab bar
        let mut close_idx: Option<usize> = None;
        ui.horizontal(|ui| {
            for (i, tab) in self.tabs.iter().enumerate() {
                let selected = i == self.active_tab;
                let mut label = if tab.modified {
                    format!("● {}", tab.title)
                } else {
                    tab.title.clone()
                };

                let tab_btn = egui::Button::new(
                    RichText::new(&label)
                        .color(if selected { Color32::WHITE } else { theme.colors.text_secondary })
                )
                .fill(if selected { theme.colors.accent_primary } else { theme.colors.bg_tertiary })
                .min_size(egui::vec2(80.0, 22.0));

                if ui.add(tab_btn).clicked() {
                    self.active_tab = i;
                }

                if ui.small_button("\u{2718}").clicked() {
                    close_idx = Some(i);
                }
            }
            if ui.small_button("+").clicked() {
                // New empty file
                self.open(&format!("untitled-{}", self.next_id), "", false);
            }
        });

        if let Some(idx) = close_idx {
            self.close(idx);
            return;
        }

        ui.separator();

        // Editor body
        if let Some(tab) = self.active_mut() {
            // Language indicator
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(&tab.language)
                        .small()
                        .color(theme.colors.text_muted),
                );
                ui.label(format!(" | {} lines", tab.content.lines().count()));

                if tab.remote {
                    ui.label(
                        RichText::new(" | remote")
                            .small()
                            .color(theme.colors.accent_warning),
                    );
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if tab.modified {
                        if ui.button("\u{1f4be} Save").clicked() {
                            tab.modified = false;
                        }
                    }
                });
            });
            ui.separator();

            // Text area
            let available = ui.available_size();
            egui::ScrollArea::vertical()
                .id_source("editor_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let mut content = tab.content.clone();
                    let response = ui.add_sized(
                        egui::vec2(available.x, available.y - 40.0),
                        egui::TextEdit::multiline(&mut content)
                            .font(egui::FontId::monospace(13.0))
                            .desired_width(f32::INFINITY)
                            .lock_focus(true)
                            .code_editor(),
                    );

                    if content != tab.content {
                        tab.content = content;
                        tab.modified = true;
                    }
                });

            // Status bar for cursor position
            ui.separator();
            ui.label(
                RichText::new(format!(
                    "Ln {}, Col {} | {} chars | {}",
                    tab.cursor_pos, 1,
                    tab.content.len(),
                    if tab.modified { "Modified" } else { "Saved" }
                ))
                .small()
                .color(theme.colors.text_muted),
            );
        }
    }
}

fn detect_language(path: &str) -> String {
    if let Some(ext) = path.rsplit('.').next() {
        match ext.to_lowercase().as_str() {
            "rs" => "Rust".into(),
            "go" => "Go".into(),
            "py" => "Python".into(),
            "js" => "JavaScript".into(),
            "ts" => "TypeScript".into(),
            "html" => "HTML".into(),
            "css" => "CSS".into(),
            "json" => "JSON".into(),
            "toml" => "TOML".into(),
            "yaml" | "yml" => "YAML".into(),
            "md" => "Markdown".into(),
            "sh" | "bash" => "Bash".into(),
            "sql" => "SQL".into(),
            "conf" | "cfg" | "ini" => "Config".into(),
            "xml" => "XML".into(),
            "dockerfile" | "dockerignore" => "Docker".into(),
            "txt" | "log" => "Plain Text".into(),
            _ => format!("{} file", ext),
        }
    } else {
        "Plain Text".into()
    }
}
