use freya::prelude::Element;
use opsis_extension_api::{
    EventAction, ExtensionRegistry, InputContext, InputEvent, OverlayContext, ViewportContext,
};
use std::path::{Path, PathBuf};

use crate::bundle::prepare_bundle;
use crate::loader::{load_native_extension, LoadedExtension};

/// Return the standard OS user extensions directory (XDG on Linux, AppData on Windows, Application Support on macOS).
pub fn get_system_user_extensions_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            PathBuf::from(appdata).join("Opsis").join("extensions")
        } else if let Ok(userprofile) = std::env::var("USERPROFILE") {
            PathBuf::from(userprofile)
                .join("AppData")
                .join("Roaming")
                .join("Opsis")
                .join("extensions")
        } else {
            PathBuf::from("extensions")
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("Opsis")
                .join("extensions")
        } else {
            PathBuf::from("extensions")
        }
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(xdg).join("opsis").join("extensions")
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".config").join("opsis").join("extensions")
        } else {
            PathBuf::from("extensions")
        }
    }
}

/// Central Extension Manager acting as the foundation layer of Opsis.
pub struct ExtensionManager {
    #[allow(dead_code)]
    pub is_portable: bool,
    pub extensions_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub registry: ExtensionRegistry,
    pub loaded_extensions: Vec<LoadedExtension>,
}

impl Default for ExtensionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtensionManager {
    /// Initialize the extension manager detecting portable mode vs system user profile mode.
    pub fn new() -> Self {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        let portable_extensions = exe_dir.join("extensions");
        let cwd_extensions = PathBuf::from("extensions");
        let system_extensions = get_system_user_extensions_dir();

        // If an extensions folder exists adjacent to the binary or in CWD, operate in Portable Mode
        let (is_portable, primary_extensions_dir, cache_dir) = if portable_extensions.exists() {
            (true, portable_extensions, exe_dir.join(".extension_cache"))
        } else if cwd_extensions.exists() {
            (true, cwd_extensions, PathBuf::from(".extension_cache"))
        } else {
            // System Profile Mode (e.g. standalone binary on Desktop / Downloads or installed in Program Files)
            let system_cache = system_extensions
                .parent()
                .unwrap_or(&system_extensions)
                .join("cache");
            let _ = std::fs::create_dir_all(&system_extensions);
            let _ = std::fs::create_dir_all(&system_cache);
            (false, system_extensions, system_cache)
        };

        // Ensure directories exist
        let _ = std::fs::create_dir_all(&primary_extensions_dir);
        let _ = std::fs::create_dir_all(&cache_dir);

        let mut manager = Self {
            is_portable,
            extensions_dir: primary_extensions_dir,
            cache_dir,
            registry: ExtensionRegistry::new(),
            loaded_extensions: Vec::new(),
        };

        manager.discover_and_load_all();
        manager
    }

    /// Discover and load all extensions across portable and system directories.
    pub fn discover_and_load_all(&mut self) {
        let mut dirs_to_scan = Vec::new();
        let mut seen_dirs = std::collections::HashSet::new();

        let candidates = [
            self.extensions_dir.clone(),
            PathBuf::from("extensions"),
            get_system_user_extensions_dir(),
        ];

        for dir in candidates {
            if dir.exists() {
                if let Ok(canonical) = dir.canonicalize() {
                    if seen_dirs.insert(canonical) {
                        dirs_to_scan.push(dir);
                    }
                } else if seen_dirs.insert(dir.clone()) {
                    dirs_to_scan.push(dir);
                }
            }
        }

        for dir in dirs_to_scan {
            if !dir.exists() {
                continue;
            }

            let entries = match std::fs::read_dir(&dir) {
                Ok(e) => e,
                Err(_) => continue,
            };

            for entry in entries.flatten() {
                let path = entry.path();
                match prepare_bundle(&path, &self.cache_dir) {
                    Ok(bundle) => {
                        match load_native_extension(&bundle.binary_path, &mut self.registry) {
                            Ok(loaded) => {
                                // Prevent duplicate loading by extension id
                                if !self
                                    .loaded_extensions
                                    .iter()
                                    .any(|e| e.manifest.id == loaded.manifest.id)
                                {
                                    println!(
                                        "[Opsis] Loaded extension: {} v{} ({})",
                                        loaded.manifest.name,
                                        loaded.manifest.version,
                                        loaded.manifest.id
                                    );
                                    self.loaded_extensions.push(loaded);
                                }
                            }
                            Err(err) => {
                                eprintln!(
                                    "[Opsis Extension Error] Failed to load {:?}: {}",
                                    path, err
                                );
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("[Opsis Extension Warning] Skipping {:?}: {}", path, err);
                    }
                }
            }
        }
    }

    /// Render the primary viewport using the registered ViewportProvider extension.
    pub fn render_viewport(&self, ctx: &ViewportContext) -> Option<Element> {
        for provider in &self.registry.viewport_providers {
            if let Some(el) = provider.render_viewport(ctx) {
                return Some(el);
            }
        }
        None
    }

    /// Render all active overlay components registered by extensions.
    pub fn render_overlays(&self, ctx: &OverlayContext) -> Vec<Element> {
        self.registry
            .overlay_providers
            .iter()
            .filter_map(|provider| provider.render_overlay(ctx))
            .collect()
    }

    /// Apply active image filters in sequence to a raw RGBA pixel buffer.
    pub fn apply_image_filters(
        &self,
        rgba: &[u8],
        width: u32,
        height: u32,
    ) -> Option<bytes::Bytes> {
        let mut current_bytes: Option<bytes::Bytes> = None;

        for filter in &self.registry.image_filter_providers {
            let src = current_bytes.as_deref().unwrap_or(rgba);
            if let Some(filtered) = filter.apply_filter(src, width, height) {
                current_bytes = Some(filtered);
            }
        }

        current_bytes
    }

    /// Dispatch input events through registered input interceptors.
    pub fn dispatch_input(&mut self, event: &InputEvent, ctx: &InputContext) -> EventAction {
        for interceptor in &mut self.registry.input_interceptors {
            if interceptor.on_input(event, ctx) == EventAction::Handled {
                return EventAction::Handled;
            }
        }
        EventAction::Pass
    }

    /// Return the count of actively loaded extensions.
    #[allow(dead_code)]
    pub fn extension_count(&self) -> usize {
        self.loaded_extensions.len()
    }

    /// Return the path to the primary extensions directory.
    #[allow(dead_code)]
    pub fn extensions_dir(&self) -> &Path {
        &self.extensions_dir
    }
}

impl Drop for ExtensionManager {
    fn drop(&mut self) {
        for ext in &mut self.loaded_extensions {
            ext.instance.on_unload();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation_and_discovery() {
        let manager = ExtensionManager::new();
        assert!(manager.extensions_dir().exists());
    }

    #[test]
    fn test_system_user_extensions_dir() {
        let dir = get_system_user_extensions_dir();
        assert!(!dir.as_os_str().is_empty());
    }
}
