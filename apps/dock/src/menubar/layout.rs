//! Island geometry: where the triggers sit and how wide a panel opens. Pure arithmetic.

use super::*;

#[derive(Clone, Copy)]
pub(super) struct IslandGeometry {
    pub(super) left: f32,
    pub(super) top: f32,
    pub(super) width: f32,
    pub(super) scale: f32,
    pub(super) light: bool,
    /// Corner radius of the opened panel, in logical pixels (set by the island preset).
    pub(super) corner: f32,
}

impl IslandGeometry {
    pub(super) fn new(span: Span, top: f32, look: &Look) -> Self {
        Self {
            left: span.left,
            top,
            width: span.width(),
            scale: look.scale,
            light: look.light,
            corner: island_metrics(&look.widgets).corner_radius,
        }
    }

    pub(super) fn hit(
        self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        action: IslandAction,
    ) -> IslandHit {
        let left = self.left + x;
        let top = self.top + y;
        IslandHit {
            bounds: (left, top, left + width, top + height),
            action,
        }
    }

    pub(super) fn px(self, logical: f32) -> f32 {
        (logical * self.scale).round()
    }
    pub(super) fn ink(self) -> Ink {
        if self.light { Ink::Dark } else { Ink::Light }
    }
    pub(super) fn control_fill(self) -> Color {
        if self.light {
            rgba(0, 0, 0, 18)
        } else {
            rgba(255, 255, 255, 27)
        }
    }
    pub(super) fn border(self) -> Color {
        if self.light {
            rgba(35, 57, 83, 31)
        } else {
            rgba(223, 237, 255, 37)
        }
    }
}

pub(super) fn toggle_panel(open: Option<Island>, clicked: Island) -> Option<Island> {
    (open != Some(clicked)).then_some(clicked)
}

pub(super) fn available_panel(open: Option<Island>, music: bool, usage: bool) -> Option<Island> {
    open.filter(|island| match island {
        Island::Music => music,
        Island::Usage => usage,
    })
}

/// Sizes of the island and its panel for the chosen preset, in logical pixels.
pub(super) fn island_metrics(widgets: &MenuBarWidgets) -> whirlpool_core::island::Metrics {
    use whirlpool_config::IslandPreset;
    use whirlpool_core::island::Preset;
    match widgets.island_preset {
        IslandPreset::Compact => Preset::Compact,
        IslandPreset::Classic => Preset::Classic,
        IslandPreset::Wide => Preset::Wide,
        IslandPreset::Expressive => Preset::Expressive,
    }
    .metrics()
}

/// The shipped (Classic) trigger layout, kept as the reference the tests compare presets against.
#[cfg(test)]
pub(super) fn trigger_layout(
    bounds: Span,
    center: f32,
    scale: f32,
    music: bool,
    providers: usize,
) -> [(Island, Option<Span>); 2] {
    let classic = whirlpool_core::island::Preset::Classic.metrics();
    trigger_layout_for(
        classic.single_width,
        bounds,
        center,
        scale,
        music,
        providers,
    )
}

pub(super) fn trigger_layout_for(
    single_width: f32,
    bounds: Span,
    center: f32,
    scale: f32,
    music: bool,
    providers: usize,
) -> [(Island, Option<Span>); 2] {
    let px = |v: f32| (v * scale).round();
    let mut result = [(Island::Music, None), (Island::Usage, None)];
    let gap = if music && providers > 0 { px(8.0) } else { 0.0 };
    let music_width = if music { px(214.0) } else { 0.0 };
    let usage_width = match providers {
        0 => 0.0,
        1 => px(single_width),
        // Classic (145 for one provider) keeps its shipped 134 per provider; the other presets scale it.
        count => px(single_width * (134.0 / 145.0) * count as f32),
    };
    let total = music_width + usage_width;
    let available = (bounds.width() - gap).min(total);
    if total == 0.0 || available < px(if gap > 0.0 { 85.0 } else { 42.0 }) {
        return result;
    }
    let music_width = if gap > 0.0 {
        (available * music_width / total)
            .round()
            .clamp(px(42.0), available - px(42.0))
    } else if music {
        available
    } else {
        0.0
    };
    let left = (center - (available + gap) / 2.0)
        .clamp(
            bounds.left,
            (bounds.right - available - gap).max(bounds.left),
        )
        .round();
    if music {
        result[0].1 = Some(Span {
            left,
            right: left + music_width,
        });
    }
    if providers > 0 {
        result[1].1 = Some(Span {
            left: left + music_width + gap,
            right: left + available + gap,
        });
    }
    result
}

pub(super) fn bubble_trigger_layout(
    bounds: Span,
    scale: f32,
    music_width: Option<f32>,
    providers: usize,
) -> [(Island, Option<Span>); 2] {
    let px = |v: f32| (v * scale).round();
    let mut result = [(Island::Music, None), (Island::Usage, None)];
    let music = music_width.is_some();
    let gap = if music && providers > 0 { px(8.0) } else { 0.0 };
    let music_width = music_width
        .map(|width| width.clamp(px(36.0), px(BUBBLE_MUSIC_WIDTH)))
        .unwrap_or(0.0);
    let usage_width = if providers > 0 {
        px(BUBBLE_USAGE_PADDING + BUBBLE_PROVIDER_WIDTH * providers as f32)
    } else {
        0.0
    };
    let total = music_width + usage_width;
    let available = (bounds.width() - gap).min(total);
    if total == 0.0 || available < px(if gap > 0.0 { 72.0 } else { 36.0 }) {
        return result;
    }
    let music_width = if gap > 0.0 {
        (available * music_width / total)
            .round()
            .clamp(px(36.0), available - px(36.0))
    } else if music {
        available
    } else {
        0.0
    };
    let left = (bounds.right - available - gap).max(bounds.left).round();
    if music {
        result[0].1 = Some(Span {
            left,
            right: left + music_width,
        });
    }
    if providers > 0 {
        result[1].1 = Some(Span {
            left: left + music_width + gap,
            right: left + available + gap,
        });
    }
    result
}

pub(super) fn panel_span(trigger: Span, screen_width: f32, desired: f32) -> Span {
    let width = desired.max(trigger.width()).min(screen_width);
    let left = ((trigger.left + trigger.right - width) / 2.0)
        .clamp(0.0, screen_width - width)
        .round();
    Span {
        left,
        right: left + width,
    }
}

pub(super) fn bubble_panel_span(anchor: Span, screen_width: f32, desired: f32) -> Span {
    let width = desired.max(anchor.width()).min(screen_width);
    let left = (anchor.right - width)
        .clamp(0.0, screen_width - width)
        .round();
    Span {
        left,
        right: left + width,
    }
}
