//! Look and feel: palette, cards, rows, switches and the little previews
//! that show what a choice does before it is picked.

use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, FontId, Frame, Layout, Margin, Painter, Rect,
    Response, RichText, Sense, Stroke, StrokeKind, TextureHandle, Ui, Vec2, pos2, vec2,
};

/// The current: a deep teal that holds 3:1 against both the abyss and the shallows.
pub const ACCENT: Color32 = Color32::from_rgb(0x0e, 0xa5, 0xa8);
/// Foam: the bright end of the water, used for highlights on dark surfaces.
pub const FOAM: Color32 = Color32::from_rgb(0x7a, 0xee, 0xde);
/// Coral: the single warm signal colour, reserved for things that need attention.
pub const DANGER: Color32 = Color32::from_rgb(0xf2, 0x64, 0x4a);

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
                window: Color32::from_rgb(0x07, 0x14, 0x19),
                sidebar: Color32::from_rgb(0x0a, 0x1b, 0x22),
                card: Color32::from_rgb(0x0e, 0x25, 0x2d),
                border: Color32::from_rgb(0x1b, 0x3a, 0x45),
                text: Color32::from_rgb(0xe6, 0xf4, 0xf2),
                muted: Color32::from_rgb(0x84, 0xa8, 0xad),
                track: Color32::from_rgb(0x2a, 0x4b, 0x56),
            }
        } else {
            Self {
                window: Color32::from_rgb(0xee, 0xf6, 0xf7),
                sidebar: Color32::from_rgb(0xdf, 0xed, 0xef),
                card: Color32::from_rgb(0xfa, 0xfd, 0xfd),
                border: Color32::from_rgb(0xcb, 0xdf, 0xe2),
                text: Color32::from_rgb(0x08, 0x1e, 0x24),
                muted: Color32::from_rgb(0x4f, 0x6f, 0x76),
                track: Color32::from_rgb(0xb4, 0xcc, 0xd0),
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
        style.visuals.hyperlink_color = ACCENT;
        style.visuals.text_cursor.stroke = Stroke::new(2.0, ACCENT);
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
            painter.rect_filled(rect, round, Color32::from_rgb(0x0c, 0x3b, 0x55));
            painter.rect_filled(
                Rect::from_min_max(rect.center_top(), rect.right_bottom()).shrink(1.0),
                CornerRadius {
                    nw: 0,
                    ne: 10,
                    sw: 0,
                    se: 10,
                },
                Color32::from_rgb(0x1f, 0x8f, 0xa0),
            );
        }
    }
}

pub fn preview_dock(
    painter: &Painter,
    rect: Rect,
    backdrop: &Backdrop,
    position: whirlpool_config::DockPosition,
) {
    use whirlpool_config::DockPosition;
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
            Color32::from_rgb(0xff, 0x8a, 0x6b),
            Color32::from_rgb(0xff, 0xd1, 0x8a),
            Color32::from_rgb(0x5a, 0xd1, 0xf0),
            Color32::from_rgb(0x2d, 0xe0, 0xb5),
            Color32::from_rgb(0x9a, 0xc4, 0xff),
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
    style: whirlpool_config::IslandStyle,
) {
    use whirlpool_config::IslandStyle;
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
            painter.circle_filled(pos2(notch.left() + 11.0, notch.center().y), 3.0, FOAM);
        }
        _ => {
            let bubble = Rect::from_min_size(
                pos2(rect.right() - 62.0, rect.top() + 2.0),
                vec2(50.0, 16.0),
            );
            painter.rect_filled(bubble, CornerRadius::same(8), ink);
            painter.circle_filled(pos2(bubble.left() + 9.0, bubble.center().y), 3.0, FOAM);
        }
    }
}

pub fn preview_speed(
    painter: &Painter,
    rect: Rect,
    _backdrop: &Backdrop,
    speed: whirlpool_config::GenieSpeed,
) {
    use whirlpool_config::GenieSpeed;
    let palette_fill = Color32::from_rgb(0x0b, 0x2a, 0x35);
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
        Color32::from_rgb(0x1e, 0x4a, 0x58),
    );
    let mut filled = track;
    filled.set_right(track.left() + track.width() * fraction);
    painter.rect_filled(filled, CornerRadius::same(4), ACCENT);
}

/// The Whirlpool mark: two spiral arms drawn as vector strokes, so it stays
/// sharp at any scale and needs no image asset.
pub fn vortex(painter: &Painter, center: egui::Pos2, radius: f32, color: Color32) {
    const STEPS: usize = 48;
    const TURNS: f32 = 1.55;
    for arm in 0..2 {
        let offset = arm as f32 * std::f32::consts::PI;
        let points: Vec<egui::Pos2> = (0..=STEPS)
            .map(|step| {
                let t = step as f32 / STEPS as f32;
                // Arms start wide and wind in; the curve eases so the eye lands at the middle.
                let r = radius * (1.0 - t).powf(0.85);
                let angle = offset + t * TURNS * std::f32::consts::TAU;
                center + vec2(angle.cos(), angle.sin()) * r
            })
            .collect();
        let width = if arm == 0 { 2.4 } else { 1.6 };
        painter.add(egui::Shape::line(points, Stroke::new(width, color)));
    }
    painter.circle_filled(center, radius * 0.1, color);
}

/// A quiet sine line: the waterline that sits under every page title.
pub fn waterline(ui: &mut Ui, width: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(width, 10.0), Sense::hover());
    let points: Vec<egui::Pos2> = (0..=60)
        .map(|i| {
            let t = i as f32 / 60.0;
            pos2(
                rect.left() + t * rect.width(),
                rect.center().y + (t * std::f32::consts::TAU * 3.0).sin() * 2.2,
            )
        })
        .collect();
    // Fades out to the right, like a ripple losing energy.
    let steps = points.len() - 1;
    for (index, pair) in points.windows(2).enumerate() {
        let fade = 1.0 - index as f32 / steps as f32;
        ui.painter().line_segment(
            [pair[0], pair[1]],
            Stroke::new(1.6, ACCENT.gamma_multiply(0.15 + 0.85 * fade)),
        );
    }
}
