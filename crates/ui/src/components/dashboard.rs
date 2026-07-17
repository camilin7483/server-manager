use egui::{self, Color32, RichText};
use super::super::theme::Theme;

pub struct Dashboard {
    pub cpu_percent: f64,
    pub memory_used_gb: f64,
    pub memory_total_gb: f64,
    pub disk_used_gb: f64,
    pub disk_total_gb: f64,
    pub network_rx_mbps: f64,
    pub network_tx_mbps: f64,
    pub uptime_hours: f64,
    pub server_count: usize,
    pub online_count: usize,
}

impl Default for Dashboard {
    fn default() -> Self {
        Self {
            cpu_percent: 23.5,
            memory_used_gb: 4.2,
            memory_total_gb: 16.0,
            disk_used_gb: 128.0,
            disk_total_gb: 512.0,
            network_rx_mbps: 12.3,
            network_tx_mbps: 4.7,
            uptime_hours: 72.5,
            server_count: 5,
            online_count: 3,
        }
    }
}

impl Dashboard {
    pub fn show(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        ui.heading(RichText::new("Dashboard").color(theme.colors.text_primary));
        ui.separator();

        let avail_w = ui.available_width();
        let card_w = (avail_w / 2.0 - 8.0).max(120.0);

        egui::Grid::new("metrics_grid")
            .num_columns(2)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                self.metric_card(ui, theme, "CPU", &format!("{:.1}%", self.cpu_percent), self.cpu_percent / 100.0, theme.colors.accent_info, card_w);
                self.metric_card(ui, theme, "RAM", &format!("{:.1}/{:.0}G", self.memory_used_gb, self.memory_total_gb), self.memory_used_gb / self.memory_total_gb, theme.colors.accent_warning, card_w);
                ui.end_row();
                self.metric_card(ui, theme, "Disco", &format!("{:.0}/{:.0}G", self.disk_used_gb, self.disk_total_gb), self.disk_used_gb / self.disk_total_gb, theme.colors.accent_secondary, card_w);
                self.metric_card(ui, theme, "Red", &format!("D{:.0} U{:.0}", self.network_rx_mbps, self.network_tx_mbps), 0.0, theme.colors.accent_success, card_w);
                ui.end_row();
            });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("{} online / {} servers", self.online_count, self.server_count)).color(theme.colors.text_secondary));
            ui.label(RichText::new(format!(" | Up: {:.1}h", self.uptime_hours)).color(theme.colors.text_muted));
        });
    }

    fn metric_card(
        &self,
        ui: &mut egui::Ui,
        theme: &Theme,
        label: &str,
        value: &str,
        fraction: f64,
        accent: Color32,
        card_w: f32,
    ) {
        let size = egui::vec2(card_w, 68.0);
        let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());
        let painter = ui.painter();

        painter.rect_filled(rect, 6.0, theme.colors.bg_secondary);
        painter.rect_stroke(rect, 6.0, egui::Stroke::new(1.0, theme.colors.border));

        let pad = 10.0;
        let inner_w = rect.width() - pad * 2.0;

        // Label (small)
        painter.text(
            rect.left_top() + egui::vec2(pad, 5.0),
            egui::Align2::LEFT_TOP,
            label,
            egui::FontId::proportional(11.0),
            theme.colors.text_secondary,
        );

        // Value (truncate to fit card width)
        let val_font = egui::FontId::proportional(14.0);
        let mut display = value.to_string();
        loop {
            let w = painter.fonts(|f| f.glyph_width(&val_font, display.chars().last().unwrap_or(' ')));
            let total_w = display.chars().count() as f32 * w;
            if total_w < inner_w || display.len() <= 3 { break; }
            display.pop();
        }
        painter.text(
            rect.left_top() + egui::vec2(pad, 20.0),
            egui::Align2::LEFT_TOP,
            &display,
            val_font,
            theme.colors.text_primary,
        );

        // Progress bar
        if fraction > 0.0 {
            let bar_y = rect.bottom() - 8.0;
            let track_rect = egui::Rect::from_min_size(
                egui::pos2(rect.left() + pad, bar_y),
                egui::vec2(inner_w, 4.0),
            );
            painter.rect_filled(track_rect, 2.0, theme.colors.bg_tertiary);
            let bar_w = inner_w * fraction as f32;
            let fill_rect = egui::Rect::from_min_size(
                egui::pos2(rect.left() + pad, bar_y),
                egui::vec2(bar_w, 4.0),
            );
            painter.rect_filled(fill_rect, 2.0, accent);
        }
    }
}
