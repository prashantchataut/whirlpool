//! Settings cards with live previews: quick settings tiles, hot corners, dock icon styles, glass.

use eframe::egui::{
    self, Color32, CornerRadius, Id, Rect, RichText, Sense, Stroke, StrokeKind, TextureHandle,
    TextureOptions, Ui, pos2, vec2,
};
use whirlpool_config::{Config, CornerAction, IconStyle, TileKind, TileSize, TileSpec};
use whirlpool_core::{
    glass::{self, Glass, Image},
    iconstyle::{self, Style},
    tiles,
};

use crate::{
    icons::{self, Icon},
    text::Texts,
    ui,
};

const INK: Color32 = Color32::from_rgb(232, 244, 248);
const MUTED: Color32 = Color32::from_rgb(140, 176, 188);
const TEAL: Color32 = Color32::from_rgb(72, 205, 220);

pub fn tile_icon(kind: TileKind) -> Icon {
    match kind {
        TileKind::Wifi => Icon::Wifi,
        TileKind::Bluetooth => Icon::Bluetooth,
        TileKind::Airplane => Icon::Airplane,
        TileKind::Cast => Icon::Cast,
        TileKind::NightLight => Icon::Moon,
        TileKind::BatterySaver => Icon::Battery,
        TileKind::Focus => Icon::Focus,
        TileKind::Display => Icon::Display,
        TileKind::Hotspot => Icon::Hotspot,
        TileKind::Sound | TileKind::Unknown => Icon::Speaker,
    }
}

pub fn tile_label(t: &Texts, kind: TileKind) -> &str {
    t.get(match kind {
        TileKind::Wifi => "tileWifi",
        TileKind::Bluetooth => "tileBluetooth",
        TileKind::Airplane => "tileAirplane",
        TileKind::Cast => "tileCast",
        TileKind::NightLight => "tileNightLight",
        TileKind::BatterySaver => "tileBatterySaver",
        TileKind::Focus => "tileFocus",
        TileKind::Display => "tileDisplay",
        TileKind::Hotspot => "tileHotspot",
        TileKind::Sound | TileKind::Unknown => "tileSound",
    })
}

fn stage(ui: &mut Ui, height: f32) -> Rect {
    let width = ui.available_width().min(520.0);
    let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same(14), Color32::from_rgb(8, 22, 38));
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(14),
        Stroke::new(1.0, Color32::from_white_alpha(18)),
        StrokeKind::Inside,
    );
    rect
}

fn hint(ui: &mut Ui, text: &str) {
    let color = ui.visuals().weak_text_color();
    ui.label(RichText::new(text).size(12.0).color(color));
    ui.add_space(8.0);
}

// ---------------------------------------------------------------- quick settings tiles

