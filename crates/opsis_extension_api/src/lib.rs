use freya::prelude::Element;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const CURRENT_API_VERSION: u32 = 1;

/// Metadata describing an Opsis extension.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// Trait for extensions that provide primary canvas/viewport rendering.
pub trait ViewportProvider: Send + Sync {
    /// Render the primary viewport. Return `None` to let the default viewport or another provider render.
    fn render_viewport(&self, ctx: &ViewportContext) -> Option<Element>;
}

/// Trait for extensions that inject HUD or overlay elements on top of the viewport.
pub trait OverlayProvider: Send + Sync {
    /// Render UI overlays layered on top of the viewport canvas.
    fn render_overlay(&self, ctx: &OverlayContext) -> Option<Element>;
}

/// Trait for extensions applying post-processing pixel filters to the core rendered image.
pub trait ImageFilterProvider: Send + Sync {
    /// Apply a filter transformation to the decoded RGBA buffer. Return `None` if inactive.
    fn apply_filter(&self, rgba: &[u8], width: u32, height: u32) -> Option<bytes::Bytes>;
}

/// Trait for extensions that intercept keyboard and pointer input events.
pub trait InputInterceptor: Send + Sync {
    /// Intercept input event before default host handlers. Return `EventAction::Handled` to consume.
    fn on_input(&mut self, event: &InputEvent, ctx: &InputContext) -> EventAction;
}

/// Registry populated by extensions during initialization.
#[derive(Default)]
pub struct ExtensionRegistry {
    pub viewport_providers: Vec<Box<dyn ViewportProvider>>,
    pub overlay_providers: Vec<Box<dyn OverlayProvider>>,
    pub image_filter_providers: Vec<Box<dyn ImageFilterProvider>>,
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

    pub fn register_image_filter_provider(&mut self, provider: Box<dyn ImageFilterProvider>) {
        self.image_filter_providers.push(provider);
    }

    pub fn register_input_interceptor(&mut self, interceptor: Box<dyn InputInterceptor>) {
        self.input_interceptors.push(interceptor);
    }
}

/// Core extension lifecycle trait implemented by dynamic library extensions.
pub trait OpsisExtension: Send + Sync {
    /// Returns metadata and capabilities manifest for this extension.
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
        fn render_viewport(&self, _ctx: &ViewportContext) -> Option<Element> {
            Some(freya::prelude::rect().into())
        }
    }

    struct MockFilterProvider;
    impl ImageFilterProvider for MockFilterProvider {
        fn apply_filter(&self, _rgba: &[u8], _w: u32, _h: u32) -> Option<bytes::Bytes> {
            None
        }
    }

    #[test]
    fn test_registry_registration() {
        let mut registry = ExtensionRegistry::new();
        registry.register_viewport_provider(Box::new(MockViewportProvider));
        registry.register_image_filter_provider(Box::new(MockFilterProvider));
        assert_eq!(registry.viewport_providers.len(), 1);
        assert_eq!(registry.image_filter_providers.len(), 1);
    }
}
