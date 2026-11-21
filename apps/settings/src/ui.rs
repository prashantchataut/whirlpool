//! Look and feel: palette, cards, rows, switches and the little previews
//! that show what a choice does before it is picked.

use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, FontId, Frame, Layout, Margin, Painter, Rect,
    Response, RichText, Sense, Stroke, StrokeKind, TextureHandle, Ui, Vec2, pos2, vec2,
};

pub const ACCENT: Color32 = Color32::from_rgb(0x3b, 0x82, 0xf6);
pub const DANGER: Color32 = Color32::from_rgb(0xef, 0x44, 0x44);

/// Colors that depend on the light or dark theme.
#[derive(Clone, Copy)]
pub struct Palette {
    pub window: Color32,
    pub sidebar: Color32,
    pub card: Color32,
    pub border: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub track: Color32,
}

impl Palette {
    pub fn of(ui: &Ui) -> Self {
        if ui.visuals().dark_mode {
            Self {
                window: Color32::from_rgb(0x14, 0x15, 0x19),
                sidebar: Color32::from_rgb(0x1a, 0x1b, 0x21),
                card: Color32::from_rgb(0x22, 0x24, 0x2b),
                border: Color32::from_rgb(0x32, 0x35, 0x3f),
                text: Color32::from_rgb(0xf1, 0xf2, 0xf5),
                muted: Color32::from_rgb(0x9a, 0x9f, 0xad),
                track: Color32::from_rgb(0x45, 0x49, 0x55),
            }
        } else {
            Self {
                window: Color32::from_rgb(0xf4, 0xf5, 0xf8),
                sidebar: Color32::from_rgb(0xea, 0xec, 0xf1),
                card: Color32::WHITE,
                border: Color32::from_rgb(0xdd, 0xe0, 0xe7),
                text: Color32::from_rgb(0x16, 0x18, 0x1d),
                muted: Color32::from_rgb(0x6a, 0x70, 0x7e),
                track: Color32::from_rgb(0xc5, 0xca, 0xd4),
            }
        }
    }
}

/// Applies the theme once; both light and dark follow Windows.
pub fn install_theme(ctx: &egui::Context) {
    ctx.set_theme(egui::ThemePreference::System);
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = vec2(10.0, 8.0);
        style.spacing.button_padding = vec2(14.0, 6.0);
        style.spacing.slider_width = 190.0;
        style.visuals.selection.bg_fill = ACCENT;
        style.visuals.selection.stroke = Stroke::new(1.0, Color32::WHITE);
        style.visuals.widgets.inactive.corner_radius = CornerRadius::same(8);
        style.visuals.widgets.hovered.corner_radius = CornerRadius::same(8);
        style.visuals.widgets.active.corner_radius = CornerRadius::same(8);
        style.visuals.window_corner_radius = CornerRadius::same(12);
    });
}

/// A rounded group of rows with an optional small title above it.
pub fn card(ui: &mut Ui, title: Option<&str>, add: impl FnOnce(&mut Ui)) {
    let palette = Palette::of(ui);
    if let Some(title) = title {
        ui.add_space(6.0);
        ui.label(
            RichText::new(title.to_uppercase())
                .size(11.5)
                .strong()
                .color(palette.muted),
        );
    }
    Frame::new()
        .fill(palette.card)
        .stroke(Stroke::new(1.0, palette.border))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::symmetric(16, 6))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 0.0;
            add(ui);
        });
    ui.add_space(6.0);
}

/// A separator between two rows of a card.
pub fn divider(ui: &mut Ui) {
    let palette = Palette::of(ui);
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, palette.border);
}

/// A label with an optional hint on the left and a control on the right.
pub fn row(
    ui: &mut Ui,
    label: &str,
    hint: Option<&str>,
    control_width: f32,
    control: impl FnOnce(&mut Ui),
) {
    let palette = Palette::of(ui);
    ui.horizontal(|ui| {
        ui.set_min_height(50.0);
        let text_width = (ui.available_width() - control_width - 24.0).max(120.0);
        ui.allocate_ui_with_layout(vec2(text_width, 50.0), Layout::top_down(Align::Min), |ui| {
            ui.set_width(text_width);
            ui.add_space(9.0);
            ui.label(RichText::new(label).size(14.0).color(palette.text));
            if let Some(hint) = hint {
                ui.add_space(1.0);
                ui.label(RichText::new(hint).size(12.0).color(palette.muted));
                ui.add_space(8.0);
            } else {
                ui.add_space(10.0);
            }
        });
        ui.with_layout(Layout::right_to_left(Align::Center), control);
    });
}