fn paint_tile(
    painter: &egui::Painter,
    rect: Rect,
    kind: TileKind,
    label: &str,
    picked: bool,
    lift: f32,
) {
    if lift > 0.0 {
        for spread in [10.0, 5.0] {
            painter.rect_filled(
                rect.expand(spread * 0.4).translate(vec2(0.0, spread * 0.5)),
                CornerRadius::same(18),
                Color32::from_black_alpha((26.0 * lift) as u8),
            );
        }
    }
    painter.rect_filled(
        rect,
        CornerRadius::same(14),
        if picked {
            Color32::from_rgb(20, 74, 88)
        } else {
            Color32::from_rgb(20, 40, 58)
        },
    );
    painter.rect_stroke(
        rect,
        CornerRadius::same(14),
        Stroke::new(
            if picked { 1.5 } else { 1.0 },
            if picked {
                TEAL
            } else {
                Color32::from_white_alpha(22)
            },
        ),
        StrokeKind::Inside,
    );
    let badge = Rect::from_center_size(pos2(rect.left() + 28.0, rect.center().y), vec2(32.0, 32.0));
    painter.circle_filled(badge.center(), 16.0, Color32::from_white_alpha(22));
    icons::paint(painter, badge.shrink(5.0), tile_icon(kind), INK);
    painter.text(
        pos2(rect.left() + 52.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(13.0),
        INK,
    );
}

/// The quick settings grid as an editor: drag to reorder, pick a tile to resize or remove it, add
/// the tiles that are missing. The grid is also the preview: the panel lays out the same way.
pub fn tiles_card(ui: &mut Ui, t: &Texts, config: &mut Config, animate: bool) {
    ui::card(ui, Some(t.get("groupTiles")), |ui| {
        hint(ui, t.get("tilesHint"));
        let ctx = ui.ctx().clone();
        let id = Id::new("quick-tiles-editor");
        let count = config.quick_tiles.len();
        let mut selected: Option<usize> = ui
            .data(|d| d.get_temp::<Option<usize>>(id.with("sel")))
            .flatten()
            .filter(|i| *i < count);
        let mut dragging: Option<usize> = ui
            .data(|d| d.get_temp::<Option<usize>>(id.with("drag")))
            .flatten()
            .filter(|i| *i < count);

        let width = ui.available_width().min(372.0);
        let (gap, cell_h) = (8.0, 54.0);
        let cell_w = ((width - gap) / 2.0).floor();
        let spans: Vec<usize> = config.quick_tiles.iter().map(|tile| tile.span()).collect();
        let rows = tiles::rows(&spans) + usize::from(dragging.is_some());
        let height = (rows.max(1) as f32) * (cell_h + gap) - gap;
        let (area, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        let pointer = ui.input(|i| i.pointer.interact_pos());

        let mut order: Vec<usize> = (0..count).collect();
        if let (Some(from), Some(p)) = (dragging, pointer) {
            let local = p - area.min;
            let from_pos = order.iter().position(|&i| i == from).unwrap_or(0);
            if let Some(to) = tiles::drop_index(&spans, local.x, local.y, cell_w, cell_h, gap) {
                tiles::move_item(&mut order, from_pos, to);
            }
        }
        let shown: Vec<usize> = order.iter().map(|&i| spans[i]).collect();
        let slots = tiles::pack(&shown);
        let mut stopped = false;
        let mut lifted: Option<(usize, Rect)> = None;
        for (position, &original) in order.iter().enumerate() {
            let (x, y, w, h) = tiles::slot_rect(&slots[position], cell_w, cell_h, gap);
            let target = Rect::from_min_size(area.min + vec2(x, y), vec2(w, h));
            let tid = id.with(("tile", original));
            let rect = if animate {
                Rect::from_min_size(
                    pos2(
                        ctx.animate_value_with_time(tid.with("x"), target.min.x, 0.14),
                        ctx.animate_value_with_time(tid.with("y"), target.min.y, 0.14),
                    ),
                    target.size(),
                )
            } else {
                target
            };
            let response = ui
                .interact(rect, tid, Sense::click_and_drag())
                .on_hover_cursor(egui::CursorIcon::Grab);
            if response.drag_started() {
                dragging = Some(original);
                selected = Some(original);
            } else if response.clicked() {
                selected = if selected == Some(original) {
                    None
                } else {
                    Some(original)
                };
            }
            if dragging == Some(original) {
                stopped |= response.drag_stopped();
                if let Some(p) = pointer {
                    lifted = Some((original, Rect::from_center_size(p, rect.size())));
                }
                // The slot it will land in stays visible as an outline.
                ui.painter().rect_stroke(
                    rect,
                    CornerRadius::same(14),
                    Stroke::new(1.5, Color32::from_rgba_unmultiplied(72, 205, 220, 110)),
                    StrokeKind::Inside,
                );
            } else {
                let kind = config.quick_tiles[original].kind;
                paint_tile(
                    ui.painter(),
                    rect,
                    kind,
                    tile_label(t, kind),
                    selected == Some(original),
                    0.0,
                );
            }
        }
        if let Some((original, rect)) = lifted {
            let kind = config.quick_tiles[original].kind;
            paint_tile(ui.painter(), rect, kind, tile_label(t, kind), true, 1.0);
        }
        if stopped {
            let reordered: Vec<TileSpec> = order.iter().map(|&i| config.quick_tiles[i]).collect();
            selected = dragging.and_then(|d| order.iter().position(|&i| i == d));
            config.quick_tiles = reordered;
            dragging = None;
        }

        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            if let Some(i) = selected.filter(|i| *i < config.quick_tiles.len()) {
                let spec = &mut config.quick_tiles[i];
                ui.label(RichText::new(tile_label(t, spec.kind)).strong());
                ui.selectable_value(&mut spec.size, TileSize::Small, t.get("tileSmall"));
                ui.selectable_value(&mut spec.size, TileSize::Wide, t.get("tileWide"));
                if ui.button(t.get("tileRemove")).clicked() {
                    config.quick_tiles.remove(i);
                    selected = None;
                }
            } else {
                let weak = ui.visuals().weak_text_color();
                ui.label(RichText::new(t.get("tilePick")).size(12.0).color(weak));
            }
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let used: Vec<&str> = config
                .quick_tiles
                .iter()
                .map(|tile| tile.kind.key())
                .collect();
            let all: Vec<&str> = TileKind::ALL.iter().map(|kind| kind.key()).collect();
            let free: Vec<TileKind> = tiles::missing(&all, &used)
                .into_iter()
                .filter_map(|key| TileKind::ALL.iter().copied().find(|kind| kind.key() == key))
                .collect();
            if !free.is_empty() {
                egui::ComboBox::from_id_salt("add-tile")
                    .selected_text(t.get("tileAdd"))
                    .show_ui(ui, |ui| {
                        for kind in free {
                            if ui.selectable_label(false, tile_label(t, kind)).clicked() {
                                config.quick_tiles.push(TileSpec {
                                    kind,
                                    size: TileSize::Small,
                                });
                            }
                        }
                    });
            }
            if ui.button(t.get("tileReset")).clicked() {
                config.quick_tiles = TileSpec::defaults();
                selected = None;
            }
        });
        ui.data_mut(|d| {
            d.insert_temp(id.with("sel"), selected);
            d.insert_temp(id.with("drag"), dragging);
        });
    });
}

// ---------------------------------------------------------------- hot corners

