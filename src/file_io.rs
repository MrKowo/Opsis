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

/// An in-memory loaded image with raw file bytes and parsed metadata.
#[derive(Debug, Clone)]
pub struct LoadedImage {
    pub metadata: ImageMetadata,
    pub bytes: Bytes,
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

    // Determine dimensions and format
    let (dimensions, format_name) = match image::ImageReader::new(std::io::Cursor::new(&raw_bytes))
        .with_guessed_format()
    {
        Ok(reader) => {
            let format_desc = reader
                .format()
                .map(|f| format!("{:?}", f).to_uppercase())
                .unwrap_or_else(|| ext.to_uppercase());

            match reader.into_dimensions() {
                Ok(dims) => (dims, format_desc),
                Err(_) => {
                    if ext == "svg" {
                        ((800, 600), "SVG".to_string())
                    } else {
                        match image::load_from_memory(&raw_bytes) {
                            Ok(dyn_img) => {
                                use image::GenericImageView;
                                (dyn_img.dimensions(), format_desc)
                            }
                            Err(e) => return Err(format!("Unsupported or invalid image data: {e}")),
                        }
                    }
                }
            }
        }
        Err(_) => {
            if ext == "svg" {
                ((800, 600), "SVG".to_string())
            } else {
                return Err(format!("Could not determine format of '{}'", path.display()));
            }
        }
    };

    let metadata = ImageMetadata {
        path: path.to_path_buf(),
        filename,
        dimensions,
        file_size_bytes,
        format_name,
    };

    Ok(LoadedImage {
        metadata,
        bytes: Bytes::from(raw_bytes),
    })
}

/// Format file size in human-readable bytes (KB, MB, GB).
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
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

/// Format dimensions (e.g. "1920 × 1080 px").
#[allow(dead_code)]
pub fn format_dimensions(dimensions: (u32, u32)) -> String {
    format!("{} × {} px", dimensions.0, dimensions.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(500), "500 B");
        assert_eq!(format_file_size(2048), "2 KB");
        assert_eq!(format_file_size(2_500_000), "2.4 MB");
    }

    #[test]
    fn test_format_dimensions() {
        assert_eq!(format_dimensions((1920, 1080)), "1920 × 1080 px");
    }
}
