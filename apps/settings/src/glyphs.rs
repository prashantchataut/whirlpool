//! Small vector glyphs drawn from shapes: the icons for palette actions, files and clipboard rows, and
//! the stand-ins for windows without an icon. Drawn at any size, tinted by the caller, no assets.

use eframe::egui::{
    Color32, CornerRadius, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, pos2, vec2,
};

/// Points of an arc around `center`, angles in degrees (0 = right, 90 = down).
fn arc(center: Pos2, radius: f32, from: f32, to: f32) -> Vec<Pos2> {
    let steps = 24;
    (0..=steps)
        .map(|step| {
            let angle = (from + (to - from) * step as f32 / steps as f32).to_radians();
            pos2(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            )
        })
        .collect()
}

/// Draws the glyph called `key` inside the square `area`. Unknown keys draw nothing and return false so
/// the caller can fall back to a letter.
pub fn draw(painter: &Painter, key: &str, area: Rect, color: Color32) -> bool {
    let c = area.center();
    let u = area.width().min(area.height()) / 32.0;
    let at = |x: f32, y: f32| pos2(c.x + x * u, c.y + y * u);
    let stroke = Stroke::new((1.7 * u).max(1.2), color);
    let line = |a: Pos2, b: Pos2| {
        painter.line_segment([a, b], stroke);
    };
    let poly = |points: Vec<Pos2>| {
        painter.add(Shape::line(points, stroke));
    };
    match key {
        "settings" | "windows-settings" => {
            // Three sliders.
            for (y, knob) in [(-7.0, -3.0), (0.0, 4.0), (7.0, -1.0)] {
                line(at(-9.0, y), at(9.0, y));
                painter.circle_filled(at(knob, y), 3.0 * u, color);
            }
        }
        "start-grid" => {
            for row in 0..3 {
                for col in 0..3 {
                    let x = -8.0 + col as f32 * 8.0;
                    let y = -8.0 + row as f32 * 8.0;
                    painter.rect_filled(
                        Rect::from_center_size(at(x, y), vec2(5.0 * u, 5.0 * u)),
                        CornerRadius::same(1),
                        color,
                    );
                }
            }
        }
        "overlay" => {
            for (x, top) in [(-7.0, 0.0), (0.0, -9.0), (7.0, -4.0)] {
                painter.rect_filled(
                    Rect::from_min_max(at(x - 2.5, top), at(x + 2.5, 9.0)),
                    CornerRadius::same(1),
                    color,
                );
            }
        }
        "lock" => {
            painter.rect_stroke(
                Rect::from_min_max(at(-8.0, -1.0), at(8.0, 10.0)),
                CornerRadius::same(2),
                stroke,
                StrokeKind::Middle,
            );
            poly(arc(at(0.0, -2.0), 5.5 * u, 180.0, 360.0));
            painter.circle_filled(at(0.0, 4.5), 1.6 * u, color);
        }
        "signout" => {
            poly(vec![
                at(-2.0, -9.0),
                at(-9.0, -9.0),
                at(-9.0, 9.0),
                at(-2.0, 9.0),
            ]);
            line(at(-2.0, 0.0), at(9.0, 0.0));
            poly(vec![at(5.0, -4.0), at(9.0, 0.0), at(5.0, 4.0)]);
        }
        "task-manager" => {
            poly(vec![
                at(-10.0, 2.0),
                at(-5.0, 2.0),
                at(-2.5, -8.0),
                at(1.5, 9.0),
                at(4.5, -2.0),
                at(6.0, 2.0),
                at(10.0, 2.0),
            ]);
        }
        "restore" => {
            poly(arc(at(0.0, 0.0), 8.5 * u, -60.0, 220.0));
            poly(vec![at(-9.5, -8.5), at(-8.0, -2.5), at(-2.5, -5.5)]);
        }
        "quit" => {
            poly(arc(at(0.0, 1.0), 8.5 * u, -60.0, 240.0));
            line(at(0.0, -10.0), at(0.0, 0.0));
        }
        "file" => {
            poly(vec![
                at(-6.5, -10.0),
                at(2.0, -10.0),
                at(7.0, -5.0),
                at(7.0, 10.0),
                at(-6.5, 10.0),
                at(-6.5, -10.0),
            ]);
            poly(vec![at(2.0, -10.0), at(2.0, -5.0), at(7.0, -5.0)]);
            line(at(-3.0, 2.0), at(3.5, 2.0));
            line(at(-3.0, 6.0), at(3.5, 6.0));
        }
        "clip" => {
            painter.rect_stroke(
                Rect::from_min_max(at(-7.5, -7.0), at(7.5, 10.0)),
                CornerRadius::same(2),
                stroke,
                StrokeKind::Middle,
            );
            painter.rect_filled(
                Rect::from_min_max(at(-3.5, -10.0), at(3.5, -5.0)),
                CornerRadius::same(2),
                color,
            );
            line(at(-3.5, 1.0), at(3.5, 1.0));
            line(at(-3.5, 5.0), at(1.5, 5.0));
        }
        "calc" => {
            line(at(-8.0, -3.0), at(8.0, -3.0));
            line(at(-8.0, 3.0), at(8.0, 3.0));
            line(at(-2.5, -9.0), at(-4.5, 9.0));
            line(at(4.5, -9.0), at(2.5, 9.0));
        }
        "window" | "overview" => {
            painter.rect_stroke(
                Rect::from_min_max(at(-10.0, -8.0), at(10.0, 8.0)),
                CornerRadius::same(2),
                stroke,
                StrokeKind::Middle,
            );
            line(at(-10.0, -3.0), at(10.0, -3.0));
        }
        _ => return false,
    }
    true
}

/// The rounded tile behind a glyph: a faint accent wash so the glyph reads on glass.
pub fn tile(painter: &Painter, area: Rect, accent: Color32) {
    let wash =
        |alpha: u8| Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), alpha);
    painter.rect_filled(area, CornerRadius::same(9), wash(34));
    painter.rect_stroke(
        area,
        CornerRadius::same(9),
        Stroke::new(1.0, wash(60)),
        StrokeKind::Inside,
    );
}
