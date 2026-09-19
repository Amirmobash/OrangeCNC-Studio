use egui::{Color32, CornerRadius, Stroke, Visuals};

pub const ORANGE: Color32 = Color32::from_rgb(245, 124, 0);
pub const ORANGE_SOFT: Color32 = Color32::from_rgb(255, 247, 237);
pub const TEXT: Color32 = Color32::from_rgb(32, 32, 32);
pub const MUTED: Color32 = Color32::from_rgb(105, 105, 105);
pub const BORDER: Color32 = Color32::from_rgb(231, 231, 231);

pub fn apply(ctx: &egui::Context) {
    let mut visuals = Visuals::light();
    visuals.panel_fill = Color32::WHITE;
    visuals.window_fill = Color32::WHITE;
    visuals.faint_bg_color = ORANGE_SOFT;
    visuals.selection.bg_fill = ORANGE;
    visuals.selection.stroke = Stroke::new(1.0, ORANGE);
    visuals.widgets.active.bg_fill = ORANGE;
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(255, 138, 28);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(8);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(8);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(8);
    visuals.widgets.active.corner_radius = CornerRadius::same(8);
    ctx.set_visuals(visuals);
}
