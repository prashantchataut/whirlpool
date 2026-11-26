//! The wallpaper studio: a window of 4K pictures from 4kwallpapers.com with the credit always
//! in view. Previews are downloaded when the window opens, a picture only when Apply is pressed.
//! If the site refuses the download, the picture's page opens in the browser instead.

use std::{
    sync::mpsc::{self, Receiver},
    thread,
};

use eframe::egui::{
    self, Color32, ColorImage, CornerRadius, RichText, ScrollArea, Sense, TextureHandle,
    TextureOptions, Ui, vec2,
};
use whirlpool_core::studio::{self, Category, Entry};
use whirlpool_shell::studio as fetch;

use crate::{
    photos,
    ui::{self, Palette},
};

pub const TITLE: &str = "Wallpaper";

const DEFAULT_ACCENT: [u8; 3] = [0x0e, 0x93, 0x98];

enum Event {
    Preview(u32, Option<ColorImage>),
    Applied(u32, Result<(), String>, Option<[u8; 3]>),
}

struct Studio {
    category: Option<Category>,
    selected: Option<u32>,
    previews: Vec<(u32, TextureHandle)>,
    events: Receiver<Event>,
    sender: mpsc::Sender<Event>,
    busy: Option<u32>,
    status: String,
    sync: bool,
    accent: Option<[u8; 3]>,
}

fn accent_of(rgb: [u8; 3]) -> Color32 {
    Color32::from_rgb(rgb[0], rgb[1], rgb[2])
}

/// Reads a downloaded picture at a small size and returns its highlight colour.
fn sample_accent(file: &std::path::Path) -> Option<[u8; 3]> {
    let bytes = std::fs::read(file).ok()?;
    let image = photos::thumbnail(&bytes, 96)?;
    let rgb: Vec<u8> = image
        .pixels
        .iter()
        .flat_map(|pixel| [pixel.r(), pixel.g(), pixel.b()])
        .collect();
    Some(studio::dominant_accent(&rgb, DEFAULT_ACCENT))
}

/// Downloads and applies one picture, then stores its accent when the setting is on.
/// Shared by the window and `whirlpool --studio-apply`.
fn apply_entry(entry: &Entry) -> Result<Option<[u8; 3]>, String> {
    let file = fetch::apply(entry).map_err(|failure| failure.to_string())?;
    let sync = whirlpool_config::load(&whirlpool_config::default_path())
        .ok()
        .flatten()
        .is_some_and(|config| config.studio_accent_sync);
    if !sync {
        return Ok(None);
    }
    let accent = sample_accent(&file);
    if let Some(accent) = accent {
        let _ = whirlpool_config::update(&whirlpool_config::default_path(), |saved| {
            let mut saved = saved.unwrap_or_default();
            saved.studio_accent = Some(accent);
            Ok(saved)
        });
    }
    Ok(accent)
}

/// `whirlpool --studio-apply ID`: for scripts and the capture driver. Returns a process exit code.
pub fn apply_cli(id: &str) -> i32 {
    let Some(entry) = id.parse::<u32>().ok().and_then(studio::find) else {
        eprintln!("unknown wallpaper id {id}");
        return 2;
    };
    match apply_entry(entry) {
        Ok(_) => {
            println!("applied {} ({})", entry.title, entry.file_name());
            0
        }
        Err(why) => {
            eprintln!("could not apply {}: {why}", entry.title);
            1
        }
    }
}

impl Studio {
    fn new(ctx: &egui::Context) -> Self {
        let (sender, events) = mpsc::channel();
        let config = whirlpool_config::load(&whirlpool_config::default_path())
            .ok()
            .flatten();
        let previews_tx = sender.clone();
        let repaint = ctx.clone();
        thread::spawn(move || {
            for entry in studio::CATALOG {
                let image = fetch::thumbnail(entry)
                    .ok()
                    .and_then(|bytes| photos::thumbnail(&bytes, 480));
                if previews_tx.send(Event::Preview(entry.id, image)).is_err() {
                    return;
                }
                repaint.request_repaint();
            }
        });
        Self {
            category: None,
            selected: None,
            previews: Vec::new(),
            events,
            sender,
            busy: None,
            status: String::new(),
            sync: config.as_ref().is_some_and(|c| c.studio_accent_sync),
            accent: config.and_then(|c| c.studio_accent),
        }
    }

