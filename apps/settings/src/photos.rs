//! The wallpaper pack: fourteen photographs from Pexels (Pexels License, free
//! to use and modify, no attribution required; credited anyway). They are
//! stored in the program so choosing one never needs a download.
//! Sources and licenses are listed in `ASSETS_LICENSES.md`.

use eframe::egui::ColorImage;

pub struct Photo {
    pub id: &'static str,
    pub title: &'static str,
    pub author: &'static str,
    /// The photo's page on Pexels.
    pub pexels_id: u32,
    pub jpeg: &'static [u8],
}

macro_rules! photo {
    ($id:literal, $title:literal, $author:literal, $pexels:literal) => {
        Photo {
            id: $id,
            title: $title,
            author: $author,
            pexels_id: $pexels,
            jpeg: include_bytes!(concat!("../../../assets/wallpapers/", $id, ".jpg")),
        }
    };
}

pub static PHOTOS: &[Photo] = &[
    photo!(
        "misty-forest",
        "Misty forest",
        "Soubhagya Maharana",
        18359798
    ),
    photo!("misty-peaks", "Misty peaks", "Luca Cavallin", 39360611),
    photo!("valley-fog", "Valley fog", "Quang Nguyen Vinh", 14024556),
    photo!("aurora-lake", "Aurora over a lake", "Damir K", 31284411),
    photo!(
        "aurora-iceland",
        "Iceland aurora",
        "Piotr Kowalonek",
        15264067
    ),
    photo!("milky-way", "Milky Way", "S S", 3083808),
    photo!("blue-hour", "Blue hour skyline", "Mohammed Alim", 30123502),
    photo!("city-lights", "City lights", "zhou shen", 18834566),
    photo!(
        "turquoise-surf",
        "Turquoise surf",
        "Jess Loiterton",
        9259704
    ),
    photo!("sea-foam", "Sea foam", "Jess Loiterton", 7502481),
    photo!("dune-sunrise", "Dune sunrise", "Francesco Ungaro", 998635),
    photo!("dune-sky", "Dune and sky", "Mike Norris", 30739226),
    photo!("dune-ripples", "Dune ripples", "MART PRODUCTION", 8869381),
    photo!("dune-night", "Dune at night", "Marjan Taghipour", 13620627),
];

#[must_use]
pub fn find(id: &str) -> Option<&'static Photo> {
    PHOTOS.iter().find(|photo| photo.id == id)
}

/// Decodes a JPEG at roughly `width` pixels across. The decoder skips whole
/// blocks of the picture when it is asked for a smaller one, so the grid of
/// previews costs far less than decoding fourteen 1440p photographs.
#[must_use]
pub fn thumbnail(jpeg: &[u8], width: u16) -> Option<ColorImage> {
    let mut decoder = jpeg_decoder::Decoder::new(jpeg);
    decoder.read_info().ok()?;
    let info = decoder.info()?;
    let height = (u32::from(width) * u32::from(info.height) / u32::from(info.width.max(1))) as u16;
    decoder.scale(width, height.max(1)).ok()?;
    let pixels = decoder.decode().ok()?;
    let info = decoder.info()?;
    let size = [usize::from(info.width), usize::from(info.height)];
    match info.pixel_format {
        jpeg_decoder::PixelFormat::RGB24 => Some(ColorImage::from_rgb(size, &pixels)),
        jpeg_decoder::PixelFormat::L8 => {
            let rgb: Vec<u8> = pixels.iter().flat_map(|&v| [v, v, v]).collect();
            Some(ColorImage::from_rgb(size, &rgb))
        }
        _ => None,
    }
}
