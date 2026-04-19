//! scribe-images: image loading, dimension extraction, SVG rasterization,
//! page-fit resizing.
//!
//! Responsibilities:
//! - Load an image file from disk.
//! - Detect its format (PNG / JPEG / GIF / WebP / BMP / TIFF / SVG).
//! - Rasterize SVG to PNG via `resvg` (Word's SVG support is inconsistent).
//! - Expose pixel dimensions and the raw bytes suitable for embedding
//!   into a `.docx` via `docx-rs::Pic`.

use std::path::Path;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImageError {
    #[error("failed to read image {path:?}: {source}")]
    Io {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("unsupported image format: {0}")]
    Format(String),
    #[error("failed to decode image: {0}")]
    Decode(String),
    #[error("SVG rasterization failed: {0}")]
    Svg(String),
}

pub type Result<T> = std::result::Result<T, ImageError>;

/// A loaded image ready to embed.
#[derive(Debug, Clone)]
pub struct LoadedImage {
    /// Raw bytes suitable for `Pic::new_with_dimensions`. For SVG input,
    /// these are rasterized PNG bytes.
    pub bytes: Vec<u8>,
    /// Width in pixels.
    pub width_px: u32,
    /// Height in pixels.
    pub height_px: u32,
}

impl LoadedImage {
    /// Aspect ratio (width / height).
    pub fn aspect(&self) -> f32 {
        if self.height_px == 0 {
            1.0
        } else {
            self.width_px as f32 / self.height_px as f32
        }
    }

    /// Compute a page-width-fitted (width, height) in EMUs. Word's
    /// standard letter page (6" text area at 96 DPI) is 5_486_400 EMU
    /// wide; A4 with 2.54 cm margins is ~5_943_600 EMU. We default to
    /// 5_600_000 EMU which fits both with a small gutter.
    ///
    /// If `max_frac` is `Some(0.8)`, the image is scaled to 80% of the
    /// text area width. Use `None` to fit full width.
    pub fn page_fit_emu(&self, max_frac: Option<f32>) -> (u32, u32) {
        const PAGE_WIDTH_EMU: u32 = 5_600_000;
        let width_emu = match max_frac {
            Some(f) => (PAGE_WIDTH_EMU as f32 * f).max(1.0) as u32,
            None => PAGE_WIDTH_EMU,
        };

        // Convert source pixel dimensions to EMU assuming 96 DPI
        // (1 inch = 914400 EMU, 1 inch = 96 px so 1 px = 9525 EMU).
        let native_w_emu = self.width_px.saturating_mul(9525);
        let native_h_emu = self.height_px.saturating_mul(9525);

        if native_w_emu <= width_emu {
            // Native size fits; keep aspect ratio unchanged.
            (native_w_emu, native_h_emu)
        } else {
            // Scale down to width_emu preserving aspect.
            let ratio = self.aspect();
            let scaled_h = (width_emu as f32 / ratio) as u32;
            (width_emu, scaled_h.max(1))
        }
    }
}

/// Load and normalize an image from disk.
pub fn load(path: impl AsRef<Path>) -> Result<LoadedImage> {
    let path = path.as_ref();
    let bytes = std::fs::read(path).map_err(|e| ImageError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase());

    if matches!(ext.as_deref(), Some("svg")) {
        return rasterize_svg(&bytes);
    }

    decode_raster(&bytes)
}

/// Decode raster image bytes (PNG/JPEG/GIF/WebP/BMP/TIFF).
pub fn decode_raster(bytes: &[u8]) -> Result<LoadedImage> {
    let img = image::load_from_memory(bytes).map_err(|e| ImageError::Decode(e.to_string()))?;
    let (w, h) = (img.width(), img.height());

    // If the input format is directly embeddable (PNG/JPEG), keep the
    // original bytes. Otherwise re-encode to PNG for Word's most reliable
    // renderer path.
    let reencode = image::guess_format(bytes)
        .map(|f| {
            !matches!(
                f,
                image::ImageFormat::Png | image::ImageFormat::Jpeg | image::ImageFormat::Gif
            )
        })
        .unwrap_or(true);

    let output = if reencode {
        let mut out = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .map_err(|e| ImageError::Decode(e.to_string()))?;
        out
    } else {
        bytes.to_vec()
    };

    Ok(LoadedImage {
        bytes: output,
        width_px: w,
        height_px: h,
    })
}

/// Rasterize SVG to PNG using `resvg`. The output is sized at the SVG's
/// native pixel dimensions (or 1024×768 fallback if size is unspecified).
pub fn rasterize_svg(bytes: &[u8]) -> Result<LoadedImage> {
    let opt = resvg::usvg::Options::default();
    let tree =
        resvg::usvg::Tree::from_data(bytes, &opt).map_err(|e| ImageError::Svg(e.to_string()))?;

    let size = tree.size();
    let width = size.width().ceil() as u32;
    let height = size.height().ceil() as u32;
    let (width, height) = if width == 0 || height == 0 {
        (1024, 768)
    } else {
        (width, height)
    };

    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| ImageError::Svg("pixmap allocation failed".into()))?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );

    let png = pixmap
        .encode_png()
        .map_err(|e| ImageError::Svg(e.to_string()))?;

    Ok(LoadedImage {
        bytes: png,
        width_px: width,
        height_px: height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny_png() -> Vec<u8> {
        // Generate a 2x2 red PNG in-memory with the `image` crate — guaranteed valid bytes.
        let img = image::RgbImage::from_fn(2, 2, |_, _| image::Rgb([255, 0, 0]));
        let mut out = Vec::new();
        image::DynamicImage::ImageRgb8(img)
            .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .unwrap();
        out
    }

    #[test]
    fn decodes_png_bytes() {
        let bytes = tiny_png();
        let img = decode_raster(&bytes).unwrap();
        assert_eq!(img.width_px, 2);
        assert_eq!(img.height_px, 2);
        assert!(img.bytes.starts_with(b"\x89PNG"));
    }

    #[test]
    fn page_fit_scales_down_large_images() {
        let img = LoadedImage {
            bytes: vec![],
            width_px: 2000,
            height_px: 1000,
        };
        let (w, h) = img.page_fit_emu(None);
        assert!(w <= 5_600_000);
        // Aspect 2:1 should be preserved roughly.
        assert!((w as f32 / h as f32 - 2.0).abs() < 0.01);
    }

    #[test]
    fn page_fit_keeps_small_images_native() {
        // 400 px × 300 px ≈ 3.8 M × 2.86 M EMU, well under page width.
        let img = LoadedImage {
            bytes: vec![],
            width_px: 400,
            height_px: 300,
        };
        let (w, h) = img.page_fit_emu(None);
        assert_eq!(w, 400 * 9525);
        assert_eq!(h, 300 * 9525);
    }

    #[test]
    fn max_frac_scales_below_page_width() {
        let img = LoadedImage {
            bytes: vec![],
            width_px: 10_000,
            height_px: 5_000,
        };
        let (w, _) = img.page_fit_emu(Some(0.5));
        assert!(w <= 5_600_000 / 2 + 100);
    }
}
