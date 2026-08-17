use serde::{Deserialize, Serialize};
use std::path::Path;

const SETTINGS_FILENAME: &str = "settings.json";

/// Global application settings and visual preferences.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppSettings {
    /// Enable dark theme mode (default true).
    pub dark_mode: bool,
    /// Show the Opsis watermark on empty canvas (default true).
    pub show_watermark: bool,
    /// Enable hardware-accelerated frosted acrylic backdrop on the canvas (default false).
    pub acrylic_background: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            dark_mode: true,
            show_watermark: true,
            acrylic_background: false,
        }
    }
}

impl AppSettings {
    /// Load settings from the given config directory, or fallback to defaults.
    pub fn load_from_dir(config_dir: &Path) -> Self {
        let file_path = config_dir.join(SETTINGS_FILENAME);
        if file_path.is_file() {
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                if let Ok(settings) = serde_json::from_str::<AppSettings>(&content) {
                    crate::log_io!("Loaded application settings from '{}'", file_path.display());
                    return settings;
                }
            }
        }
        Self::default()
    }

    /// Save current settings to the given config directory.
    pub fn save_to_dir(&self, config_dir: &Path) {
        let _ = std::fs::create_dir_all(config_dir);
        let file_path = config_dir.join(SETTINGS_FILENAME);
        if let Ok(json) = serde_json::to_string_pretty(self) {
            if std::fs::write(&file_path, json).is_ok() {
                crate::log_io!("Persisted application settings to '{}'", file_path.display());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_settings_default_and_serde() {
        let default_settings = AppSettings::default();
        assert!(default_settings.dark_mode);
        assert!(default_settings.show_watermark);
        assert!(!default_settings.acrylic_background);

        let json = serde_json::to_string(&default_settings).unwrap();
        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(default_settings, deserialized);
    }

    #[test]
    fn test_app_settings_file_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!("opsis_test_config_{}", std::process::id()));
        let mut settings = AppSettings::default();
        settings.acrylic_background = true;
        settings.save_to_dir(&temp_dir);

        let loaded = AppSettings::load_from_dir(&temp_dir);
        assert!(loaded.acrylic_background);

        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
