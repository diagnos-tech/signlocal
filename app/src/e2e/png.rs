//! Screenshots as PNG files.

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use egui::ColorImage;

/// Writes `image` (opaque sRGB) to `path` as an 8-bit RGBA PNG.
pub fn write(path: &Path, image: &ColorImage) -> Result<(), String> {
    let [width, height] = image.size;
    let (Ok(width), Ok(height)) = (u32::try_from(width), u32::try_from(height)) else {
        return Err("image too large".to_owned());
    };
    let rgba: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
    let file = File::create(path).map_err(|e| format!("cannot create {}: {e}", path.display()))?;
    let mut encoder = png::Encoder::new(BufWriter::new(file), width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::High);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(&rgba).map_err(|e| e.to_string())?;
    writer.finish().map_err(|e| e.to_string())
}
