use bytes::Bytes;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

/// Supported image extensions for the native file dialog.
pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "webp", "bmp", "gif", "ico", "tiff", "tif", "tga", "hdr", "avif",
    "svg",
];

/// Metadata for an opened image file.
#[derive(Debug, Clone)]
pub struct ImageMetadata {
    pub path: PathBuf,
    pub filename: String,
    pub dimensions: (u32, u32),
    pub file_size_bytes: u64,
    pub format_name: String,
}

/// An in-memory loaded image with raw file bytes, decoded RGBA buffer cache, and parsed metadata.
#[derive(Debug, Clone)]
pub struct LoadedImage {
    pub metadata: ImageMetadata,
    pub bytes: Bytes,
    pub rgba_cache: Arc<OnceLock<Option<Bytes>>>,
}

impl LoadedImage {
    /// Retrieve the RGBA pixel buffer, decoding from raw image bytes on demand if not yet cached.
    /// Once decoded, subsequent calls return the cached buffer instantly with zero re-decoding overhead.
    pub fn get_rgba_or_decode(&self) -> Option<Bytes> {
        self.rgba_cache
            .get_or_init(|| {
                let start = std::time::Instant::now();
                if let Ok(dyn_img) = image::load_from_memory(&self.bytes) {
                    let rgba = dyn_img.to_rgba8();
                    let bytes = Bytes::from(rgba.into_raw());
                    let elapsed = start.elapsed();
                    crate::log_io!(
                        "Decoded RGBA buffer for '{}' ({} bytes) in {:.2}ms (cached for subsequent frames)",
                        self.metadata.filename,
                        bytes.len(),
                        elapsed.as_secs_f64() * 1000.0
                    );
                    Some(bytes)
                } else {
                    None
                }
            })
            .clone()
    }
}

/// Open a native file picker dialog for selecting an image file.
pub fn pick_image_file() -> Option<PathBuf> {
    let result = rfd::FileDialog::new()
        .set_title("Open Image - Opsis")
        .add_filter("Supported Images", SUPPORTED_EXTENSIONS)
        .add_filter("All Files", &["*"])
        .pick_file();

    if let Some(ref path) = result {
        crate::log_io!("File picker selected: '{}'", path.display());
    }
    result
}

/// Load an image from disk and parse its metadata and raw bytes.
pub fn load_image(path: &Path) -> Result<LoadedImage, String> {
    let start = std::time::Instant::now();
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

    // Fast path: determine format and dimensions from headers in sub-millisecond time
    let (dimensions, format_name, rgba_bytes) = match image::ImageReader::new(std::io::Cursor::new(&raw_bytes))
        .with_guessed_format()
    {
        Ok(reader) => {
            let format_desc = reader
                .format()
                .map(|f| format!("{:?}", f).to_uppercase())
                .unwrap_or_else(|| ext.to_uppercase());

            match reader.into_dimensions() {
                Ok(dims) if dims.0 > 0 && dims.1 > 0 => (dims, format_desc, None),
                _ => {
                    // Fallback to full decode if header-only dimensions parsing is not supported
                    if let Ok(dyn_img) = image::load_from_memory(&raw_bytes) {
                        let rgba = dyn_img.to_rgba8();
                        let dims = rgba.dimensions();
                        (dims, format_desc, Some(Bytes::from(rgba.into_raw())))
                    } else {
                        return Err(format!("Corrupted or unreadable image data in '{}'", path.display()));
                    }
                }
            }
        }
        Err(_) => {
            if ext == "svg" {
                if raw_bytes.starts_with(b"<") || std::str::from_utf8(&raw_bytes).map(|s| s.contains("<svg")).unwrap_or(false) {
                    ((800, 600), "SVG".to_string(), None)
                } else {
                    return Err(format!("Corrupted or invalid SVG file '{}'", path.display()));
                }
            } else if let Ok(dyn_img) = image::load_from_memory(&raw_bytes) {
                let rgba = dyn_img.to_rgba8();
                let dims = rgba.dimensions();
                (dims, ext.to_uppercase(), Some(Bytes::from(rgba.into_raw())))
            } else {
                return Err(format!("Unsupported or corrupted image format for '{}'", path.display()));
            }
        }
    };

    let elapsed = start.elapsed();
    crate::log_io!(
        "Loaded '{}' ({}, {}x{} px, {}) in {:.2}ms",
        path.display(),
        format_name,
        dimensions.0,
        dimensions.1,
        format_file_size(file_size_bytes),
        elapsed.as_secs_f64() * 1000.0
    );

    let rgba_cache = Arc::new(OnceLock::new());
    if let Some(rgba) = rgba_bytes {
        let _ = rgba_cache.set(Some(rgba));
    }

    Ok(LoadedImage {
        metadata: ImageMetadata {
            path: path.to_path_buf(),
            filename,
            dimensions,
            file_size_bytes,
            format_name,
        },
        bytes: Bytes::from(raw_bytes),
        rgba_cache,
    })
}

/// Returns all supported image files in a given directory, sorted alphabetically (case-insensitive).
pub fn find_images_in_directory(dir: &Path) -> Vec<PathBuf> {
    let mut images = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if SUPPORTED_EXTENSIONS.contains(&ext_lower.as_str()) {
                        images.push(path);
                    }
                }
            }
        }
    }

    // Sort naturally/case-insensitively by filename
    images.sort_by(|a, b| {
        let name_a = a
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        let name_b = b
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        name_a.cmp(&name_b)
    });

    crate::log_io!("Scanned directory '{}' -> Found {} images", dir.display(), images.len());
    images
}