fn corner_label(t: &Texts, action: CornerAction) -> &str {
    t.get(match action {
        CornerAction::None => "cornerNone",
        CornerAction::StartGrid => "cornerStartGrid",
        CornerAction::Palette => "cornerPalette",
        CornerAction::WindowOverview => "cornerOverview",
        CornerAction::Notifications => "cornerNotifications",
        CornerAction::QuickSettings => "cornerQuick",
        CornerAction::ShowDesktop => "cornerDesktop",
        CornerAction::LockScreen => "cornerLock",
    })
}

pub fn corners_card(ui: &mut Ui, t: &Texts, config: &mut Config, animate: bool) {
    ui::card(ui, Some(t.get("groupCorners")), |ui| {
        hint(ui, t.get("cornersHint"));
        // Preview: a small screen; a pointer visits each corner and the action fires with a ripple.
        let rect = stage(ui, 148.0);
        let screen = Rect::from_center_size(rect.center(), vec2(236.0, 124.0));
        let painter = ui.painter().clone();
        painter.rect_filled(screen, CornerRadius::same(8), Color32::from_rgb(18, 52, 76));
        painter.rect_stroke(
            screen,
            CornerRadius::same(8),
            Stroke::new(1.0, Color32::from_white_alpha(40)),
            StrokeKind::Inside,
        );
        let actions = [
            config.hot_corner_top_left,
            config.hot_corner_top_right,
            config.hot_corner_bottom_left,
            config.hot_corner_bottom_right,
        ];
        let anchors = [
            (screen.left_top(), egui::Align2::LEFT_TOP),
            (screen.right_top(), egui::Align2::RIGHT_TOP),
            (screen.left_bottom(), egui::Align2::LEFT_BOTTOM),
            (screen.right_bottom(), egui::Align2::RIGHT_BOTTOM),
        ];
        let time = ui.ctx().input(|i| i.time);
        let phase = if animate { (time / 2.2) % 4.0 } else { -1.0 };
        for (i, ((point, align), action)) in anchors.iter().zip(actions).enumerate() {
            let on = action != CornerAction::None;
            let inset = vec2(
                if point.x < screen.center().x {
                    9.0
                } else {
                    -9.0
                },
                if point.y < screen.center().y {
                    9.0
                } else {
                    -9.0
                },
            );
            painter.circle_filled(
                *point + inset,
                3.5,
                if on {
                    TEAL
                } else {
                    Color32::from_white_alpha(40)
                },
            );
            if on {
                let local = (phase - i as f64).clamp(0.0, 1.0);
                if animate && local > 0.35 && local < 1.0 {
                    let k = ((local - 0.35) / 0.65) as f32;
                    painter.circle_stroke(
                        *point + inset,
                        4.0 + 22.0 * k,
                        Stroke::new(1.5, TEAL.gamma_multiply(1.0 - k)),
                    );
                }
                let chip = Rect::from_min_size(
                    pos2(
                        if point.x < screen.center().x {
                            screen.left() + 16.0
                        } else {
                            screen.right() - 16.0 - 74.0
                        },
                        if point.y < screen.center().y {
                            screen.top() + 22.0
                        } else {
                            screen.bottom() - 22.0 - 16.0
                        },
                    ),
                    vec2(74.0, 16.0),
                );
                painter.rect_filled(chip, CornerRadius::same(8), Color32::from_white_alpha(26));
                painter.text(
                    chip.center(),
                    egui::Align2::CENTER_CENTER,
                    short(corner_label(t, action), 12),
                    egui::FontId::proportional(9.5),
                    INK,
                );
            }
            let _ = align;
        }
        if actions.iter().all(|a| *a == CornerAction::None) {
            painter.text(
                screen.center(),
                egui::Align2::CENTER_CENTER,
                t.get("cornersIdle"),
                egui::FontId::proportional(12.0),
                Color32::from_white_alpha(150),
            );
        }
        if animate && phase >= 0.0 {
            let (point, _) = anchors[(phase as usize).min(3)];
            let local = (phase % 1.0) as f32;
            let inset = vec2(
                if point.x < screen.center().x {
                    9.0
                } else {
                    -9.0
                },
                if point.y < screen.center().y {
                    9.0
                } else {
                    -9.0
                },
            );
            let travel = (local / 0.35).clamp(0.0, 1.0);
            let from = screen.center();
            let at = from + ((point + inset) - from) * (travel * travel * (3.0 - 2.0 * travel));
            painter.circle_filled(at, 4.0, Color32::WHITE);
            ui.ctx().request_repaint();
        }
        ui.add_space(10.0);
        let options: Vec<(CornerAction, &str)> = CornerAction::ALL
            .iter()
            .map(|a| (*a, corner_label(t, *a)))
            .collect();
        for (label, id, value) in [
            ("cornerTopLeft", "hc-tl", &mut config.hot_corner_top_left),
            ("cornerTopRight", "hc-tr", &mut config.hot_corner_top_right),
            (
                "cornerBottomLeft",
                "hc-bl",
                &mut config.hot_corner_bottom_left,
            ),
            (
                "cornerBottomRight",
                "hc-br",
                &mut config.hot_corner_bottom_right,
            ),
        ] {
            ui::row(ui, t.get(label), None, 220.0, |ui| {
                ui::choose(ui, id, value, &options);
            });
        }
    });
}

