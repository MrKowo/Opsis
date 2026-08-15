use opsis_extension_api::{
    ActionDefinition, ActionHandler, EventAction, ExtensionRegistry, InputContext,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

pub const KEYBINDINGS_FILENAME: &str = "keybindings.json";

/// Built-in core actions provided by the host application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoreAction {
    OpenImage,
    NextImage,
    PrevImage,
    ZoomIn,
    ZoomOut,
    ResetZoom,
    ToggleFitAxis,
    ToggleMaximize,
    ToggleSidebar,
    ToggleZenMode,
    ClearImage,
    OpenSettings,
    CloseWindow,
}

/// Result of evaluating a key press through the centralized dispatch engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyDispatchResult {
    /// An extension action or input interceptor handled and consumed the key event.
    Handled,
    /// A core built-in action was triggered.
    Core(CoreAction),
    /// No registered action matched; pass key to default handlers.
    Pass,
}

/// Structured item representing an action for UI rendering in Settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionDisplayItem {
    pub id: String,
    pub name: String,
    pub category: String,
    pub keys_display: String,
    pub is_customized: bool,
    pub default_keys_display: String,
    pub description: String,
}

/// Holds an action definition and any user-customized key overrides.
#[derive(Debug, Clone)]
pub struct ActionBinding {
    pub definition: ActionDefinition,
    pub custom_keys: Option<Vec<String>>,
}

impl ActionBinding {
    pub fn new(definition: ActionDefinition) -> Self {
        Self {
            definition,
            custom_keys: None,
        }
    }

    pub fn active_keys(&self) -> &[String] {
        self.custom_keys
            .as_deref()
            .unwrap_or(&self.definition.default_keys)
    }

    pub fn is_customized(&self) -> bool {
        self.custom_keys.is_some()
    }

    /// Check if the given input key string matches any of this action's active keys.
    pub fn matches_key(&self, key_str: &str) -> bool {
        let key_normalized = normalize_key_str(key_str);
        for k in self.active_keys() {
            if normalize_key_str(k) == key_normalized {
                return true;
            }
        }
        false
    }
}

/// Normalize key strings for comparison (case-insensitive for single chars, normalized special keys).
pub fn normalize_key_str(key: &str) -> String {
    let trimmed = key.trim();
    match trimmed.to_lowercase().as_str() {
        " " | "space" => "space".to_string(),
        "arrowright" | "right" => "arrowright".to_string(),
        "arrowleft" | "left" => "arrowleft".to_string(),
        "arrowup" | "up" => "arrowup".to_string(),
        "arrowdown" | "down" => "arrowdown".to_string(),
        "esc" | "escape" => "escape".to_string(),
        "return" | "enter" => "enter".to_string(),
        "pagedown" | "pgdn" => "pagedown".to_string(),
        "pageup" | "pgup" => "pageup".to_string(),
        "backspace" | "back" => "backspace".to_string(),
        "delete" | "del" => "delete".to_string(),
        other => other.to_string(),
    }
}

/// Format key names for clean, readable UI display.
pub fn format_single_key_display(key: &str) -> String {
    let normalized = normalize_key_str(key);
    match normalized.as_str() {
        "space" => "Space".to_string(),
        "arrowright" => "Right".to_string(),
        "arrowleft" => "Left".to_string(),
        "arrowup" => "Up".to_string(),
        "arrowdown" => "Down".to_string(),
        "escape" => "Escape".to_string(),
        "enter" => "Enter".to_string(),
        "pagedown" => "PageDown".to_string(),
        "pageup" => "PageUp".to_string(),
        "backspace" => "Backspace".to_string(),
        "delete" => "Delete".to_string(),
        other => {
            if other.len() == 1 {
                other.to_uppercase()
            } else {
                key.to_string()
            }
        }
    }
}

/// Format multiple keys into a consolidated display string (e.g. `Right / PageDown / Space / D / N`).
pub fn format_keys_display(keys: &[String]) -> String {
    if keys.is_empty() {
        return "None".to_string();
    }

    let mut formatted_unique = Vec::new();
    for k in keys {
        let label = format_single_key_display(k);
        if !formatted_unique.contains(&label) {
            formatted_unique.push(label);
        }
    }

    formatted_unique.join(" / ")
}

