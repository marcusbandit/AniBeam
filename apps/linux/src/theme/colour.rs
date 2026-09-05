//! sRGB colour maths for the token derivation: mixing, lightness, WCAG contrast, HSL for
//! the re-tone rule, and the two derived hues.

use crate::theme::Mode;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgb {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Rgb {
    /// `#rrggbb` or `#rgb`, with or without the hash. Anything else is None.
    pub fn hex(s: &str) -> Option<Rgb> {
        let s = s.trim().trim_start_matches('#');
        let expanded: String = match s.len() {
            6 => s.to_string(),
            3 => s.chars().flat_map(|c| [c, c]).collect(),
            _ => return None,
        };
        let v = u32::from_str_radix(&expanded, 16).ok()?;
        Some(Rgb {
            r: ((v >> 16) & 0xff) as f64 / 255.0,
            g: ((v >> 8) & 0xff) as f64 / 255.0,
            b: (v & 0xff) as f64 / 255.0,
        })
    }

    pub fn to_hex(self) -> String {
        let c = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        format!("#{:02x}{:02x}{:02x}", c(self.r), c(self.g), c(self.b))
    }

    /// Bytes for handing this colour to `material_colors::color::Argb`.
    pub fn bytes(self) -> (u8, u8, u8) {
        let c = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        (c(self.r), c(self.g), c(self.b))
    }

    /// The colour `t` of the way from self toward `other`.
    pub fn mix(self, other: Rgb, t: f64) -> Rgb {
        let t = t.clamp(0.0, 1.0);
        Rgb {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
        }
    }

    pub fn lightness(self) -> f64 {
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }

    fn lin(v: f64) -> f64 {
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }

    pub fn luminance(self) -> f64 {
        0.2126 * Self::lin(self.r) + 0.7152 * Self::lin(self.g) + 0.0722 * Self::lin(self.b)
    }

    /// WCAG contrast ratio, 1 to 21.
    pub fn contrast(self, other: Rgb) -> f64 {
        let (a, b) = (self.luminance(), other.luminance());
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    /// Hue, saturation and lightness, each 0 to 1.
    pub fn to_hsl(self) -> (f64, f64, f64) {
        let max = self.r.max(self.g).max(self.b);
        let min = self.r.min(self.g).min(self.b);
        let l = (max + min) / 2.0;
        if (max - min).abs() < f64::EPSILON {
            return (0.0, 0.0, l);
        }
        let d = max - min;
        let s = if l > 0.5 {
            d / (2.0 - max - min)
        } else {
            d / (max + min)
        };
        let mut h = if max == self.r {
            (self.g - self.b) / d + if self.g < self.b { 6.0 } else { 0.0 }
        } else if max == self.g {
            (self.b - self.r) / d + 2.0
        } else {
            (self.r - self.g) / d + 4.0
        };
        h /= 6.0;
        (h, s, l)
    }

    pub fn from_hsl(h: f64, s: f64, l: f64) -> Rgb {
        fn hue(p: f64, q: f64, mut t: f64) -> f64 {
            if t < 0.0 {
                t += 1.0
            }
            if t > 1.0 {
                t -= 1.0
            }
            if t < 1.0 / 6.0 {
                return p + (q - p) * 6.0 * t;
            }
            if t < 0.5 {
                return q;
            }
            if t < 2.0 / 3.0 {
                return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
            }
            p
        }
        if s <= 0.0 {
            return Rgb { r: l, g: l, b: l };
        }
        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;
        Rgb {
            r: hue(p, q, h + 1.0 / 3.0),
            g: hue(p, q, h),
            b: hue(p, q, h - 1.0 / 3.0),
        }
    }

    /// Same hue and saturation, lightness capped at 0.42 on a light ground or floored at
    /// 0.62 on a dark one: the forced-mode rule that keeps a dark terminal's pastels
    /// visible on white.
    pub fn retone(self, mode: Mode) -> Rgb {
        let (h, s, l) = self.to_hsl();
        let l = match mode {
            Mode::Light => l.min(0.42),
            Mode::Dark => l.max(0.62),
        };
        Rgb::from_hsl(h, s, l)
    }
}

/// The hue halfway from `a` to `b` the short way round, at their mean saturation and
/// lightness: the terminal palette's orange.
pub fn hue_between(a: Rgb, b: Rgb) -> Rgb {
    let (ha, sa, la) = a.to_hsl();
    let (hb, sb, lb) = b.to_hsl();
    let mut dh = hb - ha;
    if dh > 0.5 {
        dh -= 1.0
    }
    if dh < -0.5 {
        dh += 1.0
    }
    let mut h = ha + dh / 2.0;
    if h < 0.0 {
        h += 1.0
    }
    if h > 1.0 {
        h -= 1.0
    }
    Rgb::from_hsl(h, (sa + sb) / 2.0, (la + lb) / 2.0)
}

/// The terminal palette's brown: the orange with its saturation at 0.55 and lightness at 0.72.
pub fn browned(c: Rgb) -> Rgb {
    let (h, s, l) = c.to_hsl();
    Rgb::from_hsl(h, s * 0.55, l * 0.72)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Mode;

    #[test]
    fn hex_round_trips_and_short_form_expands() {
        assert_eq!(Rgb::hex("#191724").unwrap().to_hex(), "#191724");
        assert_eq!(Rgb::hex("#fff").unwrap().to_hex(), "#ffffff");
        assert_eq!(Rgb::hex("191724").unwrap().to_hex(), "#191724");
        assert!(Rgb::hex("#12345").is_none());
        assert!(Rgb::hex("blue").is_none());
    }

    #[test]
    fn mix_lightness_and_contrast() {
        let black = Rgb::hex("#000000").unwrap();
        let white = Rgb::hex("#ffffff").unwrap();
        assert_eq!(black.mix(white, 0.5).to_hex(), "#808080");
        assert!(black.lightness() < 0.01 && white.lightness() > 0.99);
        assert!((black.contrast(white) - 21.0).abs() < 0.01);
        assert_eq!(Mode::of_ground(Rgb::hex("#0f1114").unwrap()), Mode::Dark);
        assert_eq!(Mode::of_ground(Rgb::hex("#f6f7fa").unwrap()), Mode::Light);
    }

    #[test]
    fn retone_caps_lightness_on_light_and_floors_it_on_dark() {
        let pale = Rgb::hex("#eb6f92").unwrap();
        let (h, s, _) = pale.to_hsl();
        let on_light = pale.retone(Mode::Light);
        let (h2, s2, l2) = on_light.to_hsl();
        assert!((h - h2).abs() < 0.01 && (s - s2).abs() < 0.01);
        assert!(l2 <= 0.42 + 0.001);
        let dark = Rgb::hex("#402030").unwrap();
        assert!(dark.retone(Mode::Dark).to_hsl().2 >= 0.62 - 0.001);
    }

    #[test]
    fn orange_sits_between_red_and_yellow() {
        // Rosé Pine's red (343 degrees) and yellow (35 degrees) straddle the hue seam, so the
        // short arc between them crosses 0; the orange is the midpoint along that arc.
        let red = Rgb::hex("#eb6f92").unwrap();
        let yellow = Rgb::hex("#f6c177").unwrap();
        let (hr, _, _) = red.to_hsl();
        let (hy, _, _) = yellow.to_hsl();
        let orange = hue_between(red, yellow);
        let (ho, so, lo) = orange.to_hsl();
        let arc = |from: f64, to: f64| {
            let mut d = to - from;
            if d > 0.5 {
                d -= 1.0
            }
            if d < -0.5 {
                d += 1.0
            }
            d
        };
        let along = arc(hr, ho) / arc(hr, hy);
        assert!(
            (0.45..=0.55).contains(&along),
            "orange {ho} is {along} of the way from red {hr} to yellow {hy}"
        );
        let (_, sb, lb) = browned(orange).to_hsl();
        assert!(
            sb < so && lb < lo,
            "brown is duller and darker than the orange"
        );
    }
}
