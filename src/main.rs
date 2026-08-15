#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bundle;
mod canvas;
mod file_io;
mod hotkeys;
mod loader;
pub mod log;
mod manager;
mod settings;
pub mod ui;
mod window;

use manager::ExtensionManager;
use std::sync::{Arc, Mutex};

fn main() {
    let cli = log::parse_cli_args(std::env::args_os().map(|s| s.to_string_lossy().into_owned()));

    if cli.print_help {
        log::print_help_screen();
        return;
    }

    if cli.print_version {
        println!("Opsis v{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    // Initialize logging level from flags or environment (OPSIS_LOG / RUST_LOG)
    log::init_logging_from_args_and_env(cli.log_level);

    let path = cli.image_path.filter(|p| p.is_file());

    // Initialize extension manager (non-blocking, sets up paths and empty registry)
    let extension_manager = Arc::new(Mutex::new(ExtensionManager::new()));

    // Launch window host (extension discovery & loading occurs in parallel in the background)
    window::run(path, extension_manager);
}
