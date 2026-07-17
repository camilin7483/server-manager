use egui::{self, Color32, RichText, Ui};
use super::super::theme::Theme;
use super::super::terminal::TerminalManager;

pub struct ConsolePanel {
    pub terminal: TerminalManager,
    pub command_input: String,
}

impl Default for ConsolePanel {
    fn default() -> Self {
        Self {
            terminal: TerminalManager::new(),
            command_input: String::new(),
        }
    }
}

impl ConsolePanel {
    pub fn show(&mut self, ui: &mut Ui, theme: &Theme) {
        // Tab bar
        ui.horizontal(|ui| {
            for i in 0..self.terminal.sessions.len() {
                let selected = i == self.terminal.active;
                let label = if selected {
                    RichText::new(&self.terminal.sessions[i].title)
                        .color(Color32::WHITE)
                        .strong()
                } else {
                    RichText::new(&self.terminal.sessions[i].title)
                        .color(theme.colors.text_secondary)
                };

                let tab_btn = egui::Button::new(label)
                    .fill(if selected { theme.colors.accent_primary } else { theme.colors.bg_tertiary })
                    .min_size(egui::vec2(100.0, 24.0));

                if ui.add(tab_btn).clicked() {
                    self.terminal.active = i;
                }
            }

            if ui.small_button("+").clicked() {
                let n = self.terminal.next_id;
                self.terminal.open_tab(&format!("Term {}", n));
            }
        });
        ui.separator();

        // Terminal widget
        let session = self.terminal.active_session_mut();
        session.widget.show(ui, theme);

        // Command input
        ui.separator();
        let response = ui.horizontal(|ui| {
            ui.label(RichText::new("$").color(theme.colors.accent_success).monospace());
            ui.text_edit_singleline(&mut session.command_input)
        });

        // Handle Enter
        if response.inner.lost_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter))
        {
            let cmd = session.command_input.trim().to_string();
            if !cmd.is_empty() {
                session.add_history(&cmd);
                session.widget.writeln(&format!("\x1b[1;34m> {}\x1b[0m", cmd));
                session.widget.writeln("(pendiente: conectar sesión SSH para ejecutar)");
            }
            session.command_input.clear();
        }

        // Handle arrow keys for history
        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            if let Some(cmd) = session.history_up() {
                session.command_input = cmd.to_string();
            }
        }
        if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            if let Some(cmd) = session.history_down() {
                session.command_input = cmd.to_string();
            }
        }
    }
}