/// The pill switch every on/off setting uses.
pub fn switch(ui: &mut Ui, on: &mut bool) -> Response {
    let palette = Palette::of(ui);
    let (rect, mut response) = ui.allocate_exact_size(vec2(42.0, 24.0), Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    let t = ui.ctx().animate_bool_with_time(response.id, *on, 0.12);
    let track = palette.track.lerp_to_gamma(ACCENT, t);
    ui.painter()
        .rect_filled(rect, CornerRadius::same(12), track);
    let knob_x = egui::lerp((rect.left() + 12.0)..=(rect.right() - 12.0), t);
    ui.painter()
        .circle_filled(pos2(knob_x, rect.center().y), 9.0, Color32::WHITE);
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.expand(2.0),
            CornerRadius::same(14),
            Stroke::new(1.5, ACCENT),
            StrokeKind::Outside,
        );
    }
    response
}

/// A toggle row: the whole thing in one call.
pub fn toggle_row(ui: &mut Ui, label: &str, hint: Option<&str>, value: &mut bool) {
    row(ui, label, hint, 60.0, |ui| {
        switch(ui, value);
    });
}

/// A button, optionally in a stronger color.
pub fn push(ui: &mut Ui, label: &str, tone: Option<Color32>) -> Response {
    let mut text = RichText::new(label).size(13.0);
    let mut button = egui::Button::new(text.clone());
    if let Some(tone) = tone {
        text = text.color(Color32::WHITE);
        button = egui::Button::new(text).fill(tone);
    }
    ui.add(button.min_size(vec2(0.0, 30.0)))
}

/// One option of a visual choice: a preview card with a label under it.
pub struct Choice<'a, T> {
    pub value: T,
    pub label: &'a str,
    pub detail: Option<String>,
    pub preview: fn(&Painter, Rect, &Backdrop, T),
}

/// What previews draw behind themselves.
pub struct Backdrop<'a> {
    pub wallpaper: Option<&'a TextureHandle>,
    pub icon_size: u32,
}

pub fn visual_choices<T: Copy + PartialEq>(
    ui: &mut Ui,
    current: &mut T,
    backdrop: &Backdrop,
    options: &[Choice<T>],
) -> bool {
    let palette = Palette::of(ui);
    let mut changed = false;
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 14.0;
        for option in options {
            let selected = *current == option.value;
            let (rect, response) = ui.allocate_exact_size(vec2(138.0, 104.0), Sense::click());
            let preview_rect = Rect::from_min_size(rect.min, vec2(138.0, 76.0));
            let painter = ui.painter();
            (option.preview)(painter, preview_rect, backdrop, option.value);
            let (stroke_color, width) = if selected {
                (ACCENT, 2.5)
            } else if response.hovered() {
                (palette.muted, 1.5)
            } else {
                (palette.border, 1.0)
            };
            painter.rect_stroke(
                preview_rect,
                CornerRadius::same(10),
                Stroke::new(width, stroke_color),
                StrokeKind::Inside,
            );
            let caption = match &option.detail {
                Some(detail) => format!("{}  ·  {detail}", option.label),
                None => option.label.to_owned(),
            };
            painter.text(
                pos2(rect.center().x, rect.bottom() - 11.0),
                Align2::CENTER_CENTER,
                caption,
                FontId::proportional(12.5),
                if selected {
                    palette.text
                } else {
                    palette.muted
                },
            );
            if response.clicked() && !selected {
                *current = option.value;
                changed = true;
            }
        }
    });
    ui.add_space(10.0);
    changed
}

fn desktop(painter: &Painter, rect: Rect, backdrop: &Backdrop) {
    let round = CornerRadius::same(10);
    match backdrop.wallpaper {
        Some(texture) => {
            let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
            // Cover the preview without stretching the picture.
            let size = texture.size_vec2();
            let target = rect.width() / rect.height();
            let source = size.x / size.y;
            let uv = if source > target {
                let keep = target / source;
                Rect::from_min_max(pos2((1.0 - keep) / 2.0, 0.0), pos2((1.0 + keep) / 2.0, 1.0))
            } else {
                let keep = source / target;
                Rect::from_min_max(pos2(0.0, (1.0 - keep) / 2.0), pos2(1.0, (1.0 + keep) / 2.0))
            }
            .intersect(uv);
            painter.rect_filled(rect, round, Color32::BLACK);
            let mut mesh = egui::Mesh::with_texture(texture.id());
            mesh.add_rect_with_uv(rect.shrink(0.5), uv, Color32::WHITE);
            painter.add(mesh);
        }
        None => {
            painter.rect_filled(rect, round, Color32::from_rgb(0x2b, 0x3a, 0x67));
            painter.rect_filled(
                Rect::from_min_max(rect.center_top(), rect.right_bottom()).shrink(1.0),
                CornerRadius {
                    nw: 0,
                    ne: 10,
                    sw: 0,
                    se: 10,
                },
                Color32::from_rgb(0x7c, 0x4d, 0xb5),
            );
        }
    }
}

