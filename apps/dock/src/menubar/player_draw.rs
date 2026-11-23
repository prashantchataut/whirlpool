//! Drawing of the music island: compact player and bubble player.

use super::*;

pub(super) fn track_title(track: &Track) -> &str {
    if track.title.trim().is_empty() {
        strings::text().media_title
    } else {
        &track.title
    }
}

pub(super) fn draw_compact_player(
    gfx: &Gfx,
    layers: &VisualCollection,
    geometry: IslandGeometry,
    width: f32,
    height: f32,
    track: &Track,
    cover: Option<&CompositionSurfaceBrush>,
) -> Result<()> {
    let c = &gfx.compositor;
    let px = |v| geometry.px(v);
    let ink = geometry.ink();
    let side = px(18.0).min(height).min(width);
    let x = px(23.0) + side;
    let show_title = width > x + px(18.0);
    let note_x = if show_title {
        px(18.0)
    } else {
        (width - side) / 2.0
    };
    if let Some(cover) = cover {
        add_visual(
            c,
            layers,
            &cover.cast()?,
            note_x,
            (height - side) / 2.0,
            side,
            side,
        )?
        .SetOpacity(if track.playing { 1.0 } else { 0.7 })?;
    }
    if show_title {
        let label = gfx.text_brush_fitted(
            track_title(track),
            geometry.scale,
            11.5,
            true,
            ink,
            width - x - px(13.0),
        )?;
        add_visual(
            c,
            layers,
            &label.brush.cast()?,
            x,
            (height - label.height) / 2.0,
            label.width,
            label.height,
        )?;
    }
    Ok(())
}

pub(super) struct BubbleMusic {
    pub(super) artwork_size: f32,
    pub(super) label: LabelBrush,
    pub(super) width: f32,
}

pub(super) fn prepare_bubble_music(gfx: &Gfx, look: &Look, track: &Track) -> Result<BubbleMusic> {
    let px = |v: f32| (v * look.scale).round();
    let ink = if look.light { Ink::Dark } else { Ink::Light };
    let artwork_size = px(18.0);
    let left_pad = px(10.0);
    let gap = px(6.0);
    let right_pad = px(8.0);
    let max_text =
        (px(BUBBLE_MUSIC_WIDTH) - left_pad - artwork_size - gap - right_pad).max(px(18.0));
    let label = gfx.text_brush_fitted(track_title(track), look.scale, 11.5, true, ink, max_text)?;
    let width = (left_pad + artwork_size + gap + label.width + right_pad)
        .min(px(BUBBLE_MUSIC_WIDTH))
        .ceil();
    Ok(BubbleMusic {
        artwork_size,
        label,
        width,
    })
}

pub(super) fn draw_bubble_player(
    gfx: &Gfx,
    layers: &VisualCollection,
    geometry: IslandGeometry,
    height: f32,
    track: &Track,
    prepared: &BubbleMusic,
    cover: Option<&CompositionSurfaceBrush>,
) -> Result<()> {
    let width = geometry.width;
    let c = &gfx.compositor;
    let px = |v| geometry.px(v);
    let ink = geometry.ink();
    let side = prepared.artwork_size.min(height).min(width);
    let left_pad = px(10.0);
    let gap = px(6.0);
    let right_pad = px(8.0);
    let text_x = left_pad + side + gap;
    let max_label_width = width - text_x - right_pad;
    let show_title = max_label_width >= px(18.0);
    let note_x = if show_title {
        left_pad
    } else {
        (width - side) / 2.0
    };
    if let Some(cover) = cover {
        add_visual(
            c,
            layers,
            &cover.cast()?,
            note_x,
            (height - side) / 2.0,
            side,
            side,
        )?
        .SetOpacity(if track.playing { 1.0 } else { 0.7 })?;
    }
    if show_title {
        let fitted;
        let label = if prepared.label.width <= max_label_width {
            &prepared.label
        } else {
            fitted = gfx.text_brush_fitted(
                track_title(track),
                geometry.scale,
                11.5,
                true,
                ink,
                max_label_width,
            )?;
            &fitted
        };
        add_visual(
            c,
            layers,
            &label.brush.cast()?,
            text_x,
            (height - label.height) / 2.0,
            label.width,
            label.height,
        )?;
    }
    Ok(())
}
