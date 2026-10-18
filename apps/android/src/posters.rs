//! Poster bytes from the store, decoded and cut to a thumbnail off the UI thread, with the
//! poster's own hue for the bands that sit behind it.

use image::RgbaImage;
use image::imageops::FilterType;
use slint::{Rgba8Pixel, SharedPixelBuffer};

/// Three times the largest poster on screen (a series page band at 3x density), cut to 2:3.
const WIDTH: u32 = 360;
const HEIGHT: u32 = 540;

pub struct Decoded {
    pub buffer: SharedPixelBuffer<Rgba8Pixel>,
    /// The poster's dominant hue, or `None` for a grey one.
    pub tone: Option<[u8; 3]>,
}

pub fn decode(bytes: &[u8]) -> Option<Decoded> {
    let image = image::load_from_memory(bytes).ok()?;
    let image = image.resize_to_fill(WIDTH, HEIGHT, FilterType::Triangle).to_rgba8();
    let tone = tone(&image);
    Some(Decoded { buffer: SharedPixelBuffer::clone_from_slice(image.as_raw(), image.width(), image.height()), tone })
}

/// The desktop's PosterTone: on a 24x36 copy, every pixel with some colour votes for one of
/// 24 hue bins with the square of its chroma, and the heaviest bin's average colour wins,
/// held between 30% and 70% lightness so a band made from it never goes black or white.
fn tone(image: &RgbaImage) -> Option<[u8; 3]> {
    const BINS: usize = 24;
    let small = image::imageops::resize(image, 24, 36, FilterType::Triangle);
    let mut weight = [0f32; BINS];
    let mut sum = [[0f32; 3]; BINS];
    for p in small.pixels() {
        let [r, g, b] = [p[0], p[1], p[2]].map(|c| f32::from(c) / 255.0);
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let chroma = max - min;
        if chroma < 0.12 || max < 0.18 {
            continue;
        }
        let sector = if max == r {
            ((g - b) / chroma).rem_euclid(6.0)
        } else if max == g {
            (b - r) / chroma + 2.0
        } else {
            (r - g) / chroma + 4.0
        };
        let bin = ((sector * 60.0 / (360.0 / BINS as f32)) as usize).min(BINS - 1);
        let w = chroma * chroma;
        weight[bin] += w;
        for (acc, c) in sum[bin].iter_mut().zip([r, g, b]) {
            *acc += c * w;
        }
    }
    let (best, &w) = weight.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1))?;
    if w <= 0.0 {
        return None;
    }
    let [r, g, b] = sum[best].map(|c| c / w);
    Some(clamp_lightness([r, g, b], 0.3, 0.7).map(|c| (c * 255.0).round() as u8))
}

fn clamp_lightness([r, g, b]: [f32; 3], low: f32, high: f32) -> [f32; 3] {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    let target = l.clamp(low, high);
    if (target - l).abs() < f32::EPSILON {
        return [r, g, b];
    }
    // Move toward black or white by the share that brings the lightness to the bound.
    if target > l {
        let t = (target - l) / (1.0 - l);
        [r, g, b].map(|c| c + (1.0 - c) * t)
    } else {
        let t = target / l;
        [r, g, b].map(|c| c * t)
    }
}

/// The band behind a poster: its hue darkened most of the way into the ground.
pub fn band(tone: Option<[u8; 3]>) -> slint::Color {
    const GROUND: [u8; 3] = [0x0c, 0x0d, 0x10];
    match tone {
        Some(t) => mix(t, GROUND, 0.4),
        None => slint::Color::from_rgb_u8(0x1c, 0x1e, 0x21),
    }
}

/// Text and bars on that band: the hue lifted toward white.
pub fn ink(tone: Option<[u8; 3]>) -> slint::Color {
    match tone {
        Some(t) => mix(t, [255, 255, 255], 0.55),
        None => slint::Color::from_rgb_u8(0xf2, 0xf4, 0xf6),
    }
}

/// `a` with `share` of it kept and the rest `b`.
fn mix(a: [u8; 3], b: [u8; 3], share: f32) -> slint::Color {
    let c = |i: usize| (f32::from(a[i]) * share + f32::from(b[i]) * (1.0 - share)).round() as u8;
    slint::Color::from_rgb_u8(c(0), c(1), c(2))
}