pub fn preview_dock(
    painter: &Painter,
    rect: Rect,
    backdrop: &Backdrop,
    position: muelle_config::DockPosition,
) {
    use muelle_config::DockPosition;
    desktop(painter, rect, backdrop);
    let glass = Color32::from_rgba_unmultiplied(255, 255, 255, 70);
    let icon = 5.0 + (backdrop.icon_size as f32 - 32.0) / 64.0 * 6.0;
    let gap = 3.0;
    let n = 5.0;
    let span = n * icon + (n - 1.0) * gap + 8.0;
    let thick = icon + 8.0;
    let (dock, horizontal) = match position {
        DockPosition::Bottom => (
            Rect::from_center_size(
                pos2(rect.center().x, rect.bottom() - 6.0 - thick / 2.0),
                vec2(span, thick),
            ),
            true,
        ),
        DockPosition::Left => (
            Rect::from_center_size(
                pos2(rect.left() + 6.0 + thick / 2.0, rect.center().y),
                vec2(thick, span),
            ),
            false,
        ),
        DockPosition::Right => (
            Rect::from_center_size(
                pos2(rect.right() - 6.0 - thick / 2.0, rect.center().y),
                vec2(thick, span),
            ),
            false,
        ),
    };
    painter.rect_filled(dock, CornerRadius::same(7), glass);
    for index in 0..5 {
        let offset = 4.0 + index as f32 * (icon + gap);
        let min = if horizontal {
            pos2(dock.left() + offset, dock.center().y - icon / 2.0)
        } else {
            pos2(dock.center().x - icon / 2.0, dock.top() + offset)
        };
        let color = [
            Color32::from_rgb(0xff, 0x6b, 0x6b),
            Color32::from_rgb(0xfe, 0xca, 0x57),
            Color32::from_rgb(0x48, 0xdb, 0xfb),
            Color32::from_rgb(0x1d, 0xd1, 0xa1),
            Color32::from_rgb(0xa2, 0x9b, 0xfe),
        ][index];
        painter.rect_filled(
            Rect::from_min_size(min, Vec2::splat(icon)),
            CornerRadius::same(2),
            color,
        );
    }
}

pub fn preview_island(
    painter: &Painter,
    rect: Rect,
    backdrop: &Backdrop,
    style: muelle_config::IslandStyle,
) {
    use muelle_config::IslandStyle;
    desktop(painter, rect, backdrop);
    let bar = Rect::from_min_size(rect.min, vec2(rect.width(), 12.0));
    painter.rect_filled(
        bar,
        CornerRadius {
            nw: 10,
            ne: 10,
            sw: 0,
            se: 0,
        },
        Color32::from_rgba_unmultiplied(0, 0, 0, 120),
    );
    let ink = Color32::from_rgb(0x0b, 0x0b, 0x0e);
    match style {
        IslandStyle::Notch => {
            let notch =
                Rect::from_min_size(pos2(rect.center().x - 24.0, rect.top()), vec2(48.0, 22.0));
            painter.rect_filled(
                notch,
                CornerRadius {
                    nw: 0,
                    ne: 0,
                    sw: 11,
                    se: 11,
                },
                ink,
            );
            painter.circle_filled(
                pos2(notch.left() + 11.0, notch.center().y),
                3.0,
                Color32::from_rgb(0x1d, 0xd1, 0xa1),
            );
        }
        _ => {
            let bubble = Rect::from_min_size(
                pos2(rect.right() - 62.0, rect.top() + 2.0),
                vec2(50.0, 16.0),
            );
            painter.rect_filled(bubble, CornerRadius::same(8), ink);
            painter.circle_filled(
                pos2(bubble.left() + 9.0, bubble.center().y),
                3.0,
                Color32::from_rgb(0x1d, 0xd1, 0xa1),
            );
        }
    }
}

pub fn preview_speed(
    painter: &Painter,
    rect: Rect,
    _backdrop: &Backdrop,
    speed: muelle_config::GenieSpeed,
) {
    use muelle_config::GenieSpeed;
    let palette_fill = Color32::from_rgb(0x1f, 0x22, 0x30);
    painter.rect_filled(rect, CornerRadius::same(10), palette_fill);
    // A bar that fills faster the faster the genie flies.
    let fraction = match speed {
        GenieSpeed::Normal => 0.5,
        GenieSpeed::Fast => 0.72,
        GenieSpeed::Faster => 0.94,
    };
    let track = Rect::from_center_size(rect.center(), vec2(rect.width() - 36.0, 8.0));
    painter.rect_filled(
        track,
        CornerRadius::same(4),
        Color32::from_rgb(0x3a, 0x3f, 0x52),
    );
    let mut filled = track;
    filled.set_right(track.left() + track.width() * fraction);
    painter.rect_filled(filled, CornerRadius::same(4), ACCENT);
}
