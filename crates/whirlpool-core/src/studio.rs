//! The wallpaper studio's catalogue: a short list of 4K pictures on 4kwallpapers.com, picked
//! for a dark glass shell. Nothing here is a copy of a picture: an entry is a page address and a
//! title. The picture itself is downloaded only when the user presses Apply, and the credit and
//! link stay in view next to it.

/// The site every picture comes from.
pub const SITE: &str = "https://4kwallpapers.com";

/// The credit line shown in the studio, in Settings and on the site.
pub const CREDIT: &str =
    "Curated 4K wallpapers sourced from 4kwallpapers.com. Free for personal desktop use.";

/// What the badge on every tile says.
pub const BADGE: &str = "3840×2160 4K";

/// The size that is downloaded.
pub const SIZE: &str = "3840x2160";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Abstract,
    Amoled,
    Nature,
    Minimal,
    Space,
    Classics,
}

impl Category {
    pub const ALL: [Self; 6] = [
        Self::Abstract,
        Self::Amoled,
        Self::Nature,
        Self::Minimal,
        Self::Space,
        Self::Classics,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Abstract => "Abstract",
            Self::Amoled => "AMOLED",
            Self::Nature => "Nature",
            Self::Minimal => "Minimal",
            Self::Space => "Space",
            Self::Classics => "Classics",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    /// The number 4kwallpapers.com gives the picture.
    pub id: u32,
    pub category: Category,
    pub title: &'static str,
    /// The page, relative to the site: `category/name-id.html`.
    pub page: &'static str,
}

const fn entry(id: u32, category: Category, title: &'static str, page: &'static str) -> Entry {
    Entry {
        id,
        category,
        title,
        page,
    }
}

pub static CATALOG: &[Entry] = &[
    entry(
        28066,
        Category::Abstract,
        "Blue flow",
        "abstract/windows-11-2027-28066.html",
    ),
    entry(
        28132,
        Category::Abstract,
        "Smooth gradient",
        "abstract/windows-365-2027-28132.html",
    ),
    entry(
        27909,
        Category::Abstract,
        "Neon lines",
        "abstract/honor-win-2-neon-27909.html",
    ),
    entry(
        26734,
        Category::Amoled,
        "Metallic waves",
        "black-dark/metallic-abstract-26734.html",
    ),
    entry(
        24089,
        Category::Amoled,
        "Honeysuckle",
        "black-dark/macos-mojave-24089.html",
    ),
    entry(
        24088,
        Category::Amoled,
        "Glowing blossom",
        "black-dark/glowing-macos-24088.html",
    ),
    entry(
        27986,
        Category::Nature,
        "Lofoten",
        "nature/lofoten-mountains-27986.html",
    ),
    entry(
        27371,
        Category::Nature,
        "Milky Way over the Alps",
        "nature/milky-way-alps-27371.html",
    ),
    entry(
        26361,
        Category::Nature,
        "Misty hills",
        "nature/misty-forest-green-26361.html",
    ),
    entry(
        27374,
        Category::Nature,
        "Kawah Ijen",
        "nature/kawah-ijen-crater-27374.html",
    ),
    entry(
        27362,
        Category::Minimal,
        "Desert rocks",
        "nature/desert-rocks-27362.html",
    ),
    entry(
        26307,
        Category::Minimal,
        "Lone tree",
        "nature/lone-tree-26307.html",
    ),
    entry(
        26311,
        Category::Minimal,
        "Dunes and moon",
        "nature/sand-dunes-moon-26311.html",
    ),
    entry(
        27329,
        Category::Space,
        "Moon horizon",
        "space/moon-horizon-earth-27329.html",
    ),
    entry(
        27369,
        Category::Space,
        "Moon, night sky",
        "space/moon-night-sky-27369.html",
    ),
    entry(
        27326,
        Category::Space,
        "Crescent Earth",
        "space/crescent-earth-moon-27326.html",
    ),
    entry(
        24090,
        Category::Classics,
        "Mojave night",
        "nature/macos-mojave-starry-24090.html",
    ),
    entry(
        11633,
        Category::Classics,
        "Big Sur",
        "nature/macos-big-sur-11633.html",
    ),
    entry(
        12482,
        Category::Classics,
        "Fusion",
        "nature/macos-fusion-8k-12482.html",
    ),
    entry(
        11578,
        Category::Classics,
        "Sonoma",
        "abstract/macos-sonoma-11578.html",
    ),
];

#[must_use]
pub fn find(id: u32) -> Option<&'static Entry> {
    CATALOG.iter().find(|entry| entry.id == id)
}

