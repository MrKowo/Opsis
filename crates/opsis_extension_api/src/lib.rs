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

/// Metadata defining an action/command that can be triggered by hotkeys.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionDefinition {
    pub id: String,
    pub name: String,
    pub category: String,
    pub default_keys: Vec<String>,
    pub description: String,
}

/// Trait for handling executable actions/commands registered by extensions.
pub trait ActionHandler: Send + Sync {
    /// Execute the action. Return `EventAction::Handled` to consume the event.
    fn execute(&mut self, action_id: &str, ctx: &InputContext) -> EventAction;
}

/// Trait for extensions that provide sidebar panel tabs (the Blender N-Panel equivalent).
pub trait SidebarTabProvider: Send + Sync {
    /// Return the display title for this sidebar tab.
    fn tab_title(&self) -> String;

    /// Optional icon or short tag representation.
    fn tab_icon(&self) -> Option<String> {
        None
    }

    /// Render the content of the sidebar tab.
    fn render_tab(&self, ctx: &OverlayContext) -> Option<Element>;
}

/// Registry populated by extensions during initialization.
#[derive(Default)]
pub struct ExtensionRegistry {
    pub viewport_providers: Vec<Box<dyn ViewportProvider>>,
    pub overlay_providers: Vec<Box<dyn OverlayProvider>>,
    pub image_filter_providers: Vec<Box<dyn ImageFilterProvider>>,
    pub input_interceptors: Vec<Box<dyn InputInterceptor>>,
    pub registered_actions: Vec<(ActionDefinition, Box<dyn ActionHandler>)>,
    pub sidebar_tab_providers: Vec<Box<dyn SidebarTabProvider>>,
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

    /// Register a named action/command with default key bindings and an execution handler.
    pub fn register_action(&mut self, definition: ActionDefinition, handler: Box<dyn ActionHandler>) {
        self.registered_actions.push((definition, handler));
    }

    /// Register a custom sidebar panel tab.
    pub fn register_sidebar_tab_provider(&mut self, provider: Box<dyn SidebarTabProvider>) {
        self.sidebar_tab_providers.push(provider);
    }

    /// Append all providers, interceptors, actions, and sidebar tabs from another registry into this one.
    pub fn append(&mut self, mut other: ExtensionRegistry) {
        self.viewport_providers.append(&mut other.viewport_providers);
        self.overlay_providers.append(&mut other.overlay_providers);
        self.image_filter_providers.append(&mut other.image_filter_providers);
        self.input_interceptors.append(&mut other.input_interceptors);
        self.registered_actions.append(&mut other.registered_actions);
        self.sidebar_tab_providers.append(&mut other.sidebar_tab_providers);
    }

    /// Returns true if any image filter providers are registered.
    pub fn has_image_filters(&self) -> bool {
        !self.image_filter_providers.is_empty()
    }

    /// Returns true if any viewport providers are registered.
    pub fn has_viewport_providers(&self) -> bool {
        !self.viewport_providers.is_empty()
    }

    /// Returns true if any overlay providers are registered.
    pub fn has_overlay_providers(&self) -> bool {
        !self.overlay_providers.is_empty()
    }

    /// Returns true if any input interceptors are registered.
    pub fn has_input_interceptors(&self) -> bool {
        !self.input_interceptors.is_empty()
    }

    /// Returns true if any custom actions are registered.
    pub fn has_actions(&self) -> bool {
        !self.registered_actions.is_empty()
    }

    /// Returns true if any sidebar panel tab providers are registered.
    pub fn has_sidebar_tabs(&self) -> bool {
        !self.sidebar_tab_providers.is_empty()
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
        assert!(!registry.has_viewport_providers());
        assert!(!registry.has_image_filters());

        registry.register_viewport_provider(Box::new(MockViewportProvider));
        registry.register_image_filter_provider(Box::new(MockFilterProvider));
        assert_eq!(registry.viewport_providers.len(), 1);
        assert_eq!(registry.image_filter_providers.len(), 1);
        assert!(registry.has_viewport_providers());
        assert!(registry.has_image_filters());
        assert!(!registry.has_actions());

        let mut other = ExtensionRegistry::new();
        other.register_viewport_provider(Box::new(MockViewportProvider));
        registry.append(other);
        assert_eq!(registry.viewport_providers.len(), 2);
    }

    struct MockActionHandler;
    impl ActionHandler for MockActionHandler {
        fn execute(&mut self, _action_id: &str, _ctx: &InputContext) -> EventAction {
            EventAction::Handled
        }
    }

    #[test]
    fn test_action_registration_and_append() {
        let mut registry = ExtensionRegistry::new();
        assert!(!registry.has_actions());

        let action = ActionDefinition {
            id: "test.action".to_string(),
            name: "Test Action".to_string(),
            category: "Testing".to_string(),
            default_keys: vec!["t".to_string(), "T".to_string()],
            description: "A test action".to_string(),
        };

        registry.register_action(action, Box::new(MockActionHandler));
        assert!(registry.has_actions());
        assert_eq!(registry.registered_actions.len(), 1);

        let mut other = ExtensionRegistry::new();
        other.register_action(
            ActionDefinition {
                id: "other.action".to_string(),
                name: "Other Action".to_string(),
                category: "Testing".to_string(),
                default_keys: vec!["x".to_string()],
                description: "Another action".to_string(),
            },
            Box::new(MockActionHandler),
        );

        registry.append(other);
        assert_eq!(registry.registered_actions.len(), 2);
    }

    struct MockSidebarTab;
    impl SidebarTabProvider for MockSidebarTab {
        fn tab_title(&self) -> String {
            "Mock Tab".to_string()
        }
        fn render_tab(&self, _ctx: &OverlayContext) -> Option<Element> {
            Some(freya::prelude::rect().into())
        }
    }

    #[test]
    fn test_sidebar_tab_registration_and_append() {
        let mut registry = ExtensionRegistry::new();
        assert!(!registry.has_sidebar_tabs());

        registry.register_sidebar_tab_provider(Box::new(MockSidebarTab));
        assert!(registry.has_sidebar_tabs());
        assert_eq!(registry.sidebar_tab_providers.len(), 1);

        let mut other = ExtensionRegistry::new();
        other.register_sidebar_tab_provider(Box::new(MockSidebarTab));
        registry.append(other);
        assert_eq!(registry.sidebar_tab_providers.len(), 2);
    }
}
