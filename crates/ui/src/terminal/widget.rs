use super::ansi::AnsiColor;
use super::grid::TerminalGrid;
use egui::{Color32, Pos2, Rect, Vec2};

pub struct TerminalWidget {
    grid: TerminalGrid,
    font_size: f32,
    char_width: f32,
    char_height: f32,
    bg_color: Color32,
    scroll_offset: usize,
    show_cursor: bool,
    cursor_blink: f64,
}

impl TerminalWidget {
    pub fn new(rows: u16, cols: u16) -> Self {
        Self {
            grid: TerminalGrid::new(cols, rows),
            font_size: 13.0,
            char_width: 8.0,
            char_height: 18.0,
            bg_color: Color32::from_rgb(18, 18, 24),
            scroll_offset: 0,
            show_cursor: true,
            cursor_blink: 0.0,
        }
    }

    pub fn write(&mut self, data: &[u8]) {
        self.grid.write(data);
    }

    pub fn writeln(&mut self, data: &str) {
        self.grid.writeln(data);
    }

    pub fn clear(&mut self) {
        self.grid = TerminalGrid::new(self.grid.cols, self.grid.rows);
    }

    pub fn show(&mut self, ui: &mut egui::Ui, _theme: &crate::theme::Theme) {
        let available = ui.available_size();
        let cols = (available.x / self.char_width) as u16;
        let rows = (available.y / self.char_height) as u16;

        if cols != self.grid.cols || rows != self.grid.rows {
            self.grid.resize(cols.max(1), rows.max(1));
        }

        let response = ui.allocate_response(available, egui::Sense::click());
        let rect = response.rect;
        let default_fg = Color32::from_rgb(224, 224, 235);

        let painter = ui.painter().clone();
        painter.rect_filled(rect, 0.0, self.bg_color);

        self.render_lines(rect, &painter, default_fg);
        self.draw_cursor(rect, &painter);
    }

    fn render_lines(&self, rect: Rect, painter: &egui::Painter, default_fg: Color32) {
        let lines = self.grid.screen_lines();

        for (row_idx, line) in lines.iter().enumerate() {
            let y = rect.top() + (row_idx as f32 * self.char_height);
            if y + self.char_height > rect.bottom() {
                break;
            }

            let mut run_text = String::new();
            let mut run_color = default_fg;
            let mut run_x = rect.left();

            for (_col_idx, cell) in line.iter().enumerate() {
                let x = rect.left() + (_col_idx as f32 * self.char_width);
                let fg = cell_to_egui(&cell.fg, default_fg, self.bg_color);

                if cell.bg != AnsiColor::Default {
                    let cell_rect = Rect::from_min_size(
                        Pos2::new(x, y),
                        Vec2::new(self.char_width, self.char_height),
                    );
                    painter.rect_filled(cell_rect, 0.0, self.bg_color);
                }

                if fg != run_color || run_text.len() > 80 {
                    if !run_text.is_empty() {
                        painter.text(
                            Pos2::new(run_x, y + self.char_height - 4.0),
                            egui::Align2::LEFT_BOTTOM,
                            run_text.clone(),
                            egui::FontId::monospace(self.font_size),
                            run_color,
                        );
                        run_text.clear();
                    }
                    run_color = fg;
                    run_x = x;
                }

                run_text.push(cell.ch);
            }

            if !run_text.is_empty() {
                painter.text(
                    Pos2::new(run_x, y + self.char_height - 4.0),
                    egui::Align2::LEFT_BOTTOM,
                    run_text,
                    egui::FontId::monospace(self.font_size),
                    run_color,
                );
            }
        }
    }

    fn draw_cursor(&self, rect: Rect, painter: &egui::Painter) {
        if !self.show_cursor {
            return;
        }
        let (row, col) = self.grid.cursor_position();
        let x = rect.left() + (col as f32 * self.char_width);
        let y = rect.top() + (row as f32 * self.char_height);

        let cursor_rect = Rect::from_min_size(
            Pos2::new(x, y),
            Vec2::new(self.char_width, 2.0),
        );
        painter.rect_filled(cursor_rect, 0.0, Color32::WHITE);
    }
}

fn cell_to_egui(color: &AnsiColor, fg_default: Color32, _bg_default: Color32) -> Color32 {
    match color {
        AnsiColor::Default => fg_default,
        AnsiColor::Named(n) => {
            let (r, g, b) = n.to_rgb();
            Color32::from_rgb(r, g, b)
        }
        AnsiColor::Indexed(i) => {
            let (r, g, b) = super::ansi::indexed_to_rgb(*i);
            Color32::from_rgb(r, g, b)
        }
        AnsiColor::Rgb(r, g, b) => Color32::from_rgb(*r, *g, *b),
    }
}