fn short(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let mut cut: String = text.chars().take(max.saturating_sub(1)).collect();
    cut.push('…');
    cut
}

// ---------------------------------------------------------------- dock icon styles

fn sample_icon(hue: [u8; 3], px: usize) -> Vec<u8> {
    let mut rgba = vec![0u8; px * px * 4];
    let c = px as f32 / 2.0;
    for y in 0..px {
        for x in 0..px {
            let (dx, dy) = (
                (x as f32 + 0.5 - c).abs() / c,
                (y as f32 + 0.5 - c).abs() / c,
            );
            // A squircle: |x|^4 + |y|^4 < 0.86.
            let inside = dx.powi(4) + dy.powi(4);
            if inside > 0.86 {
                continue;
            }
            let edge = ((0.86 - inside) / 0.05).clamp(0.0, 1.0);
            let shade = 1.0 - 0.45 * (y as f32 / px as f32);
            let (gx, gy) = (x as f32 - c, y as f32 - c * 1.1);
            // A white drop-like glyph in the middle.
            let glyph = ((gx * gx + gy * gy).sqrt() < c * 0.36) as u8 as f32;
            let i = (y * px + x) * 4;
            for k in 0..3 {
                let base = f32::from(hue[k]) * shade;
                rgba[i + k] = (base + (250.0 - base) * glyph * 0.92) as u8;
            }
            rgba[i + 3] = (edge * 255.0) as u8;
        }
    }
    rgba
}

fn style_of(style: IconStyle) -> Style {
    match style {
        IconStyle::Default => Style::Default,
        IconStyle::Dark => Style::Dark,
        IconStyle::Clear => Style::Clear,
        IconStyle::Tinted => Style::Tinted,
        IconStyle::Squircle => Style::Squircle,
        IconStyle::Normalized => Style::Normalized,
    }
}

fn style_texture(
    ctx: &egui::Context,
    style: IconStyle,
    tint: [u8; 3],
    hue: [u8; 3],
) -> TextureHandle {
    let key = Id::new(("icon-style-texture", style as u8, tint, hue));
    if let Some(texture) = ctx.data(|d| d.get_temp::<TextureHandle>(key)) {
        return texture;
    }
    let mut rgba = sample_icon(hue, 64);
    iconstyle::apply_rgba(style_of(style), tint, &mut rgba);
    let image = if matches!(style, IconStyle::Squircle | IconStyle::Normalized) {
        // The geometric styles work on premultiplied BGRA, as the dock's icons are.
        let mut bgra = vec![0u8; rgba.len()];
        for (src, dst) in rgba.chunks_exact(4).zip(bgra.chunks_exact_mut(4)) {
            let a = u16::from(src[3]);
            let pre = |c: u8| ((u16::from(c) * a + 127) / 255) as u8;
            dst.copy_from_slice(&[pre(src[2]), pre(src[1]), pre(src[0]), src[3]]);
        }
        let shaped = iconstyle::reshape(style_of(style), &bgra, 64);
        let flipped: Vec<u8> = shaped
            .chunks_exact(4)
            .flat_map(|p| [p[2], p[1], p[0], p[3]])
            .collect();
        egui::ColorImage::from_rgba_premultiplied([64, 64], &flipped)
    } else {
        egui::ColorImage::from_rgba_unmultiplied([64, 64], &rgba)
    };
    let texture = ctx.load_texture(format!("icon-style-{key:?}"), image, TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(key, texture.clone()));
    texture
}