    fn poll(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.events.try_recv() {
            match event {
                Event::Preview(id, Some(image)) => {
                    let texture =
                        ctx.load_texture(format!("studio-{id}"), image, TextureOptions::LINEAR);
                    self.previews.push((id, texture));
                }
                Event::Preview(_, None) => {}
                Event::Applied(id, result, accent) => {
                    self.busy = None;
                    match result {
                        Ok(()) => {
                            self.status = "Applied".to_owned();
                            if self.sync {
                                self.accent = accent.or(self.accent);
                            }
                        }
                        Err(why) => {
                            self.status = format!("Could not download ({why}). Opened the page.");
                            if let Some(entry) = studio::find(id) {
                                let _ = whirlpool_shell::launcher::open(&entry.page_url());
                            }
                        }
                    }
                }
            }
        }
    }

    fn apply(&mut self, id: u32, ctx: &egui::Context) {
        let Some(entry) = studio::find(id) else {
            return;
        };
        self.busy = Some(id);
        self.status = "Downloading".to_owned();
        let sender = self.sender.clone();
        let ctx = ctx.clone();
        thread::spawn(move || {
            let result = apply_entry(entry);
            let event = match result {
                Ok(accent) => Event::Applied(id, Ok(()), accent),
                Err(why) => Event::Applied(id, Err(why), None),
            };
            let _ = sender.send(event);
            ctx.request_repaint();
        });
    }

    fn save_sync(&self) {
        let sync = self.sync;
        let _ = whirlpool_config::update(&whirlpool_config::default_path(), |saved| {
            let mut saved = saved.unwrap_or_default();
            saved.studio_accent_sync = sync;
            Ok(saved)
        });
    }

    fn chips(&mut self, ui: &mut Ui, palette: Palette, accent: Color32) {
        ui.horizontal_wrapped(|ui| {
            let all = self.category.is_none();
            if chip(ui, "All", all, palette, accent) {
                self.category = None;
            }
            for category in Category::ALL {
                let on = self.category == Some(category);
                if chip(ui, category.label(), on, palette, accent) {
                    self.category = Some(category);
                }
            }
        });
    }

    fn grid(&mut self, ui: &mut Ui, palette: Palette, accent: Color32) {
        let entries: Vec<&'static Entry> = studio::CATALOG
            .iter()
            .filter(|entry| self.category.is_none_or(|c| c == entry.category))
            .collect();
        let columns = if ui.available_width() > 880.0 { 4 } else { 3 };
        let gap = 14.0;
        let width = ((ui.available_width() - gap * (columns as f32 - 1.0)) / columns as f32)
            .floor()
            .max(120.0);
        let height = (width * 9.0 / 16.0).round();
        egui::Grid::new("studio-grid")
            .spacing([gap, gap + 6.0])
            .show(ui, |ui| {
                for (index, entry) in entries.iter().enumerate() {
                    ui.vertical(|ui| {
                        let (rect, response) =
                            ui.allocate_exact_size(vec2(width, height), Sense::click());
                        let corner = CornerRadius::same(6);
                        match self.previews.iter().find(|(id, _)| *id == entry.id) {
                            Some((_, texture)) => {
                                egui::Image::new(texture)
                                    .corner_radius(corner)
                                    .fit_to_exact_size(vec2(width, height))
                                    .paint_at(ui, rect);
                            }
                            None => {
                                ui.painter().rect_filled(rect, corner, palette.card);
                            }
                        }
                        let selected = self.selected == Some(entry.id);
                        if selected || response.hovered() {
                            ui.painter().rect_stroke(
                                rect,
                                corner,
                                egui::Stroke::new(if selected { 3.0 } else { 1.5 }, accent),
                                egui::StrokeKind::Inside,
                            );
                        }
                        if response.clicked() {
                            self.selected = Some(entry.id);
                        }
                        if response.double_clicked() && self.busy.is_none() {
                            self.apply(entry.id, ui.ctx());
                        }
                        ui.add_space(2.0);
                        ui.label(RichText::new(entry.title).size(13.0).color(palette.text));
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(studio::BADGE).size(11.0).color(palette.muted));
                            ui.hyperlink_to(
                                RichText::new("4kwallpapers.com").size(11.0),
                                entry.page_url(),
                            );
                        });
                    });
                    if (index + 1) % columns == 0 {
                        ui.end_row();
                    }
                }
            });
    }
}