/// Central Hotkey & Command Registry managing core and extension actions.
pub struct HotkeyRegistry {
    pub core_actions: Vec<(CoreAction, ActionBinding)>,
    pub extension_actions: Vec<(ActionBinding, Box<dyn ActionHandler>)>,
    pub custom_overrides: HashMap<String, Vec<String>>,
}

impl Default for HotkeyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl HotkeyRegistry {
    /// Initialize a new HotkeyRegistry with standard built-in core actions.
    pub fn new() -> Self {
        let core_definitions = vec![
            (
                CoreAction::OpenImage,
                ActionDefinition {
                    id: "core.open_image".to_string(),
                    name: "Open Image File".to_string(),
                    category: "File".to_string(),
                    default_keys: vec!["o".to_string(), "O".to_string()],
                    description: "Open native file dialog to select an image".to_string(),
                },
            ),
            (
                CoreAction::NextImage,
                ActionDefinition {
                    id: "core.next_image".to_string(),
                    name: "Next Image in Folder".to_string(),
                    category: "Navigation".to_string(),
                    default_keys: vec![
                        "ArrowRight".to_string(),
                        "PageDown".to_string(),
                        "Space".to_string(),
                        "d".to_string(),
                        "D".to_string(),
                        "l".to_string(),
                        "L".to_string(),
                    ],
                    description: "Cycle forward to the next image in the current directory".to_string(),
                },
            ),
            (
                CoreAction::PrevImage,
                ActionDefinition {
                    id: "core.prev_image".to_string(),
                    name: "Previous Image in Folder".to_string(),
                    category: "Navigation".to_string(),
                    default_keys: vec![
                        "ArrowLeft".to_string(),
                        "PageUp".to_string(),
                        "Backspace".to_string(),
                        "p".to_string(),
                        "P".to_string(),
                        "a".to_string(),
                        "A".to_string(),
                    ],
                    description: "Cycle backward to previous image in the current directory".to_string(),
                },
            ),
            (
                CoreAction::ZoomIn,
                ActionDefinition {
                    id: "core.zoom_in".to_string(),
                    name: "Zoom In (+25%)".to_string(),
                    category: "View".to_string(),
                    default_keys: vec!["+".to_string(), "=".to_string()],
                    description: "Enlarge the image viewport by 25%".to_string(),
                },
            ),
            (
                CoreAction::ZoomOut,
                ActionDefinition {
                    id: "core.zoom_out".to_string(),
                    name: "Zoom Out (-25%)".to_string(),
                    category: "View".to_string(),
                    default_keys: vec!["-".to_string(), "_".to_string()],
                    description: "Reduce the image viewport by 25%".to_string(),
                },
            ),
            (
                CoreAction::ResetZoom,
                ActionDefinition {
                    id: "core.reset_zoom".to_string(),
                    name: "100% Original Size (1:1)".to_string(),
                    category: "View".to_string(),
                    default_keys: vec!["0".to_string()],
                    description: "Reset viewport zoom to 100% pixel scale".to_string(),
                },
            ),
            (
                CoreAction::ToggleFitAxis,
                ActionDefinition {
                    id: "core.toggle_fit_axis".to_string(),
                    name: "Toggle Fit Width / Height".to_string(),
                    category: "View".to_string(),
                    default_keys: vec!["h".to_string(), "H".to_string()],
                    description: "Toggle auto-fitting image horizontally or vertically".to_string(),
                },
            ),
            (
                CoreAction::ToggleMaximize,
                ActionDefinition {
                    id: "core.toggle_maximize".to_string(),
                    name: "Toggle Window Maximize".to_string(),
                    category: "Window".to_string(),
                    default_keys: vec!["f".to_string(), "F".to_string()],
                    description: "Maximize or restore the main window".to_string(),
                },
            ),
            (
                CoreAction::ToggleSidebar,
                ActionDefinition {
                    id: "core.toggle_sidebar".to_string(),
                    name: "Toggle Sidebar (N-Panel)".to_string(),
                    category: "View".to_string(),
                    default_keys: vec!["n".to_string(), "N".to_string()],
                    description: "Open or close the right metadata & tool sidebar".to_string(),
                },
            ),
            (
                CoreAction::ToggleZenMode,
                ActionDefinition {
                    id: "core.toggle_zen_mode".to_string(),
                    name: "Toggle Zen Mode (Hide UI)".to_string(),
                    category: "View".to_string(),
                    default_keys: vec!["Tab".to_string()],
                    description: "Toggle full distraction-free image-only view".to_string(),
                },
            ),
            (
                CoreAction::ClearImage,
                ActionDefinition {
                    id: "core.clear_image".to_string(),
                    name: "Clear Loaded Image".to_string(),
                    category: "File".to_string(),
                    default_keys: vec!["Escape".to_string()],
                    description: "Unload active image and return to base canvas".to_string(),
                },
            ),
            (
                CoreAction::OpenSettings,
                ActionDefinition {
                    id: "core.open_settings".to_string(),
                    name: "Open Settings".to_string(),
                    category: "System".to_string(),
                    default_keys: vec!["s".to_string(), "S".to_string()],
                    description: "Open the native Settings and Preferences window".to_string(),
                },
            ),
            (
                CoreAction::CloseWindow,
                ActionDefinition {
                    id: "core.close_window".to_string(),
                    name: "Close Window / Exit".to_string(),
                    category: "System".to_string(),
                    default_keys: vec!["q".to_string(), "Q".to_string()],
                    description: "Close active focused window or exit application".to_string(),
                },
            ),
        ];

        let core_actions = core_definitions
            .into_iter()
            .map(|(action, def)| (action, ActionBinding::new(def)))
            .collect();

        Self {
            core_actions,
            extension_actions: Vec::new(),
            custom_overrides: HashMap::new(),
        }
    }

