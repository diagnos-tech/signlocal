//! PNG output for egui screenshots.

use std::path::Path;

use egui::ColorImage;

/// Failure while writing a screenshot.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Encoding or I/O failed.
    #[error("cannot write PNG {path}: {source}")]
    Write {
        /// Destination.
        path: String,
        /// Cause.
        source: image::ImageError,
    },
}

/// Saves `image` as an RGBA PNG at `path`, creating parent directories.
pub fn save_png(image: &ColorImage, path: &Path) -> Result<(), Error> {
    let wrap = |source| Error::Write {
        path: path.display().to_string(),
        source,
    };
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| wrap(e.into()))?;
    }
    let [width, height] = image.size;
    image::save_buffer(
        path,
        image.as_raw(),
        width as u32,
        height as u32,
        image::ColorType::Rgba8,
    )
    .map_err(wrap)
}
