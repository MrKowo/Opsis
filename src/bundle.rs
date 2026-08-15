use opsis_extension_api::ExtensionManifest;
use std::fs::File;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

/// Represents an unpacked and verified extension bundle ready for dynamic loading.
#[derive(Debug, Clone)]
pub struct ExtensionBundle {
    #[allow(dead_code)]
    pub manifest: ExtensionManifest,
    pub binary_path: PathBuf,
}

/// Detect the current target platform identifier.
pub fn current_platform_key() -> &'static str {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        "windows-x86_64"
    }
    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    {
        "windows-aarch64"
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        "linux-x86_64"
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        "linux-aarch64"
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "macos-aarch64"
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "macos-x86_64"
    }
    #[cfg(not(any(
        all(target_os = "windows", any(target_arch = "x86_64", target_arch = "aarch64")),
        all(target_os = "linux", any(target_arch = "x86_64", target_arch = "aarch64")),
        all(target_os = "macos", any(target_arch = "x86_64", target_arch = "aarch64")),
    )))]
    {
        "unknown"
    }
}

/// Unpack a .opx bundle (or recognize a raw native library) and prepare it for loading.
pub fn prepare_bundle(path: &Path, cache_dir: &Path) -> Result<ExtensionBundle, String> {
    if !path.exists() {
        return Err(format!("Extension path does not exist: {:?}", path));
    }

    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    match extension {
        "opx" => unpack_opx_archive(path, cache_dir),
        "dll" | "so" | "dylib" => {
            // Direct native dynamic library
            let file_stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            let manifest = ExtensionManifest {
                id: file_stem.to_string(),
                name: file_stem.to_string(),
                version: "0.1.0".to_string(),
                author: "Local".to_string(),
                description: "Direct native dynamic library extension".to_string(),
                api_version: 1,
            };

            Ok(ExtensionBundle {
                manifest,
                binary_path: path.to_path_buf(),
            })
        }
        _ => Err(format!(
            "Unsupported extension file format for {:?}. Expected .opx, .dll, .so, or .dylib",
            path
        )),
    }
}

fn unpack_opx_archive(opx_path: &Path, cache_dir: &Path) -> Result<ExtensionBundle, String> {
    let file = File::open(opx_path).map_err(|e| format!("Failed to open .opx file: {e}"))?;
    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("Failed to read .opx ZIP archive: {e}"))?;

    let file_stem = opx_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unnamed_extension");

    let target_extract_dir = cache_dir.join(file_stem);
    std::fs::create_dir_all(&target_extract_dir)
        .map_err(|e| format!("Failed to create cache directory: {e}"))?;

    // Read manifest.json from bundle
    let manifest: ExtensionManifest = {
        let manifest_file = archive
            .by_name("manifest.json")
            .map_err(|_| "Invalid .opx bundle: missing 'manifest.json'".to_string())?;
        serde_json::from_reader(manifest_file)
            .map_err(|e| format!("Failed to parse manifest.json: {e}"))?
    };

    // Locate the platform binary for the current operating system and architecture
    let platform_key = current_platform_key();
    let platform_bin_dir = target_extract_dir.join("bin").join(platform_key);

    // Check if the cache is already up-to-date
    let should_extract = if let Ok(bin) = find_dynamic_library_in_dir(&platform_bin_dir) {
        if let (Ok(opx_meta), Ok(bin_meta)) = (opx_path.metadata(), bin.metadata()) {
            if let (Ok(opx_time), Ok(bin_time)) = (opx_meta.modified(), bin_meta.modified()) {
                bin_time < opx_time
            } else {
                true
            }
        } else {
            true
        }
    } else {
        true
    };

    if should_extract {
        // Extract all files
        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| format!("Failed to read archive entry: {e}"))?;
            let outpath = match file.enclosed_name() {
                Some(path) => target_extract_dir.join(path),
                None => continue,
            };

            if file.is_dir() {
                std::fs::create_dir_all(&outpath)
                    .map_err(|e| format!("Failed to create dir in cache: {e}"))?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p)
                            .map_err(|e| format!("Failed to create parent dir: {e}"))?;
                    }
                }
                match File::create(&outpath) {
                    Ok(mut outfile) => {
                        std::io::copy(&mut file, &mut outfile)
                            .map_err(|e| format!("Failed to write extracted file {:?}: {e}", outpath))?;
                    }
                    Err(e) => {
                        // If file already exists and is locked by the active process, keep existing cached binary
                        if !outpath.exists() {
                            return Err(format!("Failed to extract file {:?}: {e}", outpath));
                        }
                    }
                }
            }
        }
    }

    let binary_path = find_dynamic_library_in_dir(&platform_bin_dir)?;

    Ok(ExtensionBundle {
        manifest,
        binary_path,
    })
}

fn find_dynamic_library_in_dir(dir: &Path) -> Result<PathBuf, String> {
    if !dir.exists() {
        return Err(format!(
            "Missing binary directory for current platform: {:?}",
            dir
        ));
    }

    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("Failed to read platform binary directory: {e}"))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if matches!(ext, "dll" | "so" | "dylib") {
                return Ok(path);
            }
        }
    }

    Err(format!("No dynamic library (.dll, .so, .dylib) found in {:?}", dir))
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_platform_key_validity() {
        let key = current_platform_key();
        assert!(!key.is_empty());
        assert_ne!(key, "unknown");
    }

    #[test]
    fn test_find_binary_in_empty_dir() {
        let temp_dir = std::env::temp_dir().join("opsis_test_empty_bin_dir");
        let _ = std::fs::create_dir_all(&temp_dir);
        let res = find_dynamic_library_in_dir(&temp_dir);
        assert!(res.is_err());
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