    /// Import new actions registered by loaded extensions into the central registry.
    pub fn sync_extension_actions(&mut self, registry: &mut ExtensionRegistry) {
        for (def, handler) in registry.registered_actions.drain(..) {
            // Prevent duplicate registration by action id
            if !self.extension_actions.iter().any(|(b, _)| b.definition.id == def.id) {
                let mut binding = ActionBinding::new(def);
                if let Some(custom) = self.custom_overrides.get(&binding.definition.id) {
                    binding.custom_keys = Some(custom.clone());
                }
                self.extension_actions.push((binding, handler));
            }
        }
    }

    /// Load custom keybindings from disk if present. Gracefully uses in-memory defaults if file is absent.
    pub fn load_keybindings(&mut self, config_dir: &Path) {
        let file_path = config_dir.join(KEYBINDINGS_FILENAME);
        if !file_path.exists() {
            return;
        }

        if let Ok(data) = std::fs::read(&file_path) {
            if let Ok(overrides) = serde_json::from_slice::<HashMap<String, Vec<String>>>(&data) {
                self.custom_overrides = overrides;

                // Apply to core actions
                for (_, binding) in &mut self.core_actions {
                    if let Some(custom) = self.custom_overrides.get(&binding.definition.id) {
                        binding.custom_keys = Some(custom.clone());
                    }
                }

                // Apply to extension actions
                for (binding, _) in &mut self.extension_actions {
                    if let Some(custom) = self.custom_overrides.get(&binding.definition.id) {
                        binding.custom_keys = Some(custom.clone());
                    }
                }

                crate::log_hotkey!(
                    "Loaded {} custom hotkey override(s) from '{}'",
                    self.custom_overrides.len(),
                    file_path.display()
                );
            } else {
                eprintln!(
                    "[Opsis Hotkeys] Warning: Could not parse '{:?}'. Using default keybindings.",
                    file_path
                );
            }
        }
    }

    /// Save active custom keybindings to disk (lazy persistence: only writes if overrides exist).
    pub fn save_keybindings(&self, config_dir: &Path) {
        let file_path = config_dir.join(KEYBINDINGS_FILENAME);

        if self.custom_overrides.is_empty() {
            // Clean up file if no custom overrides remain
            if file_path.exists() {
                let _ = std::fs::remove_file(&file_path);
                crate::log_hotkey!("Cleaned up '{}' (all bindings at default)", file_path.display());
            }
            return;
        }

        let _ = std::fs::create_dir_all(config_dir);
        if let Ok(json) = serde_json::to_string_pretty(&self.custom_overrides) {
            if std::fs::write(&file_path, json).is_ok() {
                crate::log_hotkey!(
                    "Persisted {} custom hotkey override(s) to '{}'",
                    self.custom_overrides.len(),
                    file_path.display()
                );
            }
        }
    }

