//! Window chrome shared by Whirlpool's small panels: rounded corners that do not depend on the
//! renderer's alpha support, and glass built from a picture of what is behind the panel.

use eframe::egui::{self, Color32, CornerRadius, Rect, TextureHandle, TextureOptions, pos2};
use whirlpool_core::glass::{self, Glass, Image};
use windows::{
    Win32::{
        Foundation::RECT,
        Graphics::Gdi::{
            BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CreateCompatibleBitmap,
            CreateCompatibleDC, CreateRoundRectRgn, DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC,
            GetDIBits, ReleaseDC, SRCCOPY, SelectObject, SetWindowRgn,
        },
        UI::WindowsAndMessaging::{FindWindowW, GetWindowRect},
    },
    core::HSTRING,
};

/// Clips the window titled `title` to a rounded rectangle. `rect` is in physical pixels relative to
/// the window's top-left corner. Works the same with OpenGL and DirectX, with or without alpha.
pub fn round_window(title: &str, rect: (i32, i32, i32, i32), radius_px: i32) {
    // SAFETY: Looks a window of this process up by title and gives it a region; Windows owns the
    // region afterwards.
    unsafe {
        let Ok(hwnd) = FindWindowW(None, &HSTRING::from(title)) else {
            return;
        };
        let mut bounds = RECT::default();
        if GetWindowRect(hwnd, &mut bounds).is_err() {
            return;
        }
        let (x, y, w, h) = rect;
        let region = CreateRoundRectRgn(x, y, x + w + 1, y + h + 1, radius_px * 2, radius_px * 2);
        let _ = SetWindowRgn(hwnd, Some(region), true);
    }
}

/// A picture of the screen area `(x, y, width, height)` in physical pixels, taken before a panel
/// is shown there. `None` if the screen cannot be read (locked session, remote desktop).
pub fn capture(x: i32, y: i32, width: i32, height: i32) -> Option<Image> {
    if width <= 0 || height <= 0 {
        return None;
    }
    // SAFETY: Plain GDI: every object created here is selected out and deleted before returning.
    unsafe {
        let screen = GetDC(None);
        if screen.is_invalid() {
            return None;
        }
        let dc = CreateCompatibleDC(Some(screen));
        let bitmap = CreateCompatibleBitmap(screen, width, height);
        let old = SelectObject(dc, bitmap.into());
        let copied = BitBlt(dc, 0, 0, width, height, Some(screen), x, y, SRCCOPY).is_ok();
        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bgra = vec![0u8; width as usize * height as usize * 4];
        let rows = GetDIBits(
            dc,
            bitmap,
            0,
            height as u32,
            Some(bgra.as_mut_ptr().cast()),
            &mut info,
            DIB_RGB_COLORS,
        );
        SelectObject(dc, old);
        let _ = DeleteObject(bitmap.into());
        let _ = DeleteDC(dc);
        ReleaseDC(None, screen);
        if !copied || rows == 0 {
            return None;
        }
        let mut image = Image::new(width as usize, height as usize);
        for (out, px) in image.rgba.chunks_exact_mut(4).zip(bgra.chunks_exact(4)) {
            out.copy_from_slice(&[px[2], px[1], px[0], 255]);
        }
        // A black frame means the capture was refused (protected content): do not use it.
        let lit = image
            .rgba
            .chunks_exact(4)
            .step_by(97)
            .any(|p| p[0] | p[1] | p[2] > 8);
        lit.then_some(image)
    }
}

/// Glass behind a panel: refraction of a picture taken before the panel opened.
pub struct GlassPanel {
    backdrop: Option<Image>,
    cached: Option<((usize, usize, u8), TextureHandle)>,
}

impl GlassPanel {
    pub fn new(backdrop: Option<Image>) -> Self {
        Self {
            backdrop,
            cached: None,
        }
    }

    /// Paints the glass over `panel` (points) and the dark wash that keeps text legible.
    /// `window` is the whole window rect (points) the backdrop was captured for. Returns `false`
    /// when there is no backdrop or glass is off; the caller then paints its flat fill.
    pub fn paint(
        &mut self,
        ctx: &egui::Context,
        painter: &egui::Painter,
        window: Rect,
        panel: Rect,
        radius: f32,
        strength: u8,
        wash: Color32,
    ) -> bool {
        let Some(backdrop) = &self.backdrop else {
            return false;
        };
        let ppp = ctx.pixels_per_point();
        let to_px = |v: f32| (v * ppp).round().max(0.0) as usize;
        let rect = (
            to_px(panel.left() - window.left()),
            to_px(panel.top() - window.top()),
            to_px(panel.width()),
            to_px(panel.height()),
        );
        if rect.2 == 0
            || rect.3 == 0
            || rect.0 + rect.2 > backdrop.width
            || rect.1 + rect.3 > backdrop.height
        {
            return false;
        }
        let key = (rect.2, rect.3, strength);
        if self.cached.as_ref().is_none_or(|(k, _)| *k != key) {
            let look = Glass::from_percent(strength, radius * ppp);
            let out = glass::refract(backdrop, rect, &look);
            let image =
                egui::ColorImage::from_rgba_unmultiplied([out.width, out.height], &out.rgba);
            self.cached = Some((
                key,
                ctx.load_texture("glass", image, TextureOptions::LINEAR),
            ));
        }
        if let Some((_, texture)) = &self.cached {
            painter.image(
                texture.id(),
                panel,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
            painter.rect_filled(panel, CornerRadius::same(radius as u8), wash);
        }
        true
    }
}
