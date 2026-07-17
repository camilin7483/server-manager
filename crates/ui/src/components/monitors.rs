use egui::{self, Color32, Frame, RichText};
use super::super::theme::Theme;

pub struct MonitoringGraphs {
    pub cpu_history: Vec<f64>,
    pub ram_history: Vec<f64>,
    pub net_rx_history: Vec<f64>,
    pub net_tx_history: Vec<f64>,
    pub max_points: usize,
}

impl Default for MonitoringGraphs {
    fn default() -> Self {
        Self {
            cpu_history: Vec::new(),
            ram_history: Vec::new(),
            net_rx_history: Vec::new(),
            net_tx_history: Vec::new(),
            max_points: 60,
        }
    }
}

impl MonitoringGraphs {
    pub fn push(&mut self, cpu: f64, ram_percent: f64, net_rx: f64, net_tx: f64) {
        if self.cpu_history.len() >= self.max_points {
            self.cpu_history.remove(0);
            self.ram_history.remove(0);
            self.net_rx_history.remove(0);
            self.net_tx_history.remove(0);
        }
        self.cpu_history.push(cpu);
        self.ram_history.push(ram_percent);
        self.net_rx_history.push(net_rx);
        self.net_tx_history.push(net_tx);
    }

    pub fn clear(&mut self) {
        self.cpu_history.clear();
        self.ram_history.clear();
        self.net_rx_history.clear();
        self.net_tx_history.clear();
    }

    pub fn show(&self, ui: &mut egui::Ui, theme: &Theme) {
        let avail_w = ui.available_width();
        let card_w = (avail_w / 2.0 - 8.0).max(100.0);

        egui::Grid::new("monitor_grid")
            .num_columns(2)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                self.sparkline(ui, theme, "CPU", &self.cpu_history, "%", theme.colors.accent_info, 100.0, card_w);
                self.sparkline(ui, theme, "RAM", &self.ram_history, "%", theme.colors.accent_warning, 100.0, card_w);
                ui.end_row();
                self.sparkline(ui, theme, "Red RX", &self.net_rx_history, "", theme.colors.accent_success, 100.0, card_w);
                self.sparkline(ui, theme, "Red TX", &self.net_tx_history, "", theme.colors.accent_secondary, 100.0, card_w);
                ui.end_row();
            });
    }

    fn sparkline(
        &self,
        ui: &mut egui::Ui,
        theme: &Theme,
        label: &str,
        data: &[f64],
        unit: &str,
        color: Color32,
        max_val: f64,
        card_w: f32,
    ) {
        let size = egui::vec2(card_w, 56.0);
        let (rect, _resp) = ui.allocate_exact_size(size, egui::Sense::hover());
        let painter = ui.painter();

        painter.rect_filled(rect, 4.0, theme.colors.bg_secondary);
        painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0, theme.colors.border));

        let current = data.last().copied().unwrap_or(0.0);

        // Label — short and clipped
        let label_text = if label.len() > 8 { &label[..8] } else { label };
        let value_str = format!("{}: {:.1}{}", label_text, current, unit);
        let display = if value_str.len() > 18 { &value_str[..18] } else { &value_str };

        painter.text(
            rect.left_top() + egui::vec2(6.0, 3.0),
            egui::Align2::LEFT_TOP,
            display,
            egui::FontId::proportional(10.0),
            theme.colors.text_secondary,
        );

        // Sparkline graph
        if data.len() >= 2 {
            let inner = rect.shrink2(egui::vec2(4.0, 14.0));
            let h = inner.height() as f64;
            let w = inner.width();
            let step_x = w / (self.max_points - 1) as f32;

            let points: Vec<egui::Pos2> = data.iter().enumerate().map(|(i, &v)| {
                let x = inner.left() + (i as f32 * step_x);
                let y = inner.bottom() - (v / max_val * h) as f32;
                egui::pos2(x, y)
            }).collect();

            if points.len() >= 2 {
                painter.add(egui::Shape::line(points.clone(), egui::Stroke::new(1.5, color)));

                let first = *points.first().unwrap();
                let last = *points.last().unwrap();
                let mut fill_pts = points;
                fill_pts.push(egui::pos2(last.x, inner.bottom()));
                fill_pts.push(egui::pos2(first.x, inner.bottom()));
                fill_pts.push(first);

                let fill_color = Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 40);
                painter.add(egui::Shape::convex_polygon(fill_pts, fill_color, egui::Stroke::NONE));
            }
        }
    }
}