    /// Rebind an action by its ID to a new key trigger and persist to disk.
    pub fn rebind_action(&mut self, action_id: &str, new_key: String, config_dir: &Path) {
        crate::log_hotkey!("Rebinding action '{}' -> Key: '{}'", action_id, new_key);
        let keys = vec![new_key];
        self.custom_overrides.insert(action_id.to_string(), keys.clone());

        for (_, binding) in &mut self.core_actions {
            if binding.definition.id == action_id {
                binding.custom_keys = Some(keys.clone());
            }
        }

        for (binding, _) in &mut self.extension_actions {
            if binding.definition.id == action_id {
                binding.custom_keys = Some(keys.clone());
            }
        }

        self.save_keybindings(config_dir);
    }

    /// Reset an action by its ID to factory default keybindings.
    pub fn reset_action(&mut self, action_id: &str, config_dir: &Path) {
        crate::log_hotkey!("Resetting action '{}' to default bindings", action_id);
        self.custom_overrides.remove(action_id);

        for (_, binding) in &mut self.core_actions {
            if binding.definition.id == action_id {
                binding.custom_keys = None;
            }
        }

        for (binding, _) in &mut self.extension_actions {
            if binding.definition.id == action_id {
                binding.custom_keys = None;
            }
        }

        self.save_keybindings(config_dir);
    }

    /// Reset all custom keybindings to factory defaults and remove configuration file.
    pub fn reset_all(&mut self, config_dir: &Path) {
        crate::log_hotkey!("Reset all hotkeys to factory defaults");
        self.custom_overrides.clear();

        for (_, binding) in &mut self.core_actions {
            binding.custom_keys = None;
        }

        for (binding, _) in &mut self.extension_actions {
            binding.custom_keys = None;
        }

        self.save_keybindings(config_dir);
    }

    /// Dispatch a key string event: checks extension action handlers first, then core actions.
    pub fn dispatch_key(&mut self, key_str: &str, ctx: &InputContext) -> KeyDispatchResult {
        // 1. Check extension-registered actions
        for (binding, handler) in &mut self.extension_actions {
            if binding.matches_key(key_str)
                && handler.execute(&binding.definition.id, ctx) == EventAction::Handled
            {
                return KeyDispatchResult::Handled;
            }
        }

        // 2. Check core built-in actions
        for (action, binding) in &self.core_actions {
            if binding.matches_key(key_str) {
                return KeyDispatchResult::Core(*action);
            }
        }

        KeyDispatchResult::Pass
    }

