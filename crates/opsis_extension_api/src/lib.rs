use freya::prelude::Element;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Current API version of the extension specification.
pub const CURRENT_API_VERSION: u32 = 1;

/// Metadata describing an extension.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExtensionManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub api_version: u32,
}

/// Context passed to viewport renderers.
#[derive(Debug, Clone)]
pub struct ViewportContext {
    pub image_path: Option<PathBuf>,
    pub image_bytes: Option<Vec<u8>>,
    pub window_size: (f64, f64),
}

use std::sync::Arc;

/// Redraw trigger function to request a re-render of the native window.
pub type RedrawTriggerFn = Arc<dyn Fn() + Send + Sync + 'static>;

/// Window builder function receiving a redraw trigger and returning a Freya Element.
pub type WindowBuilderFn = Arc<dyn Fn(RedrawTriggerFn) -> Element + Send + Sync + 'static>;

/// Function provided by the host to launch a native floating window.
pub type WindowLauncherFn = Arc<dyn Fn(String, (f64, f64), WindowBuilderFn) + Send + Sync>;

/// Context passed to overlay renderers.
#[derive(Clone)]
pub struct OverlayContext {
    pub image_path: Option<PathBuf>,
    pub window_size: (f64, f64),
    pub extensions_dir: PathBuf,
    pub installed_extensions: Vec<ExtensionManifest>,
    pub launch_window: Option<WindowLauncherFn>,
}

/// Context passed to input interceptors.
#[derive(Clone)]
pub struct InputContext {
    pub image_path: Option<PathBuf>,
    pub window_size: (f64, f64),
    pub extensions_dir: PathBuf,
    pub installed_extensions: Vec<ExtensionManifest>,
    pub launch_window: Option<WindowLauncherFn>,
}

/// User input events dispatched to extensions.
#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    KeyDown(String),
    KeyUp(String),
    PointerMove { x: f64, y: f64 },
    PointerClick { button: u8 },
    Scroll { delta_y: f32 },
}

/// Result of input event handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EventAction {
    #[default]
    Pass,
    Handled,
}

/// Capability trait for extensions that provide the primary viewport renderer.
pub trait ViewportProvider: Send + Sync {
    fn render_viewport(&self, ctx: &ViewportContext) -> Element;
}

/// Capability trait for extensions that inject HUD overlays and UI widgets.
pub trait OverlayProvider: Send + Sync {
    fn render_overlay(&self, ctx: &OverlayContext) -> Option<Element>;
}

/// Capability trait for extensions that intercept and process user input.
pub trait InputInterceptor: Send + Sync {
    fn on_input(&mut self, event: &InputEvent, ctx: &InputContext) -> EventAction;
}

/// Capability registry populated by extensions during `on_init`.
#[derive(Default)]
pub struct ExtensionRegistry {
    pub viewport_providers: Vec<Box<dyn ViewportProvider>>,
    pub overlay_providers: Vec<Box<dyn OverlayProvider>>,
    pub input_interceptors: Vec<Box<dyn InputInterceptor>>,
}

impl ExtensionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_viewport_provider(&mut self, provider: Box<dyn ViewportProvider>) {
        self.viewport_providers.push(provider);
    }

    pub fn register_overlay_provider(&mut self, provider: Box<dyn OverlayProvider>) {
        self.overlay_providers.push(provider);
    }

    pub fn register_input_interceptor(&mut self, interceptor: Box<dyn InputInterceptor>) {
        self.input_interceptors.push(interceptor);
    }
}

/// Main trait that every Opsis extension must implement.
pub trait OpsisExtension: Send + Sync {
    /// Return the metadata manifest for this extension.
    fn manifest(&self) -> ExtensionManifest;

    /// Called on extension initialization to register capabilities.
    fn on_init(&mut self, registry: &mut ExtensionRegistry) -> Result<(), String>;

    /// Called when the extension is unloaded.
    fn on_unload(&mut self) {}
}

/// C-ABI entry point symbol name.
pub const EXTENSION_ENTRYPOINT_SYMBOL: &[u8] = b"opsis_extension_create\0";

/// Function signature for the exported extension constructor.
pub type ExtensionCreateFn = unsafe extern "Rust" fn() -> Box<dyn OpsisExtension>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_serde() {
        let manifest = ExtensionManifest {
            id: "test.ext".to_string(),
            name: "Test Extension".to_string(),
            version: "1.0.0".to_string(),
            author: "Tester".to_string(),
            description: "A test extension".to_string(),
            api_version: 1,
        };

        let json = serde_json::to_string(&manifest).expect("serialize");
        let deserialized: ExtensionManifest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(manifest, deserialized);
    }

    struct MockViewportProvider;
    impl ViewportProvider for MockViewportProvider {
        fn render_viewport(&self, _ctx: &ViewportContext) -> Element {
            freya::prelude::rect().into()
        }
    }

    #[test]
    fn test_registry_registration() {
        let mut registry = ExtensionRegistry::new();
        assert_eq!(registry.viewport_providers.len(), 0);
        registry.register_viewport_provider(Box::new(MockViewportProvider));
        assert_eq!(registry.viewport_providers.len(), 1);
    }
}

