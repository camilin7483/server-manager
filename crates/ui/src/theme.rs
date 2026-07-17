use egui::{Color32, Rounding, Stroke, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeKind {
    Dark,
    Light,
    HighContrast,
    Solarized,
}

impl ThemeKind {
    pub fn all_kinds() -> &'static [ThemeKind] {
        &[ThemeKind::Dark, ThemeKind::Light, ThemeKind::HighContrast, ThemeKind::Solarized]
    }

    pub fn name(&self) -> &'static str {
        match self {
            ThemeKind::Dark => "Oscuro", ThemeKind::Light => "Claro",
            ThemeKind::HighContrast => "Alto Contraste", ThemeKind::Solarized => "Solarized",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub kind: ThemeKind,
    pub colors: Colors,
    pub rounding: f32,
    pub spacing: f32,
}

#[derive(Debug, Clone)]
pub struct Colors {
    pub bg_primary: Color32,
    pub bg_secondary: Color32,
    pub bg_tertiary: Color32,
    pub surface: Color32,
    pub surface_hover: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub text_accent: Color32,
    pub accent_primary: Color32,
    pub accent_secondary: Color32,
    pub accent_danger: Color32,
    pub accent_warning: Color32,
    pub accent_success: Color32,
    pub accent_info: Color32,
    pub status_online: Color32,
    pub status_offline: Color32,
    pub status_warning: Color32,
    pub status_error: Color32,
    pub border: Color32,
    pub separator: Color32,
    pub hover: Color32,
    pub selection: Color32,
    pub input_bg: Color32,
    pub scrollbar: Color32,
    pub gradient_start: Color32,
    pub gradient_end: Color32,
    pub sidebar_bg: Color32,
    pub sidebar_active: Color32,
    pub card_bg: Color32,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            kind: ThemeKind::Dark,
            colors: Colors {
                bg_primary: Color32::from_rgb(16, 16, 24),
                bg_secondary: Color32::from_rgb(22, 22, 32),
                bg_tertiary: Color32::from_rgb(30, 30, 42),
                surface: Color32::from_rgb(36, 36, 50),
                surface_hover: Color32::from_rgb(44, 44, 60),
                text_primary: Color32::from_rgb(230, 230, 240),
                text_secondary: Color32::from_rgb(160, 160, 185),
                text_muted: Color32::from_rgb(100, 100, 125),
                text_accent: Color32::from_rgb(120, 180, 255),
                accent_primary: Color32::from_rgb(88, 160, 255),
                accent_secondary: Color32::from_rgb(156, 100, 255),
                accent_danger: Color32::from_rgb(245, 85, 85),
                accent_warning: Color32::from_rgb(245, 195, 75),
                accent_success: Color32::from_rgb(70, 210, 120),
                accent_info: Color32::from_rgb(70, 185, 230),
                status_online: Color32::from_rgb(70, 210, 120),
                status_offline: Color32::from_rgb(175, 75, 75),
                status_warning: Color32::from_rgb(245, 195, 75),
                status_error: Color32::from_rgb(245, 85, 85),
                border: Color32::from_rgb(45, 45, 62),
                separator: Color32::from_rgb(40, 40, 55),
                hover: Color32::from_rgba_premultiplied(88, 160, 255, 20),
                selection: Color32::from_rgba_premultiplied(88, 160, 255, 50),
                input_bg: Color32::from_rgb(24, 24, 36),
                scrollbar: Color32::from_rgb(55, 55, 75),
                gradient_start: Color32::from_rgb(88, 160, 255),
                gradient_end: Color32::from_rgb(156, 100, 255),
                sidebar_bg: Color32::from_rgb(14, 14, 22),
                sidebar_active: Color32::from_rgba_premultiplied(88, 160, 255, 30),
                card_bg: Color32::from_rgb(26, 26, 38),
            },
            rounding: 8.0,
            spacing: 10.0,
        }
    }

    pub fn light() -> Self {
        Self {
            kind: ThemeKind::Light,
            colors: Colors {
                bg_primary: Color32::from_rgb(248, 248, 252),
                bg_secondary: Color32::from_rgb(238, 238, 245),
                bg_tertiary: Color32::from_rgb(228, 228, 238),
                surface: Color32::from_rgb(255, 255, 255),
                surface_hover: Color32::from_rgb(240, 240, 250),
                text_primary: Color32::from_rgb(30, 30, 45),
                text_secondary: Color32::from_rgb(100, 100, 125),
                text_muted: Color32::from_rgb(160, 160, 180),
                text_accent: Color32::from_rgb(40, 100, 210),
                accent_primary: Color32::from_rgb(50, 120, 230),
                accent_secondary: Color32::from_rgb(120, 80, 230),
                accent_danger: Color32::from_rgb(210, 55, 55),
                accent_warning: Color32::from_rgb(210, 150, 30),
                accent_success: Color32::from_rgb(40, 175, 85),
                accent_info: Color32::from_rgb(40, 150, 195),
                status_online: Color32::from_rgb(40, 175, 85),
                status_offline: Color32::from_rgb(210, 55, 55),
                status_warning: Color32::from_rgb(210, 150, 30),
                status_error: Color32::from_rgb(210, 55, 55),
                border: Color32::from_rgb(215, 215, 225),
                separator: Color32::from_rgb(225, 225, 235),
                hover: Color32::from_rgba_premultiplied(50, 120, 230, 15),
                selection: Color32::from_rgba_premultiplied(50, 120, 230, 25),
                input_bg: Color32::from_rgb(240, 240, 248),
                scrollbar: Color32::from_rgb(195, 195, 210),
                gradient_start: Color32::from_rgb(50, 120, 230),
                gradient_end: Color32::from_rgb(120, 80, 230),
                sidebar_bg: Color32::from_rgb(235, 235, 242),
                sidebar_active: Color32::from_rgba_premultiplied(50, 120, 230, 20),
                card_bg: Color32::from_rgb(255, 255, 255),
            },
            rounding: 8.0,
            spacing: 10.0,
        }
    }

    pub fn from_kind(kind: ThemeKind) -> Self {
        match kind {
            ThemeKind::Dark => Self::dark(),
            ThemeKind::Light => Self::light(),
            ThemeKind::HighContrast => Self::dark(),
            ThemeKind::Solarized => Self::dark(),
        }
    }

    pub fn apply_to_egui(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        let c = &self.colors;
        let r = self.rounding;

        style.visuals.override_text_color = Some(c.text_primary);
        style.visuals.window_fill = c.surface;
        style.visuals.panel_fill = c.bg_primary;
        style.visuals.widgets.noninteractive.bg_fill = c.bg_secondary;
        style.visuals.widgets.inactive.bg_fill = c.bg_tertiary;
        style.visuals.widgets.hovered.bg_fill = c.surface_hover;
        style.visuals.widgets.active.bg_fill = c.surface;
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, c.text_secondary);
        style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, c.text_primary);
        style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, c.accent_primary);
        style.visuals.widgets.inactive.weak_bg_fill = c.input_bg;
        style.visuals.widgets.hovered.weak_bg_fill = c.surface;
        style.visuals.widgets.active.weak_bg_fill = c.selection;
        style.visuals.selection.bg_fill = c.selection;
        style.visuals.selection.stroke = Stroke::new(1.0, c.accent_primary);
        style.visuals.widgets.inactive.rounding = Rounding::same(r);
        style.visuals.widgets.hovered.rounding = Rounding::same(r);
        style.visuals.widgets.active.rounding = Rounding::same(r);
        style.visuals.window_rounding = Rounding::same(r);
        style.visuals.menu_rounding = Rounding::same(r);
        style.visuals.window_shadow.blur = 12.0;
        style.visuals.faint_bg_color = c.bg_tertiary;
        style.visuals.extreme_bg_color = c.input_bg;
        style.visuals.code_bg_color = c.input_bg;
        style.visuals.warn_fg_color = c.accent_warning;
        style.visuals.error_fg_color = c.accent_danger;
        style.visuals.hyperlink_color = c.accent_primary;
        style.spacing.item_spacing = Vec2::new(self.spacing, 5.0);
        style.spacing.button_padding = Vec2::new(14.0, 7.0);

        ctx.set_style(style);
    }
}