pub fn icon_style_card(ui: &mut Ui, t: &Texts, config: &mut Config) {
    ui::card(ui, Some(t.get("groupIconStyle")), |ui| {
        hint(ui, t.get("iconStyleHint"));
        let ctx = ui.ctx().clone();
        let width = ui.available_width().min(520.0);
        let cell = (width - 5.0 * 8.0) / 6.0;
        let (area, _) = ui.allocate_exact_size(vec2(width, cell * 0.95 + 28.0), Sense::hover());
        let hues = [[46u8, 140, 230], [244, 150, 50], [70, 190, 120]];
        let (ink, weak) = (ui.visuals().text_color(), ui.visuals().weak_text_color());
        for (n, style) in IconStyle::ALL.into_iter().enumerate() {
            let rect = Rect::from_min_size(
                area.min + vec2(n as f32 * (cell + 8.0), 0.0),
                vec2(cell, cell * 0.95),
            );
            let response = ui
                .interact(rect, Id::new(("icon-style", n)), Sense::click())
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            if response.clicked() {
                config.dock_icon_style = style;
            }
            let picked = config.dock_icon_style == style;
            let painter = ui.painter();
            // A light-to-dark wash behind, so the translucent style reads as glass.
            painter.rect_filled(
                rect,
                CornerRadius::same(14),
                Color32::from_rgb(52, 112, 150),
            );
            painter.rect_filled(
                Rect::from_min_max(pos2(rect.left(), rect.center().y), rect.max),
                CornerRadius {
                    nw: 0,
                    ne: 0,
                    sw: 14,
                    se: 14,
                },
                Color32::from_rgb(20, 40, 70),
            );
            painter.rect_stroke(
                rect,
                CornerRadius::same(14),
                Stroke::new(
                    if picked { 2.0 } else { 1.0 },
                    if picked {
                        TEAL
                    } else {
                        Color32::from_white_alpha(24)
                    },
                ),
                StrokeKind::Inside,
            );
            let size = (cell - 20.0) / 3.0;
            for (k, hue) in hues.iter().enumerate() {
                let texture = style_texture(&ctx, style, config.dock_icon_tint, *hue);
                let at = Rect::from_min_size(
                    pos2(
                        rect.left() + 8.0 + k as f32 * (size + 2.0),
                        rect.center().y - size / 2.0,
                    ),
                    vec2(size, size),
                );
                ui.painter().image(
                    texture.id(),
                    at,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
            ui.painter().text(
                pos2(rect.center().x, rect.bottom() + 14.0),
                egui::Align2::CENTER_CENTER,
                t.get(match style {
                    IconStyle::Default => "iconStyleDefault",
                    IconStyle::Dark => "iconStyleDark",
                    IconStyle::Clear => "iconStyleClear",
                    IconStyle::Tinted => "iconStyleTinted",
                    IconStyle::Squircle => "iconStyleSquircle",
                    IconStyle::Normalized => "iconStyleNormalized",
                }),
                egui::FontId::proportional(12.5),
                if picked { ink } else { weak },
            );
        }
        if config.dock_icon_style == IconStyle::Tinted {
            ui.add_space(8.0);
            ui::row(ui, t.get("iconTint"), None, 120.0, |ui| {
                egui::color_picker::color_edit_button_srgb(ui, &mut config.dock_icon_tint);
            });
        }
    });
}

// ---------------------------------------------------------------- glass

/// A colourful synthetic desktop with hard edges, so refraction at a panel's rim is easy to see.
fn demo_backdrop(w: usize, h: usize) -> Image {
    let mut img = Image::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let (fx, fy) = (x as f32 / w as f32, y as f32 / h as f32);
            let mut c = [
                30.0 + 90.0 * fx,
                90.0 + 80.0 * (1.0 - fy),
                150.0 + 70.0 * fx * (1.0 - fy),
            ];
            let stripe = ((x as f32 * 0.09 + y as f32 * 0.05).sin() > 0.82) as u8 as f32;
            let (cx, cy) = (x as f32 - w as f32 * 0.78, y as f32 - h as f32 * 0.3);
            let disc = ((cx * cx + cy * cy).sqrt() < h as f32 * 0.22) as u8 as f32;
            let (bx, by) = (x as f32 - w as f32 * 0.12, y as f32 - h as f32 * 0.72);
            let block = (bx.abs() < w as f32 * 0.07 && by.abs() < h as f32 * 0.14) as u8 as f32;
            c[0] += stripe * 90.0 + disc * 170.0 - block * 20.0;
            c[1] += stripe * 70.0 + disc * 90.0 + block * 60.0;
            c[2] += stripe * 40.0 - disc * 60.0 - block * 50.0;
            let i = (y * w + x) * 4;
            for k in 0..3 {
                img.rgba[i + k] = c[k].clamp(0.0, 255.0) as u8;
            }
            img.rgba[i + 3] = 255;
        }
    }
    img
}

pub fn glass_card(ui: &mut Ui, t: &Texts, config: &mut Config) {
    ui::card(ui, Some(t.get("groupGlass")), |ui| {
        ui::toggle_row(
            ui,
            t.get("glassOn"),
            Some(t.get("glassOnHint")),
            &mut config.glass_refraction,
        );
        ui::divider(ui);
        let mut strength = i32::from(config.glass_strength);
        ui::row(ui, t.get("glassStrength"), None, 220.0, |ui| {
            ui::slider(ui, &mut strength, 0..=100, "");
        });
        config.glass_strength = strength.clamp(0, 100) as u8;
        ui.add_space(8.0);
        let ctx = ui.ctx().clone();
        let rect = stage(ui, 168.0);
        let inner = rect.shrink(1.0);
        let (w, h) = (inner.width() as usize, inner.height() as usize);
        let key = Id::new((
            "glass-preview",
            w,
            h,
            config.glass_refraction,
            config.glass_strength,
        ));
        let textures = ctx.data(|d| d.get_temp::<(TextureHandle, TextureHandle)>(key));
        let (back, panel) = textures.unwrap_or_else(|| {
            let image = demo_backdrop(w.max(8), h.max(8));
            let to_tex = |name: &str, img: &Image| {
                ctx.load_texture(
                    name,
                    egui::ColorImage::from_rgba_unmultiplied([img.width, img.height], &img.rgba),
                    TextureOptions::LINEAR,
                )
            };
            let (pw, ph) = ((w as f32 * 0.62) as usize, (h as f32 * 0.6) as usize);
            let rect_px = ((w - pw) / 2, (h - ph) / 2, pw, ph);
            let look = if config.glass_refraction {
                Glass::from_percent(config.glass_strength, 22.0)
            } else {
                Glass::from_percent(0, 22.0)
            };
            let refracted = glass::refract(&image, rect_px, &look);
            let pair = (
                to_tex("glass-back", &image),
                to_tex("glass-panel", &refracted),
            );
            ctx.data_mut(|d| d.insert_temp(key, pair.clone()));
            pair
        });
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        ui.painter().image(back.id(), inner, uv, Color32::WHITE);
        let (pw, ph) = (w as f32 * 0.62, h as f32 * 0.6);
        let panel_rect = Rect::from_center_size(inner.center(), vec2(pw.floor(), ph.floor()));
        ui.painter()
            .image(panel.id(), panel_rect, uv, Color32::WHITE);
        ui.painter().rect_stroke(
            panel_rect,
            CornerRadius::same(22),
            Stroke::new(1.0, Color32::from_white_alpha(70)),
            StrokeKind::Inside,
        );
    });
}

