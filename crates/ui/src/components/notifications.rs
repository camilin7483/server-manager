use egui::{self, Color32, Frame, RichText};

use super::super::theme::Theme;

#[derive(Debug, Clone)]
pub struct Toast {
    pub title: String,
    pub message: String,
    pub kind: ToastKind,
    pub remaining: f64,
    pub duration: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Error,
}

impl ToastKind {
    pub fn color(&self, theme: &Theme) -> Color32 {
        match self {
            Self::Info => theme.colors.accent_info,
            Self::Success => theme.colors.accent_success,
            Self::Warning => theme.colors.accent_warning,
            Self::Error => theme.colors.accent_danger,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Info => "\u{2139}",
            Self::Success => "\u{2714}",
            Self::Warning => "\u{26a0}",
            Self::Error => "\u{2718}",
        }
    }
}

pub struct Notifications {
    pub toasts: Vec<Toast>,
    pub max_toasts: usize,
}

impl Default for Notifications {
    fn default() -> Self {
        Self {
            toasts: Vec::new(),
            max_toasts: 5,
        }
    }
}

impl Notifications {
    pub fn push(&mut self, title: &str, message: &str, kind: ToastKind) {
        if self.toasts.len() >= self.max_toasts {
            self.toasts.remove(0);
        }
        self.toasts.push(Toast {
            title: title.to_string(),
            message: message.to_string(),
            kind,
            remaining: 3.0,
            duration: 3.0,
        });
    }

    pub fn show(&mut self, ctx: &egui::Context, theme: &Theme, dt: f64) {
        // Update remaining times and remove expired
        let mut i = 0;
        while i < self.toasts.len() {
            self.toasts[i].remaining -= dt;
            if self.toasts[i].remaining <= 0.0 {
                self.toasts.remove(i);
            } else {
                i += 1;
            }
        }

        if self.toasts.is_empty() {
            return;
        }

        // Render toasts in top-right corner
        egui::Area::new("notifications".into())
            .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-8.0, 32.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                for toast in &self.toasts {
                    let alpha = (toast.remaining / toast.duration).min(1.0) as f32;
                    let color = toast.kind.color(theme);

                    Frame::window(ui.style())
                        .fill(theme.colors.bg_secondary)
                        .rounding(6.0)
                        .stroke(egui::Stroke::new(1.0, color.gamma_multiply(alpha)))
                        .inner_margin(egui::vec2(12.0, 8.0))
                        .show(ui, |ui| {
                            ui.set_max_width(280.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(toast.kind.icon())
                                        .color(color.gamma_multiply(alpha))
                                );
                                ui.vertical(|ui| {
                                    ui.label(
                                        RichText::new(&toast.title)
                                            .color(theme.colors.text_primary)
                                            .strong()
                                    );
                                    if !toast.message.is_empty() {
                                        ui.label(
                                            RichText::new(&toast.message)
                                                .small()
                                                .color(theme.colors.text_secondary)
                                        );
                                    }
                                });
                            });
                        });

                    ui.add_space(4.0);
                }
            });
    }
}
