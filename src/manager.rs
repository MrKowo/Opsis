use freya::prelude::Element;
use opsis_extension_api::{
    EventAction, ExtensionRegistry, InputContext, InputEvent, OverlayContext, ViewportContext,
};
use std::path::{Path, PathBuf};

use crate::bundle::prepare_bundle;
use crate::loader::{load_native_extension, LoadedExtension};

/// Central Extension Manager acting as the foundation layer of Opsis.
pub struct ExtensionManager {
    pub extensions_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub registry: ExtensionRegistry,
    pub loaded_extensions: Vec<LoadedExtension>,
}

impl ExtensionManager {
    /// Initialize the extension manager relative to the binary executable.
    pub fn new() -> Self {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        let extensions_dir = exe_dir.join("extensions");
        let cache_dir = exe_dir.join(".extension_cache");

        // Ensure directories exist
        let _ = std::fs::create_dir_all(&extensions_dir);
        let _ = std::fs::create_dir_all(&cache_dir);

        let mut manager = Self {
            extensions_dir,
            cache_dir,
            registry: ExtensionRegistry::new(),
            loaded_extensions: Vec::new(),
        };

        manager.discover_and_load_all();
        manager
    }

    /// Discover and load all extensions located in <exe_dir>/extensions/ or ./extensions/.
    pub fn discover_and_load_all(&mut self) {
        let mut dirs_to_scan = vec![self.extensions_dir.clone()];
        let cwd_extensions = PathBuf::from("extensions");
        if cwd_extensions.exists() && cwd_extensions != self.extensions_dir {
            dirs_to_scan.push(cwd_extensions);
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
                                if !self.loaded_extensions.iter().any(|e| e.manifest.id == loaded.manifest.id) {
                                    println!(
                                        "[Opsis] Loaded extension: {} v{} ({})",
                                        loaded.manifest.name, loaded.manifest.version, loaded.manifest.id
                                    );
                                    self.loaded_extensions.push(loaded);
                                }
                            }
                            Err(err) => {
                                eprintln!("[Opsis Extension Error] Failed to load {:?}: {}", path, err);
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
        self.registry
            .viewport_providers
            .first()
            .map(|provider| provider.render_viewport(ctx))
    }

    /// Render all active overlay components registered by extensions.
    pub fn render_overlays(&self, ctx: &OverlayContext) -> Vec<Element> {
        self.registry
            .overlay_providers
            .iter()
            .filter_map(|provider| provider.render_overlay(ctx))
            .collect()
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

    /// Return the path to the extensions directory.
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
}