// ---------------------------------------------------------------- clock

fn clock_options(config: &Config) -> whirlpool_core::clock::ClockOptions {
    use whirlpool_core::clock::{
        ClockFont, ClockOptions, ClockWeight, DateStyle, DateStyleOpt, HourCycle,
    };
    ClockOptions {
        hour: match config.menu_bar_24h {
            Some(true) => HourCycle::H24,
            Some(false) => HourCycle::H12,
            None => HourCycle::System,
        },
        seconds: config.clock_seconds,
        date: DateStyleOpt(if config.menu_bar_date {
            DateStyle::from_id(&config.clock_date)
        } else {
            DateStyle::Off
        }),
        zones: config.clock_zones.iter().take(3).cloned().collect(),
        font: ClockFont::from_id(&config.clock_font),
        weight: ClockWeight::from_id(&config.clock_weight),
    }
}

/// The clock: presets, 12 or 24 hours, seconds, date, typeface, weight and time zones.
pub fn clock_card(ui: &mut Ui, t: &Texts, config: &mut Config) {
    use whirlpool_core::clock::{ClockFont, ClockPreset, ClockWeight, DateStyle, ZONES};
    ui::card(ui, Some(t.get("groupClock")), |ui| {
        // A live preview in the chosen typeface and weight.
        let text = whirlpool_shell::system::clock_text_for(&clock_options(config));
        let family = if config.clock_font == "mono" {
            egui::FontFamily::Monospace
        } else {
            egui::FontFamily::Proportional
        };
        let size = if config.clock_weight == "semibold" {
            20.0
        } else {
            19.0
        };
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 54.0), Sense::hover());
        ui.painter()
            .rect_filled(rect, CornerRadius::same(14), Color32::from_rgb(14, 34, 46));
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::new(size, family),
            Color32::from_rgb(236, 246, 250),
        );
        ui.add_space(6.0);
        hint(ui, t.get("clockPreset"));
        let current = ClockPreset::matching(
            ClockFont::from_id(&config.clock_font),
            ClockWeight::from_id(&config.clock_weight),
            DateStyle::from_id(&config.clock_date),
            config.clock_seconds,
        );
        ui.horizontal_wrapped(|ui| {
            for preset in ClockPreset::ALL {
                let key = match preset {
                    ClockPreset::Classic => "clockPresetClassic",
                    ClockPreset::Quiet => "clockPresetQuiet",
                    ClockPreset::Bold => "clockPresetBold",
                    ClockPreset::Digital => "clockPresetDigital",
                    ClockPreset::Editorial => "clockPresetEditorial",
                    ClockPreset::Compact => "clockPresetCompact",
                };
                if ui
                    .selectable_label(current == Some(preset), t.get(key))
                    .clicked()
                {
                    let look = preset.look();
                    config.clock_font = look.font.id().to_owned();
                    config.clock_weight = look.weight.id().to_owned();
                    config.clock_date = look.date.id().to_owned();
                    config.menu_bar_date = look.date != DateStyle::Off;
                    config.clock_seconds = look.seconds;
                }
            }
        });
        ui.add_space(4.0);
        ui::divider(ui);
        ui::toggle_row(
            ui,
            t.get("clockSeconds"),
            Some(t.get("clockSecondsHint")),
            &mut config.clock_seconds,
        );
        ui::divider(ui);
        let mut date = DateStyle::from_id(&config.clock_date);
        ui::row(ui, t.get("clockDateStyle"), None, 180.0, |ui| {
            ui::choose(
                ui,
                "clock-date",
                &mut date,
                &[
                    (DateStyle::Short, t.get("clockDateShort")),
                    (DateStyle::Long, t.get("clockDateLong")),
                    (DateStyle::Numeric, t.get("clockDateNumeric")),
                ],
            );
        });
        if date != DateStyle::Off {
            config.clock_date = date.id().to_owned();
        }
        ui::divider(ui);
        let mut font = ClockFont::from_id(&config.clock_font);
        ui::row(ui, t.get("clockFont"), None, 180.0, |ui| {
            ui::choose(
                ui,
                "clock-font",
                &mut font,
                &[
                    (ClockFont::System, t.get("clockFontSystem")),
                    (ClockFont::Mono, t.get("clockFontMono")),
                    (ClockFont::Condensed, t.get("clockFontCondensed")),
                    (ClockFont::Serif, t.get("clockFontSerif")),
                ],
            );
        });
        config.clock_font = font.id().to_owned();
        ui::divider(ui);
        let mut weight = ClockWeight::from_id(&config.clock_weight);
        ui::row(ui, t.get("clockWeight"), None, 280.0, |ui| {
            ui.selectable_value(
                &mut weight,
                ClockWeight::Semibold,
                t.get("clockWeightSemibold"),
            );
            ui.selectable_value(
                &mut weight,
                ClockWeight::Regular,
                t.get("clockWeightRegular"),
            );
            ui.selectable_value(&mut weight, ClockWeight::Light, t.get("clockWeightLight"));
        });
        config.clock_weight = weight.id().to_owned();
        ui::divider(ui);
        ui::toggle_row(ui, t.get("clockPanel"), None, &mut config.clock_panel);
        ui::divider(ui);
        let mut island = config.clock_place == "island";
        ui::row(
            ui,
            t.get("clockPlace"),
            Some(t.get("clockPlaceHint")),
            220.0,
            |ui| {
                ui.selectable_value(&mut island, false, t.get("clockPlaceBar"));
                ui.selectable_value(&mut island, true, t.get("clockPlaceIsland"));
            },
        );
        config.clock_place = if island { "island" } else { "bar" }.to_owned();
        ui::divider(ui);
        ui.add_space(6.0);
        hint(ui, t.get("clockZones"));
        ui.horizontal_wrapped(|ui| {
            for zone in ZONES {
                let on = config.clock_zones.iter().any(|k| k == zone.key);
                if ui
                    .selectable_label(on, format!("{} {}", zone.tag, zone.city))
                    .clicked()
                {
                    if on {
                        config.clock_zones.retain(|k| k != zone.key);
                    } else if config.clock_zones.len() < 6 {
                        config.clock_zones.push(zone.key.to_owned());
                    }
                }
            }
        });
    });
}