    /// Return all active actions formatted for UI display in Settings &rarr; Shortcuts.
    pub fn all_actions_for_display(&self) -> Vec<ActionDisplayItem> {
        let mut items = Vec::new();

        // Core actions
        for (_, binding) in &self.core_actions {
            items.push(ActionDisplayItem {
                id: binding.definition.id.clone(),
                name: binding.definition.name.clone(),
                category: binding.definition.category.clone(),
                keys_display: format_keys_display(binding.active_keys()),
                is_customized: binding.is_customized(),
                default_keys_display: format_keys_display(&binding.definition.default_keys),
                description: binding.definition.description.clone(),
            });
        }

        // Extension actions
        for (binding, _) in &self.extension_actions {
            items.push(ActionDisplayItem {
                id: binding.definition.id.clone(),
                name: binding.definition.name.clone(),
                category: binding.definition.category.clone(),
                keys_display: format_keys_display(binding.active_keys()),
                is_customized: binding.is_customized(),
                default_keys_display: format_keys_display(&binding.definition.default_keys),
                description: binding.definition.description.clone(),
            });
        }

        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyActionHandler;
    impl ActionHandler for DummyActionHandler {
        fn execute(&mut self, _id: &str, _ctx: &InputContext) -> EventAction {
            EventAction::Handled
        }
    }

    fn dummy_input_context() -> InputContext {
        InputContext {
            image_path: None,
            window_size: (800.0, 600.0),
            extensions_dir: std::path::PathBuf::from("extensions"),
            installed_extensions: Vec::new(),
            launch_window: None,
        }
    }

    #[test]
    fn test_core_action_dispatch_defaults() {
        let mut registry = HotkeyRegistry::new();
        let ctx = dummy_input_context();

        // Test NextImage default keys
        assert_eq!(
            registry.dispatch_key("ArrowRight", &ctx),
            KeyDispatchResult::Core(CoreAction::NextImage)
        );
        assert_eq!(
            registry.dispatch_key("PageDown", &ctx),
            KeyDispatchResult::Core(CoreAction::NextImage)
        );
        assert_eq!(
            registry.dispatch_key("d", &ctx),
            KeyDispatchResult::Core(CoreAction::NextImage)
        );

        // Test ToggleSidebar (N)
        assert_eq!(
            registry.dispatch_key("n", &ctx),
            KeyDispatchResult::Core(CoreAction::ToggleSidebar)
        );
        assert_eq!(
            registry.dispatch_key("N", &ctx),
            KeyDispatchResult::Core(CoreAction::ToggleSidebar)
        );

        // Test ToggleZenMode (Tab)
        assert_eq!(
            registry.dispatch_key("Tab", &ctx),
            KeyDispatchResult::Core(CoreAction::ToggleZenMode)
        );

        // Test ZoomIn
        assert_eq!(
            registry.dispatch_key("+", &ctx),
            KeyDispatchResult::Core(CoreAction::ZoomIn)
        );
        assert_eq!(
            registry.dispatch_key("=", &ctx),
            KeyDispatchResult::Core(CoreAction::ZoomIn)
        );

        // Test unmapped key
        assert_eq!(
            registry.dispatch_key("UnmappedKey", &ctx),
            KeyDispatchResult::Pass
        );
    }

    #[test]
    fn test_rebind_and_reset_action() {
        let temp_dir = std::env::temp_dir().join("opsis_test_hotkeys_rebind");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let mut registry = HotkeyRegistry::new();
        let ctx = dummy_input_context();

        // Initially 'j' is not mapped
        assert_eq!(registry.dispatch_key("j", &ctx), KeyDispatchResult::Pass);

        // Rebind NextImage to 'j'
        registry.rebind_action("core.next_image", "j".to_string(), &temp_dir);

        // 'j' now triggers NextImage
        assert_eq!(
            registry.dispatch_key("j", &ctx),
            KeyDispatchResult::Core(CoreAction::NextImage)
        );
        assert_eq!(
            registry.dispatch_key("J", &ctx),
            KeyDispatchResult::Core(CoreAction::NextImage)
        );

        // Check persistence file was written
        assert!(temp_dir.join(KEYBINDINGS_FILENAME).exists());

        // Load into fresh registry
        let mut fresh = HotkeyRegistry::new();
        fresh.load_keybindings(&temp_dir);
        assert_eq!(
            fresh.dispatch_key("j", &ctx),
            KeyDispatchResult::Core(CoreAction::NextImage)
        );

        // Reset single action
        fresh.reset_action("core.next_image", &temp_dir);
        assert_eq!(fresh.dispatch_key("j", &ctx), KeyDispatchResult::Pass);
        assert_eq!(
            fresh.dispatch_key("ArrowRight", &ctx),
            KeyDispatchResult::Core(CoreAction::NextImage)
        );

        // Rebind again and reset all
        fresh.rebind_action("core.zoom_in", "z".to_string(), &temp_dir);
        fresh.reset_all(&temp_dir);
        assert_eq!(fresh.dispatch_key("z", &ctx), KeyDispatchResult::Pass);
        // File should be deleted when no overrides remain
        assert!(!temp_dir.join(KEYBINDINGS_FILENAME).exists());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_extension_action_registration_and_dispatch() {
        let mut registry = HotkeyRegistry::new();
        let mut ext_reg = ExtensionRegistry::new();
        let ctx = dummy_input_context();

        ext_reg.register_action(
            ActionDefinition {
                id: "ext.custom".to_string(),
                name: "Custom Action".to_string(),
                category: "Custom".to_string(),
                default_keys: vec!["k".to_string()],
                description: "Custom extension action".to_string(),
            },
            Box::new(DummyActionHandler),
        );

        registry.sync_extension_actions(&mut ext_reg);

        assert_eq!(
            registry.dispatch_key("k", &ctx),
            KeyDispatchResult::Handled
        );
    }
}
