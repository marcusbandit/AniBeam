//! Poster bytes from the store, decoded and cut to a thumbnail off the UI thread.

use image::imageops::FilterType;
use slint::{Rgba8Pixel, SharedPixelBuffer};

/// Three times the largest poster on screen (a Next up card at 3x density), cut to 2:3.
const WIDTH: u32 = 300;
const HEIGHT: u32 = 450;

pub fn thumbnail(bytes: &[u8]) -> Option<SharedPixelBuffer<Rgba8Pixel>> {
    let image = image::load_from_memory(bytes).ok()?;
    let image = image.resize_to_fill(WIDTH, HEIGHT, FilterType::Triangle).to_rgba8();
    Some(SharedPixelBuffer::clone_from_slice(image.as_raw(), image.width(), image.height()))
}
