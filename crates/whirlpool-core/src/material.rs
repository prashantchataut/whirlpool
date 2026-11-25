//! Glass materials for the dock and a wallpaper tint. Pure numbers: the dock turns them into
//! compositor brushes. Four looks: Frosted (milky), Liquid (clear, bright edge), Matte (solid,
//! no see-through) and Classic (the look Whirlpool has always had).

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Material {
    Frosted,
    Liquid,
    Matte,
    #[default]
    Classic,
}

/// What a material changes about the dock's surface.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    /// Alpha of the colour laid over the blurred backdrop.
    pub tint_alpha: u8,
    /// Alpha of the white highlight at the top.
    pub sheen_alpha: u8,
    /// Multiplier for the rim's alpha stops.
    pub rim_scale: f32,
    /// Whether the backdrop shows through at all.
    pub backdrop: bool,
}

impl Material {
    pub const ALL: [Self; 4] = [Self::Frosted, Self::Liquid, Self::Matte, Self::Classic];

    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::Frosted => "frosted",
            Self::Liquid => "liquid",
            Self::Matte => "matte",
            Self::Classic => "classic",
        }
    }

    /// `base_alpha` is the tint alpha of the Classic look for the current theme.
    #[must_use]
    pub fn look(self, base_alpha: u8) -> Look {
        let scaled = |k: f32| (f32::from(base_alpha) * k).round().clamp(0.0, 255.0) as u8;
        match self {
            Self::Classic => Look {
                tint_alpha: base_alpha,
                sheen_alpha: 38,
                rim_scale: 1.0,
                backdrop: true,
            },
            Self::Frosted => Look {
                tint_alpha: scaled(1.45).max(base_alpha),
                sheen_alpha: 22,
                rim_scale: 0.8,
                backdrop: true,
            },
            Self::Liquid => Look {
                tint_alpha: scaled(0.55),
                sheen_alpha: 78,
                rim_scale: 1.6,
                backdrop: true,
            },
            Self::Matte => Look {
                tint_alpha: 232,
                sheen_alpha: 10,
                rim_scale: 0.45,
                backdrop: false,
            },
        }
    }

    /// An icon style that suits the material, offered as a hint in Settings.
    #[must_use]
    pub fn blurb(self) -> &'static str {
        match self {
            Self::Frosted => "Milky glass: heavy tint, soft edge.",
            Self::Liquid => "Clear glass with a bright rim and a strong highlight.",
            Self::Matte => "Solid and quiet: nothing shows through.",
            Self::Classic => "Whirlpool's original glass.",
        }
    }
}

fn channel(v: u8) -> f32 {
    let c = f32::from(v) / 255.0;
    if c <= 0.039_28 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// WCAG relative luminance of an sRGB colour.
#[must_use]
pub fn luminance(c: [u8; 3]) -> f32 {
    0.2126 * channel(c[0]) + 0.7152 * channel(c[1]) + 0.0722 * channel(c[2])
}

/// WCAG contrast ratio between two colours.
#[must_use]
pub fn contrast(a: [u8; 3], b: [u8; 3]) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

fn mix(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let one = |x: u8, y: u8| {
        (f32::from(x) + (f32::from(y) - f32::from(x)) * t)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    [one(a[0], b[0]), one(a[1], b[1]), one(a[2], b[2])]
}

/// The dock tint after leaning toward the wallpaper. `strength` is 0 to 100 percent. The result is
/// pulled back until white text (dark theme) or black text (light theme) still reads at 4.5:1,
/// whatever the picture.
#[must_use]
pub fn wallpaper_tint(base: [u8; 3], wallpaper: [u8; 3], strength: u32, light: bool) -> [u8; 3] {
    let t = strength.min(100) as f32 / 100.0;
    let mut c = mix(base, wallpaper, t);
    let (pole, floor) = if light {
        ([255, 255, 255], [0, 0, 0])
    } else {
        ([0, 0, 0], [255, 255, 255])
    };
    // Walk toward black (dark theme) or white (light theme) until the text colour reads on it.
    for _ in 0..24 {
        if contrast(c, floor) >= 4.5 {
            break;
        }
        c = mix(c, pole, 0.15);
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classic_is_the_existing_look() {
        let l = Material::Classic.look(94);
        assert_eq!((l.tint_alpha, l.sheen_alpha, l.backdrop), (94, 38, true));
        assert!((l.rim_scale - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn materials_differ_in_the_way_their_names_say() {
        let (c, f, l, m) = (
            Material::Classic.look(94),
            Material::Frosted.look(94),
            Material::Liquid.look(94),
            Material::Matte.look(94),
        );
        assert!(f.tint_alpha > c.tint_alpha, "frosted is heavier");
        assert!(
            l.tint_alpha < c.tint_alpha && l.sheen_alpha > c.sheen_alpha && l.rim_scale > 1.0,
            "liquid is clearer and brighter"
        );
        assert!(!m.backdrop && m.tint_alpha > 200, "matte is solid");
        for mat in Material::ALL {
            let look = mat.look(255);
            assert!(look.tint_alpha <= 255);
            assert!(!mat.blurb().is_empty());
        }
    }

    #[test]
    fn wallpaper_tint_keeps_text_readable_for_any_picture() {
        let pictures = [
            [255, 255, 255],
            [0, 0, 0],
            [255, 0, 0],
            [0, 255, 0],
            [250, 240, 20],
            [20, 40, 230],
            [128, 128, 128],
        ];
        for p in pictures {
            for s in [0, 35, 70, 100] {
                let dark = wallpaper_tint([4, 34, 44], p, s, false);
                assert!(
                    contrast(dark, [255, 255, 255]) >= 4.5,
                    "dark {p:?} {s}: {dark:?}"
                );
                let light = wallpaper_tint([226, 250, 247], p, s, true);
                assert!(
                    contrast(light, [0, 0, 0]) >= 4.5,
                    "light {p:?} {s}: {light:?}"
                );
            }
        }
    }

    #[test]
    fn zero_strength_keeps_the_base_and_full_strength_follows_the_picture() {
        assert_eq!(
            wallpaper_tint([4, 34, 44], [200, 10, 10], 0, false),
            [4, 34, 44]
        );
        let t = wallpaper_tint([4, 34, 44], [120, 10, 10], 100, false);
        assert!(t[0] > t[1] && t[0] > t[2], "reddish: {t:?}");
    }

    #[test]
    fn contrast_reference_values() {
        assert!((contrast([0, 0, 0], [255, 255, 255]) - 21.0).abs() < 0.01);
        assert!((contrast([10, 10, 10], [10, 10, 10]) - 1.0).abs() < 0.001);
    }
}