#[must_use]
pub fn in_category(category: Category) -> Vec<&'static Entry> {
    CATALOG
        .iter()
        .filter(|entry| entry.category == category)
        .collect()
}

impl Entry {
    /// The picture's page on the site: the credit link.
    #[must_use]
    pub fn page_url(&self) -> String {
        format!("{SITE}/{}", self.page)
    }

    /// An 800 px preview.
    #[must_use]
    pub fn thumb_url(&self) -> String {
        format!("{SITE}/images/walls/thumbs_2t/{}.jpg", self.id)
    }

    /// The page's file name without the number and `.html`, which is how the site names downloads.
    #[must_use]
    pub fn slug(&self) -> &str {
        let name = self.page.rsplit('/').next().unwrap_or(self.page);
        let name = name.strip_suffix(".html").unwrap_or(name);
        let tail = format!("-{}", self.id);
        name.strip_suffix(tail.as_str()).unwrap_or(name)
    }

    /// Where the 4K file is expected. The studio reads the page first and only falls back to this.
    #[must_use]
    pub fn download_url(&self) -> String {
        format!(
            "{SITE}/images/wallpapers/{}-{SIZE}-{}.jpg",
            self.slug(),
            self.id
        )
    }

    /// The name the file gets in the cache.
    #[must_use]
    pub fn file_name(&self) -> String {
        format!("{}-{SIZE}-{}.jpg", self.slug(), self.id)
    }
}

/// The 4K link on a picture page, as an absolute address.
#[must_use]
pub fn find_download_link(html: &str) -> Option<String> {
    let mut rest = html;
    while let Some(at) = rest.find("href=\"") {
        rest = &rest[at + 6..];
        let end = rest.find('"')?;
        let link = &rest[..end];
        if link.contains("/images/wallpapers/") && link.contains(SIZE) {
            return Some(if link.starts_with("http") {
                link.to_owned()
            } else {
                format!("{SITE}{link}")
            });
        }
    }
    None
}

/// A download is a picture only if it starts like a JPEG or a PNG. A site that answers a script
/// with a challenge page sometimes returns HTML with status 200, and that must never be saved as
/// a wallpaper.
#[must_use]
pub fn looks_like_image(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0xff, 0xd8, 0xff]) || bytes.starts_with(&[0x89, b'P', b'N', b'G'])
}

