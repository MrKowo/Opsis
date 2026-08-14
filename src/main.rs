#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bundle;
mod canvas;
mod file_io;
mod loader;
mod manager;
mod settings;
mod window;

use manager::ExtensionManager;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

fn main() {
    let path = std::env::args_os().nth(1).map(PathBuf::from).filter(|p| p.is_file());

    // Initialize extension manager and load extensions
    let extension_manager = Arc::new(Mutex::new(ExtensionManager::new()));

    // Launch window host
    window::run(path, extension_manager);
}
