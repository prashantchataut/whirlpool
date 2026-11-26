//! Vector icons drawn with the painter. One stroke weight, one 24-unit grid,
//! so the sidebar, the tiles and the buttons all look cut from the same sheet.

use eframe::egui::{
    Color32, CornerRadius, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, pos2, vec2,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Dock,
    Palette,
    Window,
    Bar,
    System,
    Info,
    Clock,
    Search,
    Wifi,
    Bluetooth,
    Airplane,
    Cast,
    Moon,
    Battery,
    Speaker,
    Muted,
    Bolt,
    Focus,
    Display,
    Hotspot,
}

/// Points on a circular arc, angles in degrees (0 = right, clockwise).
fn arc(center: Pos2, radius: f32, from: f32, to: f32) -> Vec<Pos2> {
    let steps = 20;
    (0..=steps)
        .map(|i| {
            let t = from + (to - from) * i as f32 / steps as f32;
            let r = t.to_radians();
            pos2(center.x + radius * r.cos(), center.y + radius * r.sin())
        })
        .collect()
}

/// Draws `icon` centered in `rect`. The art is laid out on a 24 x 24 grid.
pub fn paint(painter: &Painter, rect: Rect, icon: Icon, color: Color32) {
    let u = rect.width().min(rect.height()) / 24.0;
    let o = rect.center() - vec2(12.0 * u, 12.0 * u);
    let p = |x: f32, y: f32| pos2(o.x + x * u, o.y + y * u);
    let stroke = Stroke::new((1.7 * u).max(1.2), color);
    let line = |points: Vec<Pos2>| {
        painter.add(Shape::line(points, stroke));
    };
    let boxed = |min: Pos2, max: Pos2, radius: f32| {
        painter.rect_stroke(
            Rect::from_min_max(min, max),
            CornerRadius::same((radius * u).round() as u8),
            stroke,
            StrokeKind::Middle,
        );
    };
    match icon {
        Icon::Dock => {
            boxed(p(3.0, 6.0), p(21.0, 18.0), 4.0);
            for x in [8.0, 12.0, 16.0] {
                painter.circle_filled(p(x, 12.0), 1.3 * u, color);
            }
        }
        Icon::Palette => {
            boxed(p(3.5, 4.5), p(20.5, 19.5), 4.0);
            line(vec![
                p(3.5, 15.0),
                p(8.5, 10.5),
                p(12.5, 14.0),
                p(15.5, 11.5),
                p(20.5, 16.0),
            ]);
            painter.circle_filled(p(16.5, 8.2), 1.5 * u, color);
        }
        Icon::Window => {
            boxed(p(3.5, 4.5), p(20.5, 19.5), 3.0);
            line(vec![p(3.5, 9.0), p(20.5, 9.0)]);
        }
        Icon::Bar => {
            boxed(p(3.0, 5.0), p(21.0, 19.0), 3.0);
            boxed(p(5.5, 7.5), p(18.5, 10.5), 1.5);
        }
        Icon::System => {
            for (y, x) in [(7.0, 15.0), (12.0, 8.0), (17.0, 13.0)] {
                line(vec![p(4.0, y), p(20.0, y)]);
                painter.circle_filled(p(x, y), 2.4 * u, color);
            }
        }
        Icon::Info => {
            painter.circle_stroke(p(12.0, 12.0), 8.5 * u, stroke);
            line(vec![p(12.0, 11.0), p(12.0, 16.5)]);
            painter.circle_filled(p(12.0, 7.8), 1.1 * u, color);
        }
        Icon::Clock => {
            painter.circle_stroke(p(12.0, 12.0), 8.5 * u, stroke);
            line(vec![p(12.0, 7.0), p(12.0, 12.0), p(15.5, 14.0)]);
        }
        Icon::Search => {
            painter.circle_stroke(p(10.5, 10.5), 6.0 * u, stroke);
            line(vec![p(15.0, 15.0), p(20.0, 20.0)]);
        }
        Icon::Wifi => {
            let c = p(12.0, 19.0);
            for r in [5.0, 9.5, 14.0] {
                line(arc(c, r * u, 233.0, 307.0));
            }
            painter.circle_filled(c, 1.5 * u, color);
        }
        Icon::Bluetooth => {
            line(vec![
                p(7.0, 8.0),
                p(17.0, 16.0),
                p(12.0, 20.5),
                p(12.0, 3.5),
                p(17.0, 8.0),
                p(7.0, 16.0),
            ]);
        }
        Icon::Airplane => {
            line(vec![
                p(12.0, 3.0),
                p(14.0, 9.5),
                p(21.0, 14.0),
                p(21.0, 16.0),
                p(14.0, 14.0),
                p(13.0, 19.0),
                p(15.5, 21.0),
                p(15.5, 22.0),
                p(12.0, 21.0),
                p(8.5, 22.0),
                p(8.5, 21.0),
                p(11.0, 19.0),
                p(10.0, 14.0),
                p(3.0, 16.0),
                p(3.0, 14.0),
                p(10.0, 9.5),
                p(12.0, 3.0),
            ]);
        }
        Icon::Cast => {
            line(vec![
                p(4.0, 7.0),
                p(4.0, 5.5),
                p(20.0, 5.5),
                p(20.0, 18.5),
                p(14.5, 18.5),
            ]);
            let c = p(3.0, 21.0);
            for r in [5.0, 9.0, 13.0] {
                line(arc(c, r * u, 270.0, 360.0));
            }
        }
        Icon::Moon => {
            // Outer circle C1 minus a bite from circle C2: the two arcs meet
            // where the circles cross.
            let (c1, r1) = ((11.5_f32, 12.5_f32), 8.0_f32);
            let (c2, r2) = ((16.0_f32, 8.5_f32), 6.6_f32);
            let (dx, dy) = (c2.0 - c1.0, c2.1 - c1.1);
            let d = (dx * dx + dy * dy).sqrt();
            let a = (r1 * r1 - r2 * r2 + d * d) / (2.0 * d);
            let h = (r1 * r1 - a * a).max(0.0).sqrt();
            let (mx, my) = (c1.0 + a * dx / d, c1.1 + a * dy / d);
            let hit = |sign: f32| (mx + sign * h * dy / d, my - sign * h * dx / d);
            let (i1, i2) = (hit(1.0), hit(-1.0));
            let angle = |c: (f32, f32), q: (f32, f32)| (q.1 - c.1).atan2(q.0 - c.0).to_degrees();
            let (mut start, mut end) = (angle(c1, i1), angle(c1, i2));
            if end < start {
                end += 360.0;
            }
            // The long way round, away from the bite.
            if end - start < 180.0 {
                std::mem::swap(&mut start, &mut end);
                start -= 360.0;
            }
            let mut pts = arc(p(c1.0, c1.1), r1 * u, start, end);
            let (mut s2, mut e2) = (angle(c2, i2), angle(c2, i1));
            if e2 < s2 {
                e2 += 360.0;
            }
            if e2 - s2 > 180.0 {
                std::mem::swap(&mut s2, &mut e2);
                s2 -= 360.0;
            }
            pts.extend(arc(p(c2.0, c2.1), r2 * u, s2, e2));
            line(pts);
        }
        Icon::Battery => {
            boxed(p(2.5, 7.5), p(19.0, 16.5), 2.5);
            line(vec![p(21.5, 10.5), p(21.5, 13.5)]);
            painter.rect_filled(
                Rect::from_min_max(p(5.0, 10.0), p(11.5, 14.0)),
                CornerRadius::same(u as u8),
                color,
            );
        }
        Icon::Speaker | Icon::Muted => {
            line(vec![
                p(4.0, 9.5),
                p(8.0, 9.5),
                p(13.0, 5.5),
                p(13.0, 18.5),
                p(8.0, 14.5),
                p(4.0, 14.5),
                p(4.0, 9.5),
            ]);
            if icon == Icon::Speaker {
                line(arc(p(13.0, 12.0), 4.5 * u, -45.0, 45.0));
                line(arc(p(13.0, 12.0), 8.5 * u, -45.0, 45.0));
            } else {
                line(vec![p(16.5, 9.5), p(21.0, 14.5)]);
                line(vec![p(21.0, 9.5), p(16.5, 14.5)]);
            }
        }
        Icon::Focus => {
            painter.circle_stroke(p(12.0, 12.0), 8.0 * u, stroke);
            painter.circle_filled(p(12.0, 12.0), 3.2 * u, color);
        }
        Icon::Display => {
            boxed(p(3.0, 5.0), p(21.0, 16.0), 2.5);
            line(vec![p(9.0, 20.0), p(15.0, 20.0)]);
            line(vec![p(12.0, 16.0), p(12.0, 20.0)]);
        }
        Icon::Hotspot => {
            painter.circle_filled(p(12.0, 13.0), 2.0 * u, color);
            line(arc(p(12.0, 13.0), 6.0 * u, 215.0, 325.0));
            line(arc(p(12.0, 13.0), 10.0 * u, 215.0, 325.0));
        }
        Icon::Bolt => {
            line(vec![
                p(13.5, 3.0),
                p(6.0, 13.5),
                p(11.5, 13.5),
                p(10.0, 21.0),
                p(18.0, 10.0),
                p(12.5, 10.0),
                p(13.5, 3.0),
            ]);
        }
    }
}