/// Find the next or previous image path relative to the current image in its folder.
/// Wraps around to the first/last image when reaching either end of the directory.
pub fn get_adjacent_image_path(current_path: &Path, forward: bool) -> Option<PathBuf> {
    let parent = current_path.parent()?;
    let images = find_images_in_directory(parent);
    if images.is_empty() {
        return None;
    }

    // Find position of current image
    let current_canonical = current_path.canonicalize().ok();
    let current_filename = current_path.file_name();

    let current_idx = images.iter().position(|p| {
        if let (Some(ref curr), Ok(p_canon)) = (&current_canonical, p.canonicalize()) {
            curr == &p_canon
        } else {
            p.file_name() == current_filename
        }
    });

    let target = if let Some(idx) = current_idx {
        let target_idx = if forward {
            (idx + 1) % images.len()
        } else if idx == 0 {
            images.len() - 1
        } else {
            idx - 1
        };
        Some(images[target_idx].clone())
    } else {
        // If current image is not in list, return first or last
        if forward {
            images.first().cloned()
        } else {
            images.last().cloned()
        }
    };

    if let Some(ref next_path) = target {
        crate::log_io!(
            "Folder navigation ({}) from '{}' -> Target: '{}'",
            if forward { "Next" } else { "Prev" },
            current_path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
            next_path.file_name().and_then(|n| n.to_str()).unwrap_or("")
        );
    }

    target
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

fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// Format image aspect ratio as simplified `W:H`.
#[allow(dead_code)]
pub fn format_aspect_ratio(dims: (u32, u32)) -> String {
    if dims.0 == 0 || dims.1 == 0 {
        return "-".to_string();
    }
    let d = gcd(dims.0, dims.1);
    let (w, h) = (dims.0 / d, dims.1 / d);
    if (w, h) == (16, 9)
        || (w, h) == (4, 3)
        || (w, h) == (1, 1)
        || (w, h) == (3, 2)
        || (w, h) == (21, 9)
        || (w, h) == (9, 16)
        || (w, h) == (2, 3)
    {
        format!("{w}:{h}")
    } else {
        let ratio = dims.0 as f64 / dims.1 as f64;
        format!("{:.2}:1", ratio)
    }
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

    #[test]
    fn test_format_aspect_ratio() {
        assert_eq!(format_aspect_ratio((1920, 1080)), "16:9");
        assert_eq!(format_aspect_ratio((1024, 768)), "4:3");
        assert_eq!(format_aspect_ratio((1000, 1000)), "1:1");
        assert_eq!(format_aspect_ratio((0, 0)), "-");
    }

    #[test]
    fn test_load_image_and_on_demand_rgba() {
        let logo_path = Path::new("assets/logo.png");
        let loaded = load_image(logo_path).expect("Failed to load logo.png");
        assert_eq!(loaded.metadata.filename, "logo.png");
        assert!(loaded.metadata.dimensions.0 > 0);
        assert!(loaded.metadata.dimensions.1 > 0);
        assert!(!loaded.bytes.is_empty());
        // Initially rgba_cache is not yet populated (fast path without eager decoding)
        assert!(loaded.rgba_cache.get().is_none());

        // On-demand RGBA decoding should decode the full pixel buffer and cache it
        let rgba = loaded.get_rgba_or_decode();
        assert!(rgba.is_some());
        assert!(loaded.rgba_cache.get().is_some());
        let rgba_unwrapped = rgba.unwrap();
        let expected_len = (loaded.metadata.dimensions.0 * loaded.metadata.dimensions.1 * 4) as usize;
        assert_eq!(rgba_unwrapped.len(), expected_len);
    }

    #[test]
    fn test_find_images_and_adjacent_cycling() {
        let temp_dir = std::env::temp_dir().join("opsis_test_folder_cycling");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let img_a = temp_dir.join("a_first.png");
        let img_b = temp_dir.join("b_second.jpg");
        let img_c = temp_dir.join("c_third.webp");
        let non_img = temp_dir.join("readme.txt");

        std::fs::write(&img_a, b"fake png").unwrap();
        std::fs::write(&img_b, b"fake jpg").unwrap();
        std::fs::write(&img_c, b"fake webp").unwrap();
        std::fs::write(&non_img, b"not an image").unwrap();

        let images = find_images_in_directory(&temp_dir);
        assert_eq!(images.len(), 3);
        assert_eq!(images[0].file_name().unwrap(), "a_first.png");
        assert_eq!(images[1].file_name().unwrap(), "b_second.jpg");
        assert_eq!(images[2].file_name().unwrap(), "c_third.webp");

        // Forward cycling: a -> b -> c -> a (wrap around)
        assert_eq!(
            get_adjacent_image_path(&img_a, true).unwrap().file_name().unwrap(),
            "b_second.jpg"
        );
        assert_eq!(
            get_adjacent_image_path(&img_b, true).unwrap().file_name().unwrap(),
            "c_third.webp"
        );
        assert_eq!(
            get_adjacent_image_path(&img_c, true).unwrap().file_name().unwrap(),
            "a_first.png"
        );

        // Backward cycling: a -> c -> b -> a
        assert_eq!(
            get_adjacent_image_path(&img_a, false).unwrap().file_name().unwrap(),
            "c_third.webp"
        );
        assert_eq!(
            get_adjacent_image_path(&img_c, false).unwrap().file_name().unwrap(),
            "b_second.jpg"
        );
        assert_eq!(
            get_adjacent_image_path(&img_b, false).unwrap().file_name().unwrap(),
            "a_first.png"
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_corrupted_image_error() {
        let temp_file = std::env::temp_dir().join("corrupted_test_image.jpg");
        std::fs::write(&temp_file, b"this is completely corrupted not a jpeg").unwrap();

        let result = load_image(&temp_file);
        assert!(result.is_err());
        let _ = std::fs::remove_file(&temp_file);
    }
}