fn chip(ui: &mut Ui, label: &str, on: bool, palette: Palette, accent: Color32) -> bool {
    let text =
        RichText::new(label)
            .size(13.0)
            .color(if on { Color32::WHITE } else { palette.text });
    let button = egui::Button::new(text)
        .fill(if on { accent } else { palette.card })
        .corner_radius(CornerRadius::same(14))
        .min_size(vec2(0.0, 28.0));
    ui.add(button).clicked()
}

impl eframe::App for Studio {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.07, 0.08, 0.09, 1.0]
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.poll(&ctx);
        let palette = Palette::of(ui);
        let accent = if self.sync {
            self.accent.map_or(ui::ACCENT, accent_of)
        } else {
            ui::ACCENT
        };
        egui::Panel::bottom("studio-bar")
            .frame(
                egui::Frame::new()
                    .fill(palette.sidebar)
                    .stroke(egui::Stroke::new(1.0, palette.border))
                    .inner_margin(egui::Margin::symmetric(24, 12)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.hyperlink_to(RichText::new(studio::CREDIT).size(12.0), studio::SITE);
                });
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    let mut sync = self.sync;
                    if ui::switch(ui, &mut sync).changed() {
                        self.sync = sync;
                        self.save_sync();
                    }
                    ui.label(
                        RichText::new("Match accent to wallpaper")
                            .size(13.0)
                            .color(palette.text),
                    );
                    if self.sync
                        && let Some(rgb) = self.accent
                    {
                        let (rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
                        ui.painter()
                            .circle_filled(rect.center(), 7.0, accent_of(rgb));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let ready = self.selected.is_some() && self.busy.is_none();
                        let label = if self.busy.is_some() {
                            "Applying…"
                        } else {
                            "Apply to desktop"
                        };
                        let button = egui::Button::new(
                            RichText::new(label).size(13.0).color(Color32::WHITE),
                        )
                        .fill(accent)
                        .min_size(vec2(150.0, 32.0));
                        if ui.add_enabled(ready, button).clicked()
                            && let Some(id) = self.selected
                        {
                            self.apply(id, &ctx);
                        }
                        ui.label(RichText::new(&self.status).size(12.0).color(palette.muted));
                    });
                });
            });
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(palette.window)
                    .inner_margin(egui::Margin::symmetric(24, 18)),
            )
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Wallpaper")
                        .font(ui::heading(22.0))
                        .color(palette.text),
                );
                ui.add_space(10.0);
                self.chips(ui, palette, accent);
                ui.add_space(12.0);
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.grid(ui, palette, accent);
                        ui.add_space(8.0);
                    });
            });
    }
}

pub fn run(compat: bool) -> eframe::Result {
    whirlpool_config::migrate_legacy_folder();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(TITLE)
            .with_inner_size([980.0, 680.0])
            .with_min_inner_size([620.0, 460.0]),
        centered: true,
        renderer: if compat {
            eframe::Renderer::Wgpu
        } else {
            eframe::Renderer::Glow
        },
        ..Default::default()
    };
    eframe::run_native(
        TITLE,
        options,
        Box::new(|creation| {
            ui::install_theme(&creation.egui_ctx);
            Ok(Box::new(Studio::new(&creation.egui_ctx)))
        }),
    )
}