/// The colour that stands out in a picture: the strongest hue among pixels that are neither grey
/// nor near black, averaged. `rgb` is packed R, G, B. Returns `fallback` for a picture without
/// colour.
#[must_use]
pub fn dominant_accent(rgb: &[u8], fallback: [u8; 3]) -> [u8; 3] {
    let mut weight = [0.0f64; 12];
    let mut sum = [[0.0f64; 3]; 12];
    for pixel in rgb.chunks_exact(3) {
        let (r, g, b) = (
            f64::from(pixel[0]) / 255.0,
            f64::from(pixel[1]) / 255.0,
            f64::from(pixel[2]) / 255.0,
        );
        let high = r.max(g).max(b);
        let low = r.min(g).min(b);
        let chroma = high - low;
        if high < 0.25 || chroma < 0.12 {
            continue;
        }
        let saturation = chroma / high;
        let hue = if (high - r).abs() < f64::EPSILON {
            ((g - b) / chroma).rem_euclid(6.0)
        } else if (high - g).abs() < f64::EPSILON {
            (b - r) / chroma + 2.0
        } else {
            (r - g) / chroma + 4.0
        };
        let bin = ((hue / 6.0 * 12.0) as usize).min(11);
        let w = saturation * high;
        weight[bin] += w;
        sum[bin][0] += r * w;
        sum[bin][1] += g * w;
        sum[bin][2] += b * w;
    }
    let mut best = 0;
    for bin in 1..12 {
        if weight[bin] > weight[best] {
            best = bin;
        }
    }
    if weight[best] <= 0.0 {
        return fallback;
    }
    let mut out = [0u8; 3];
    for (channel, slot) in out.iter_mut().enumerate() {
        *slot = (sum[best][channel] / weight[best] * 255.0)
            .round()
            .clamp(0.0, 255.0) as u8;
    }
    // A highlight, not a shadow: lift a dark average so text on it stays readable.
    let top = out.iter().copied().max().unwrap_or(0);
    if top < 150 && top > 0 {
        let lift = 150.0 / f64::from(top);
        for value in &mut out {
            *value = (f64::from(*value) * lift).round().min(255.0) as u8;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_follow_the_site_pattern() {
        let first = find(28066).expect("entry");
        assert_eq!(first.slug(), "windows-11-2027");
        assert_eq!(
            first.download_url(),
            "https://4kwallpapers.com/images/wallpapers/windows-11-2027-3840x2160-28066.jpg"
        );
        assert_eq!(
            first.page_url(),
            "https://4kwallpapers.com/abstract/windows-11-2027-28066.html"
        );
        assert_eq!(
            first.thumb_url(),
            "https://4kwallpapers.com/images/walls/thumbs_2t/28066.jpg"
        );
    }

    #[test]
    fn every_page_ends_with_its_own_number() {
        for entry in CATALOG {
            assert!(
                entry.page.ends_with(&format!("-{}.html", entry.id)),
                "{}",
                entry.title
            );
            assert!(
                !entry.slug().ends_with(&entry.id.to_string()),
                "{}",
                entry.title
            );
            assert!(!entry.title.is_empty());
        }
    }

    #[test]
    fn ids_are_unique_and_every_category_has_pictures() {
        for (at, one) in CATALOG.iter().enumerate() {
            assert!(CATALOG[at + 1..].iter().all(|other| other.id != one.id));
        }
        for category in Category::ALL {
            assert!(in_category(category).len() >= 3, "{}", category.label());
        }
    }

    #[test]
    fn download_link_is_read_from_the_page() {
        let html = r#"<a href="/images/wallpapers/x-1920x1080-1.jpg">a</a>
            <a title="t" href="/images/wallpapers/x-3840x2160-1.jpg" target="_blank">4K</a>"#;
        assert_eq!(
            find_download_link(html).as_deref(),
            Some("https://4kwallpapers.com/images/wallpapers/x-3840x2160-1.jpg")
        );
        assert_eq!(find_download_link("<html>Just a moment...</html>"), None);
    }

    #[test]
    fn a_challenge_page_is_not_a_picture() {
        assert!(looks_like_image(&[0xff, 0xd8, 0xff, 0xe0, 0]));
        assert!(looks_like_image(b"\x89PNG\r\n"));
        assert!(!looks_like_image(b"<!DOCTYPE html>"));
        assert!(!looks_like_image(&[]));
    }

    #[test]
    fn accent_is_the_strong_hue_not_the_grey() {
        let mut pixels = Vec::new();
        for _ in 0..500 {
            pixels.extend_from_slice(&[40, 40, 40]);
        }
        for _ in 0..100 {
            pixels.extend_from_slice(&[230, 90, 30]);
        }
        for _ in 0..20 {
            pixels.extend_from_slice(&[20, 60, 200]);
        }
        let accent = dominant_accent(&pixels, [1, 2, 3]);
        assert!(accent[0] > 200 && accent[2] < 80, "{accent:?}");
    }

    #[test]
    fn a_grey_picture_keeps_the_default_accent() {
        let pixels = vec![90u8; 3 * 64];
        assert_eq!(dominant_accent(&pixels, [14, 147, 152]), [14, 147, 152]);
    }

    #[test]
    fn a_dark_accent_is_lifted() {
        let pixels: Vec<u8> = std::iter::repeat_n([20u8, 30, 90], 200).flatten().collect();
        let accent = dominant_accent(&pixels, [0, 0, 0]);
        assert!(accent.iter().copied().max().unwrap() >= 140, "{accent:?}");
    }
}
