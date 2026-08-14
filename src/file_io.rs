use bytes::Bytes;
use std::path::{Path, PathBuf};

/// Supported image extensions for the native file dialog.
pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "webp", "bmp", "gif", "ico", "tiff", "tif", "tga", "hdr", "avif",
    "svg",
];

/// Metadata for an opened image file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageMetadata {
    pub path: PathBuf,
    pub filename: String,
    pub dimensions: (u32, u32),
    pub file_size_bytes: u64,
    pub format_name: String,
}

/// An in-memory loaded image with raw file bytes, decoded RGBA buffer, and parsed metadata.
#[derive(Debug, Clone)]
pub struct LoadedImage {
    pub metadata: ImageMetadata,
    pub bytes: Bytes,
    pub rgba_bytes: Option<Bytes>,
}

/// Open a native file picker dialog for selecting an image file.
pub fn pick_image_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Open Image - Opsis")
        .add_filter("Supported Images", SUPPORTED_EXTENSIONS)
        .add_filter("All Files", &["*"])
        .pick_file()
}

/// Load an image from disk and parse its metadata and raw bytes.
pub fn load_image(path: &Path) -> Result<LoadedImage, String> {
    if !path.exists() {
        return Err(format!("File does not exist: {}", path.display()));
    }

    let raw_bytes = std::fs::read(path)
        .map_err(|e| format!("Failed to read file '{}': {e}", path.display()))?;

    let file_size_bytes = raw_bytes.len() as u64;

    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("image")
        .to_string();

    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Determine dimensions, format, and decode RGBA pixels
    let (dimensions, format_name, rgba_bytes) = match image::ImageReader::new(std::io::Cursor::new(&raw_bytes))
        .with_guessed_format()
    {
        Ok(reader) => {
            let format_desc = reader
                .format()
                .map(|f| format!("{:?}", f).to_uppercase())
                .unwrap_or_else(|| ext.to_uppercase());

            if let Ok(dyn_img) = reader.decode() {
                let rgba = dyn_img.to_rgba8();
                let dims = rgba.dimensions();
                (dims, format_desc, Some(Bytes::from(rgba.into_raw())))
            } else {
                ((800, 600), format_desc, None)
            }
        }
        Err(_) => {
            if ext == "svg" {
                ((800, 600), "SVG".to_string(), None)
            } else if let Ok(dyn_img) = image::load_from_memory(&raw_bytes) {
                let rgba = dyn_img.to_rgba8();
                let dims = rgba.dimensions();
                (dims, ext.to_uppercase(), Some(Bytes::from(rgba.into_raw())))
            } else {
                return Err(format!("Unsupported or invalid image data for '{}'", path.display()));
            }
        }
    };

    Ok(LoadedImage {
        metadata: ImageMetadata {
            path: path.to_path_buf(),
            filename,
            dimensions,
            file_size_bytes,
            format_name,
        },
        bytes: Bytes::from(raw_bytes),
        rgba_bytes,
    })
}

/// Format a byte count into a human-readable string.
#[allow(dead_code)]
pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

/// Format image dimensions as `WIDTH x HEIGHT px`.
#[allow(dead_code)]
pub fn format_dimensions(dims: (u32, u32)) -> String {
    format!("{} x {} px", dims.0, dims.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(500), "500 B");
        assert_eq!(format_file_size(2048), "2.0 KB");
        assert_eq!(format_file_size(1024 * 1024 * 5), "5.0 MB");
    }

    #[test]
    fn test_format_dimensions() {
        assert_eq!(format_dimensions((1920, 1080)), "1920 x 1080 px");
        assert_eq!(format_dimensions((3840, 2160)), "3840 x 2160 px");
    }
}