// ---------------------------------------------------------------- glass material

/// The dock's glass material (frosted, liquid, matte, classic) and how far it leans toward the
/// wallpaper. Each swatch is drawn from the same numbers the dock uses.
pub fn material_card(ui: &mut Ui, t: &Texts, config: &mut Config) {
    use whirlpool_config::GlassMaterial;
    use whirlpool_core::material::Material;
    ui::card(ui, Some(t.get("groupMaterial")), |ui| {
        hint(ui, t.get("materialHint"));
        let width = ui.available_width().min(560.0);
        let cell = (width - 3.0 * 8.0) / 4.0;
        let (area, _) = ui.allocate_exact_size(vec2(width, 96.0 + 24.0), Sense::hover());
        let (ink, weak) = (ui.visuals().text_color(), ui.visuals().weak_text_color());
        for (n, setting) in GlassMaterial::ALL.into_iter().enumerate() {
            let material = match setting {
                GlassMaterial::Frosted => Material::Frosted,
                GlassMaterial::Liquid => Material::Liquid,
                GlassMaterial::Matte => Material::Matte,
                GlassMaterial::Classic => Material::Classic,
            };
            let rect = Rect::from_min_size(
                area.min + vec2(n as f32 * (cell + 8.0), 0.0),
                vec2(cell, 96.0),
            );
            let response = ui
                .interact(rect, Id::new(("material", n)), Sense::click())
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            if response.clicked() {
                config.glass_material = setting;
            }
            let picked = config.glass_material == setting;
            let painter = ui.painter();
            // A two-tone "wallpaper" so see-through and solid read differently.
            painter.rect_filled(
                rect,
                CornerRadius::same(14),
                Color32::from_rgb(214, 128, 72),
            );
            painter.rect_filled(
                Rect::from_min_max(pos2(rect.center().x, rect.top()), rect.max),
                CornerRadius {
                    nw: 0,
                    ne: 14,
                    sw: 0,
                    se: 14,
                },
                Color32::from_rgb(56, 118, 196),
            );
            let look = material.look(94);
            let pill = Rect::from_center_size(rect.center(), vec2(cell - 22.0, 34.0));
            let tint = Color32::from_rgba_unmultiplied(4, 34, 44, look.tint_alpha);
            painter.rect_filled(pill, CornerRadius::same(12), tint);
            painter.rect_filled(
                Rect::from_min_size(pill.min, vec2(pill.width(), pill.height() * 0.5)),
                CornerRadius {
                    nw: 12,
                    ne: 12,
                    sw: 0,
                    se: 0,
                },
                Color32::from_white_alpha(look.sheen_alpha / 2),
            );
            painter.rect_stroke(
                pill,
                CornerRadius::same(12),
                Stroke::new(
                    1.2,
                    Color32::from_white_alpha((150.0 * look.rim_scale).min(255.0) as u8),
                ),
                StrokeKind::Inside,
            );
            painter.rect_stroke(
                rect,
                CornerRadius::same(14),
                Stroke::new(
                    if picked { 2.0 } else { 1.0 },
                    if picked {
                        TEAL
                    } else {
                        Color32::from_white_alpha(24)
                    },
                ),
                StrokeKind::Inside,
            );
            painter.text(
                pos2(rect.center().x, rect.bottom() + 14.0),
                egui::Align2::CENTER_CENTER,
                t.get(match setting {
                    GlassMaterial::Frosted => "materialFrosted",
                    GlassMaterial::Liquid => "materialLiquid",
                    GlassMaterial::Matte => "materialMatte",
                    GlassMaterial::Classic => "materialClassic",
                }),
                egui::FontId::proportional(12.5),
                if picked { ink } else { weak },
            );
        }
        ui::divider(ui);
        let mut tint = config.glass_wallpaper_tint as i32;
        ui::row(ui, t.get("materialWallpaperTint"), None, 220.0, |ui| {
            ui::slider(ui, &mut tint, 0..=100, "");
        });
        config.glass_wallpaper_tint = tint.clamp(0, 100) as u32;
    });
}

