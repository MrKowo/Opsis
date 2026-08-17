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

use crate::config::AppSettings;
use crate::hotkeys::{HotkeyRegistry, KeyDispatchResult};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Central Extension Manager acting as the foundation layer of Opsis.
pub struct ExtensionManager {
    #[allow(dead_code)]
    pub is_portable: bool,
    pub extensions_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub config_dir: PathBuf,
    pub registry: ExtensionRegistry,
    pub loaded_extensions: Vec<LoadedExtension>,
    pub hotkey_registry: HotkeyRegistry,
    pub settings: AppSettings,
    pub is_loading: Arc<AtomicBool>,
}

impl Default for ExtensionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Load and initialize a candidate extension file (.opx or dynamic library).
fn load_candidate_extension(
    path: &Path,
    cache_dir: &Path,
) -> Result<(LoadedExtension, ExtensionRegistry), String> {
    let bundle = prepare_bundle(path, cache_dir)?;
    let mut temp_registry = ExtensionRegistry::new();
    let loaded = load_native_extension(&bundle.binary_path, &mut temp_registry)?;
    Ok((loaded, temp_registry))
}

impl ExtensionManager {
    /// Initialize the extension manager detecting portable mode vs system user profile mode.
    /// Returns immediately without blocking on extension discovery or dynamic loading.
    pub fn new() -> Self {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        let portable_extensions = exe_dir.join("extensions");
        let cwd_extensions = PathBuf::from("extensions");
        let system_extensions = get_system_user_extensions_dir();

        // If an extensions folder exists adjacent to the binary or in CWD, operate in Portable Mode
        let (is_portable, primary_extensions_dir, cache_dir, config_dir) = if portable_extensions.exists() {
            (
                true,
                portable_extensions,
                exe_dir.join(".extension_cache"),
                exe_dir.clone(),
            )
        } else if cwd_extensions.exists() {
            (
                true,
                cwd_extensions,
                PathBuf::from(".extension_cache"),
                PathBuf::from("."),
            )
        } else {
            // System Profile Mode (e.g. standalone binary on Desktop / Downloads or installed in Program Files)
            let system_base = system_extensions
                .parent()
                .unwrap_or(&system_extensions)
                .to_path_buf();
            let system_cache = system_base.join("cache");
            let _ = std::fs::create_dir_all(&system_extensions);
            let _ = std::fs::create_dir_all(&system_cache);
            (false, system_extensions, system_cache, system_base)
        };

        // Ensure directories exist
        let _ = std::fs::create_dir_all(&primary_extensions_dir);
        let _ = std::fs::create_dir_all(&cache_dir);
        let _ = std::fs::create_dir_all(&config_dir);

        let mut hotkey_registry = HotkeyRegistry::new();
        hotkey_registry.load_keybindings(&config_dir);

        let settings = AppSettings::load_from_dir(&config_dir);

        Self {
            is_portable,
            extensions_dir: primary_extensions_dir,
            cache_dir,
            config_dir,
            registry: ExtensionRegistry::new(),
            loaded_extensions: Vec::new(),
            hotkey_registry,
            settings,
            is_loading: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Save current application settings to config directory.
    pub fn save_settings(&self) {
        self.settings.save_to_dir(&self.config_dir);
    }

    /// Return list of extension directories to scan based on portable vs user mode.
    pub fn candidate_extension_dirs(&self) -> Vec<PathBuf> {
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

        dirs_to_scan
    }

    /// Scan directories and collect candidate extension files (.opx, .dll, .so, .dylib).
    pub fn find_candidate_extension_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();
        for dir in self.candidate_extension_dirs() {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                            let ext_lower = ext.to_lowercase();
                            if matches!(ext_lower.as_str(), "opx" | "dll" | "so" | "dylib") {
                                files.push(path);
                            }
                        }
                    }
                }
            }
        }
        crate::log_ext!("Found {} candidate extension file(s) across scan paths", files.len());
        files
    }

    /// Discover and load all extensions across portable and system directories synchronously.
    pub fn discover_and_load_all(&mut self) {
        let candidate_files = self.find_candidate_extension_files();
        for path in candidate_files {
            match load_candidate_extension(&path, &self.cache_dir) {
                Ok((loaded, temp_registry)) => {
                    // Prevent duplicate loading by extension id
                    if !self
                        .loaded_extensions
                        .iter()
                        .any(|e| e.manifest.id == loaded.manifest.id)
                    {
                        crate::log_ext!(
                            "Loaded extension: {} v{} ({}) from '{}'",
                            loaded.manifest.name,
                            loaded.manifest.version,
                            loaded.manifest.id,
                            path.display()
                        );
                        self.loaded_extensions.push(loaded);
                        self.registry.append(temp_registry);
                        self.hotkey_registry.sync_extension_actions(&mut self.registry);
                    }
                }
                Err(err) => {
                    crate::log_ext!("Warning: Failed to load '{:?}': {}", path, err);
                }
            }
        }
    }

    /// Load extensions concurrently in a dedicated background thread in parallel with window and image display.
    pub fn load_in_background(
        ext_mgr: Arc<std::sync::Mutex<ExtensionManager>>,
        on_update: Option<Arc<dyn Fn() + Send + Sync + 'static>>,
    ) -> std::thread::JoinHandle<()> {
        std::thread::Builder::new()
            .name("opsis-extension-loader".to_string())
            .spawn(move || {
                let (candidate_files, cache_dir) = {
                    if let Ok(mgr) = ext_mgr.lock() {
                        mgr.is_loading.store(true, Ordering::SeqCst);
                        crate::log_ext!("Starting background extension discovery & loading...");
                        (mgr.find_candidate_extension_files(), mgr.cache_dir.clone())
                    } else {
                        return;
                    }
                };

                for path in candidate_files {
                    match load_candidate_extension(&path, &cache_dir) {
                        Ok((loaded, temp_registry)) => {
                            if let Ok(mut mgr) = ext_mgr.lock() {
                                if !mgr
                                    .loaded_extensions
                                    .iter()
                                    .any(|e| e.manifest.id == loaded.manifest.id)
                                {
                                    crate::log_ext!(
                                        "Loaded background extension: {} v{} ({}) from '{}'",
                                        loaded.manifest.name,
                                        loaded.manifest.version,
                                        loaded.manifest.id,
                                        path.display()
                                    );
                                    mgr.loaded_extensions.push(loaded);
                                    mgr.registry.append(temp_registry);
                                    let mut temp_reg = ExtensionRegistry::new();
                                    std::mem::swap(&mut mgr.registry.registered_actions, &mut temp_reg.registered_actions);
                                    mgr.hotkey_registry.sync_extension_actions(&mut temp_reg);
                                }
                            }
                            if let Some(ref cb) = on_update {
                                cb();
                            }
                        }
                        Err(err) => {
                            crate::log_ext!("Warning: Failed to load '{:?}': {}", path, err);
                        }
                    }
                }

                if let Ok(mgr) = ext_mgr.lock() {
                    mgr.is_loading.store(false, Ordering::SeqCst);
                    crate::log_ext!("Background extension loading complete ({} active)", mgr.loaded_extensions.len());
                }

                if let Some(ref cb) = on_update {
                    cb();
                }
            })
            .expect("Failed to spawn extension loader thread")
    }

    /// Check if background extension loading is currently active.
    pub fn is_loading(&self) -> bool {
        self.is_loading.load(Ordering::SeqCst)
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

    /// Dispatch a key event using the centralized HotkeyRegistry (prioritizing extension actions, then core actions).
    pub fn dispatch_key(&mut self, key_str: &str, ctx: &InputContext) -> KeyDispatchResult {
        // Raw input interceptors (legacy/direct input interception)
        if self.dispatch_input(&InputEvent::KeyDown(key_str.to_string()), ctx) == EventAction::Handled {
            return KeyDispatchResult::Handled;
        }

        // Centralized hotkey & action registry
        self.hotkey_registry.dispatch_key(key_str, ctx)
    }

    /// Rebind an action to a new key and persist to disk.
    pub fn rebind_hotkey(&mut self, action_id: &str, new_key: String) {
        self.hotkey_registry.rebind_action(action_id, new_key, &self.config_dir);
    }

    /// Reset an action to its default keybindings.
    pub fn reset_hotkey(&mut self, action_id: &str) {
        self.hotkey_registry.reset_action(action_id, &self.config_dir);
    }

    /// Reset all actions to default keybindings.
    pub fn reset_all_hotkeys(&mut self) {
        self.hotkey_registry.reset_all(&self.config_dir);
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
        let mut manager = ExtensionManager::new();
        assert!(manager.extensions_dir().exists());
        assert!(!manager.is_loading());
        manager.discover_and_load_all();
    }

    #[test]
    fn test_system_user_extensions_dir() {
        let dir = get_system_user_extensions_dir();
        assert!(!dir.as_os_str().is_empty());
    }

    #[test]
    fn test_candidate_dirs_and_background_loading() {
        let manager = Arc::new(std::sync::Mutex::new(ExtensionManager::new()));
        let update_called = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let update_clone = Arc::clone(&update_called);

        let handle = ExtensionManager::load_in_background(
            Arc::clone(&manager),
            Some(Arc::new(move || {
                update_clone.store(true, Ordering::SeqCst);
            })),
        );

        handle.join().expect("Loader thread join");
        let mgr = manager.lock().unwrap();
        assert!(!mgr.is_loading());
        assert!(update_called.load(Ordering::SeqCst));
    }
}
