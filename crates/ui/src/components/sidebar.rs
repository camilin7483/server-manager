use egui::{self, Color32, RichText, Rounding, Stroke, Vec2};

use super::super::theme::Theme;

pub struct Sidebar {
    pub selected_tab: usize,
    pub tabs: Vec<SidebarTab>,
}

pub struct SidebarTab {
    pub id: String,
    pub label: String,
    pub icon: String,
}

impl Default for Sidebar {
    fn default() -> Self {
        Self {
            selected_tab: 0,
            tabs: vec![
                SidebarTab { id: "servers".into(), label: "Servers".into(), icon: "\u{1f5a5}".into() },
                SidebarTab { id: "dashboard".into(), label: "Dashboard".into(), icon: "\u{1f4cb}".into() },
                SidebarTab { id: "monitor".into(), label: "Monitor".into(), icon: "\u{1f4ca}".into() },
                SidebarTab { id: "files".into(), label: "Files".into(), icon: "\u{1f4c1}".into() },
                SidebarTab { id: "docker".into(), label: "Docker".into(), icon: "\u{1f433}".into() },
                SidebarTab { id: "minecraft".into(), label: "Minecraft".into(), icon: "\u{26cf}".into() },
                SidebarTab { id: "network".into(), label: "Network".into(), icon: "\u{1f310}".into() },
                SidebarTab { id: "security".into(), label: "Security".into(), icon: "\u{1f6e1}".into() },
                SidebarTab { id: "tasks".into(), label: "Tasks".into(), icon: "\u{23f0}".into() },
                SidebarTab { id: "plugins".into(), label: "Plugins".into(), icon: "\u{1f9e9}".into() },
                SidebarTab { id: "settings".into(), label: "Settings".into(), icon: "\u{2699}".into() },
            ],
        }
    }
}

impl Sidebar {
    pub fn show(&mut self, ui: &mut egui::Ui, theme: &Theme) -> Option<usize> {
        let c = &theme.colors;
        let r = theme.rounding;
        let mut clicked: Option<usize> = None;

        // Header with gradient
        let header_h = 56.0;
        let avail_w = ui.available_width();
        let (header_area, _) = ui.allocate_exact_size(Vec2::new(avail_w, header_h), egui::Sense::hover());
        {
            let painter = ui.painter();
            painter.rect_filled(header_area, Rounding::same(0.0), c.sidebar_bg);

            // Logo text
            painter.text(
                header_area.center() - Vec2::new(0.0, 8.0),
                egui::Align2::CENTER_CENTER,
                "Server Manager",
                egui::FontId::proportional(16.0),
                c.accent_primary,
            );
            painter.text(
                header_area.center() + Vec2::new(0.0, 10.0),
                egui::Align2::CENTER_CENTER,
                "by DevCam",
                egui::FontId::proportional(9.0),
                c.text_muted,
            );
        }

        ui.add_space(4.0);

        // Tabs
        for (i, tab) in self.tabs.iter().enumerate() {
            let selected = self.selected_tab == i;
            let avail_w = ui.available_width();
            let h = 36.0;

            let (rect, response) = ui.allocate_exact_size(Vec2::new(avail_w, h), egui::Sense::click());
            let painter = ui.painter().clone();

            if selected {
                painter.rect_filled(rect, Rounding::same(r), c.sidebar_active);
                let bar = egui::Rect::from_min_size(rect.left_top(), Vec2::new(3.0, h));
                painter.rect_filled(bar, Rounding::same(0.0), c.accent_primary);
            } else if response.hovered() {
                painter.rect_filled(rect, Rounding::same(r), c.hover);
            }

            let icon_color = if selected { c.accent_primary } else { c.text_muted };
            let label_color = if selected { c.text_primary } else { c.text_secondary };

            painter.text(
                rect.min + Vec2::new(16.0, h / 2.0),
                egui::Align2::LEFT_CENTER,
                &tab.icon,
                egui::FontId::proportional(15.0),
                icon_color,
            );
            painter.text(
                rect.min + Vec2::new(40.0, h / 2.0),
                egui::Align2::LEFT_CENTER,
                &tab.label,
                egui::FontId::proportional(13.0),
                label_color,
            );

            if response.clicked() {
                self.selected_tab = i;
                clicked = Some(i);
            }
        }

        // Bottom: version info
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            ui.label(RichText::new("v0.6.0").small().color(c.text_muted));
            ui.add_space(4.0);
        });

        clicked
    }

    pub fn active_tab(&self) -> &str {
        &self.tabs[self.selected_tab].id
    }
}
