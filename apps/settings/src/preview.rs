//! Live previews drawn with the painter, for Settings cards that change motion or layout.
//! They are small scenes on a loop, driven by the clock; with reduced motion they show the end state.

use eframe::egui::{Color32, CornerRadius, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

const ACCENT: Color32 = Color32::from_rgb(72, 205, 220);

fn stage(ui: &mut Ui, height: f32) -> (Rect, f64) {
    let width = ui.available_width().min(520.0);
    let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(14), Color32::from_rgb(8, 22, 38));
    // A soft water-coloured wash so the stage reads as a desktop, not a grey box.
    painter.rect_filled(
        Rect::from_min_max(rect.min, pos2(rect.max.x, rect.center().y)),
        CornerRadius {
            nw: 14,
            ne: 14,
            sw: 0,
            se: 0,
        },
        Color32::from_rgba_unmultiplied(40, 120, 170, 26),
    );
    painter.rect_stroke(
        rect,
        CornerRadius::same(14),
        Stroke::new(1.0, Color32::from_white_alpha(18)),
        StrokeKind::Inside,
    );
    (rect, ui.ctx().input(|input| input.time))
}

fn smooth(x: f64) -> f32 {
    let x = x.clamp(0.0, 1.0);
    (x * x * (3.0 - 2.0 * x)) as f32
}

/// Start grid: icons rise in one after another, a highlight walks the grid like arrow keys would.
pub fn start_grid(ui: &mut Ui, animate: bool) {
    let (rect, time) = stage(ui, 132.0);
    let painter = ui.painter().clone();
    let search =
        Rect::from_center_size(pos2(rect.center().x, rect.top() + 20.0), vec2(150.0, 20.0));
    painter.rect_filled(
        search,
        CornerRadius::same(10),
        Color32::from_white_alpha(22),
    );
    painter.circle_stroke(
        pos2(search.left() + 12.0, search.center().y),
        4.0,
        Stroke::new(1.3, Color32::from_white_alpha(120)),
    );
    let (columns, rows) = (6usize, 2usize);
    let cell = 44.0;
    let origin = pos2(
        rect.center().x - columns as f32 * cell / 2.0,
        rect.top() + 46.0,
    );
    let cycle = if animate { time % 5.0 } else { 99.0 };
    let selected = if animate {
        ((time * 1.1) as usize) % (columns * rows)
    } else {
        0
    };
    for index in 0..columns * rows {
        let (column, row) = (index % columns, index / columns);
        let appear = smooth((cycle - index as f64 * 0.05) / 0.35);
        let center = pos2(
            origin.x + column as f32 * cell + cell / 2.0,
            origin.y + row as f32 * 38.0 + 17.0 + (1.0 - appear) * 8.0,
        );
        let size = 26.0 * (0.7 + 0.3 * appear);
        let tint = [
            Color32::from_rgb(64, 190, 210),
            Color32::from_rgb(70, 140, 230),
            Color32::from_rgb(110, 210, 190),
            Color32::from_rgb(120, 130, 235),
        ][index % 4];
        let icon = Rect::from_center_size(center, vec2(size, size));
        if index == selected && cycle > 1.2 {
            painter.rect_filled(
                icon.expand(6.0),
                CornerRadius::same(12),
                Color32::from_rgba_unmultiplied(72, 205, 220, 46),
            );
        }
        painter.rect_filled(
            icon,
            CornerRadius::same(8),
            Color32::from_rgba_unmultiplied(tint.r(), tint.g(), tint.b(), (230.0 * appear) as u8),
        );
    }
    if animate {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(16));
    }
}

/// Drag motion: lift with a shadow, a snap preview at the screen edge, a critically damped settle.
pub fn drag_motion(ui: &mut Ui, animate: bool, enabled: bool) {
    let (rect, time) = stage(ui, 132.0);
    let painter = ui.painter().clone();
    let window = vec2(84.0, 56.0);
    let half = Rect::from_min_max(
        pos2(rect.left() + 8.0, rect.top() + 8.0),
        pos2(rect.center().x - 2.0, rect.bottom() - 8.0),
    );
    let rest_left = rect.center().x + 34.0;
    let start = pos2(rest_left, rect.center().y - 4.0);
    let snapped = pos2(half.center().x, half.center().y);
    let t = if animate { time % 5.0 } else { 0.0 };

    // Where the window is, how lifted it is, how visible the snap preview is.
    let (position, lift, preview) = if !animate || !enabled {
        (start, 0.0f32, 0.0f32)
    } else if t < 1.3 {
        let k = smooth(t / 1.3);
        (
            start + (pos2(half.right() - 6.0, half.center().y) - start) * k,
            smooth(t / 0.25),
            0.0,
        )
    } else if t < 2.4 {
        (
            pos2(half.right() - 6.0, half.center().y),
            1.0,
            smooth((t - 1.3) / 0.25),
        )
    } else {
        // Released at 2.4 s: critically damped spring toward the snapped rectangle, never overshooting.
        let since = (t - 2.4) as f32;
        let w = 15.0f32;
        let remaining = (1.0 + w * since) * (-w * since).exp();
        let from = pos2(half.right() - 6.0, half.center().y);
        let at = snapped + (from - snapped) * remaining;
        (
            at,
            (1.0 - smooth((t - 2.4) / 0.18)).max(0.0),
            (1.0 - smooth((t - 2.4) / 0.3)).max(0.0),
        )
    };
    if preview > 0.0 {
        painter.rect_filled(
            half,
            CornerRadius::same(10),
            Color32::from_rgba_unmultiplied(72, 205, 220, (46.0 * preview) as u8),
        );
        painter.rect_stroke(
            half,
            CornerRadius::same(10),
            Stroke::new(
                1.5,
                Color32::from_rgba_unmultiplied(72, 205, 220, (170.0 * preview) as u8),
            ),
            StrokeKind::Inside,
        );
    }
    let frame_size = if t >= 2.4 && animate && enabled {
        let settle = ((t - 2.4) as f32 / 0.5).clamp(0.0, 1.0);
        window + (half.size() - window) * smooth(f64::from(settle))
    } else {
        window
    };
    let body = Rect::from_center_size(position - vec2(0.0, lift * 4.0), frame_size);
    if enabled {
        for (spread, alpha) in [(14.0, 16.0), (8.0, 22.0), (3.0, 30.0)] {
            painter.rect_filled(
                body.expand(spread * lift)
                    .translate(vec2(0.0, 4.0 + 8.0 * lift)),
                CornerRadius::same(12),
                Color32::from_black_alpha((alpha * lift) as u8),
            );
        }
    }
    painter.rect_filled(
        body,
        CornerRadius::same(9),
        Color32::from_rgb(222, 236, 244),
    );
    painter.rect_filled(
        Rect::from_min_size(body.min, vec2(body.width(), 12.0)),
        CornerRadius {
            nw: 9,
            ne: 9,
            sw: 0,
            se: 0,
        },
        Color32::from_rgb(160, 200, 220),
    );
    for index in 0..3 {
        painter.circle_filled(
            pos2(body.left() + 8.0 + index as f32 * 8.0, body.top() + 6.0),
            2.0,
            if index == 0 {
                ACCENT
            } else {
                Color32::from_white_alpha(170)
            },
        );
    }
    if animate {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(16));
    }
}