// ---------------------------------------------------------------- focus timer and lyrics

/// A mock of the island pill with the given parts, drawn the way the bar draws it.
fn pill_preview(ui: &mut Ui, parts: &[String]) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 54.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same(14), Color32::from_rgb(14, 34, 46));
    let font = egui::FontId::proportional(13.0);
    let ink = Color32::from_rgb(236, 246, 250);
    let galleys: Vec<_> = parts
        .iter()
        .map(|part| ui.painter().layout_no_wrap(part.clone(), font.clone(), ink))
        .collect();
    let gap = 22.0;
    let text: f32 = galleys.iter().map(|g| g.size().x).sum();
    let total =
        (text + gap * galleys.len().saturating_sub(1) as f32 + 28.0).min(rect.width() - 16.0);
    let pill = Rect::from_center_size(rect.center(), vec2(total, 28.0));
    ui.painter()
        .rect_filled(pill, CornerRadius::same(14), Color32::from_rgb(4, 8, 12));
    ui.painter().rect_stroke(
        pill,
        CornerRadius::same(14),
        Stroke::new(1.0, Color32::from_white_alpha(40)),
        StrokeKind::Inside,
    );
    let mut x = pill.left() + 14.0;
    for (index, galley) in galleys.iter().enumerate() {
        if index > 0 {
            let at = x - gap / 2.0;
            ui.painter().line_segment(
                [pos2(at, pill.top() + 7.0), pos2(at, pill.bottom() - 7.0)],
                Stroke::new(1.0, Color32::from_white_alpha(40)),
            );
        }
        let width = galley.size().x;
        let y = pill.center().y - galley.size().y / 2.0;
        ui.painter().galley(pos2(x, y), galley.clone(), ink);
        x += width + gap;
    }
}

/// The focus timer: lengths and whether a running session shows beside the island.
pub fn focus_card(ui: &mut Ui, t: &Texts, config: &mut Config) {
    use whirlpool_core::pomodoro::{Plan, Timer};
    ui::card(ui, Some(t.get("groupFocus")), |ui| {
        let plan = Plan::from_minutes(
            config.focus_minutes,
            config.focus_short_minutes,
            config.focus_long_minutes,
        );
        let mut parts = Vec::new();
        if config.clock_place == "island" {
            parts.push(whirlpool_shell::system::clock_text_for(&clock_options(
                config,
            )));
        }
        parts.push(Timer::new(plan).label());
        pill_preview(ui, &parts);
        ui.add_space(6.0);
        hint(ui, t.get("focusStart"));
        ui::toggle_row(
            ui,
            t.get("focusIsland"),
            Some(t.get("focusIslandHint")),
            &mut config.focus_island,
        );
        ui::divider(ui);
        ui::row(ui, t.get("focusLength"), None, 220.0, |ui| {
            ui::slider(ui, &mut config.focus_minutes, 5..=90, " min");
        });
        ui::divider(ui);
        ui::row(ui, t.get("focusShort"), None, 220.0, |ui| {
            ui::slider(ui, &mut config.focus_short_minutes, 1..=30, " min");
        });
        ui::divider(ui);
        ui::row(ui, t.get("focusLong"), None, 220.0, |ui| {
            ui::slider(ui, &mut config.focus_long_minutes, 5..=60, " min");
        });
    });
}

/// Synced lyrics: off by default, with the exact privacy statement next to the switch.
pub fn lyrics_card(ui: &mut Ui, t: &Texts, config: &mut Config) {
    ui::card(ui, Some(t.get("groupLyrics")), |ui| {
        pill_preview(ui, &[t.get("lyricsSample").to_owned()]);
        ui.add_space(6.0);
        ui::toggle_row(
            ui,
            t.get("lyricsOn"),
            Some(t.get("lyricsOnHint")),
            &mut config.lyrics,
        );
        ui::divider(ui);
        ui.add_space(6.0);
        hint(ui, t.get("lyricsPrivacy"));
    });
}
