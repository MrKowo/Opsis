# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-08-14

### Added

- **Microkernel Architecture**: Minimal host binary that initializes the extension manager and delegates feature handling to modular extensions.
- **Dedicated Window Host (`src/window.rs`)**: Isolated Freya window creation, lifecycle management, Skia 2D rendering, and event dispatching.
- **Core 2D Canvas & File I/O (`src/canvas.rs`, `src/file_io.rs`)**: High-performance Skia viewport with cursor-centered scroll zooming, mouse-drag panning, file drag-and-drop, and native file dialogs (`rfd`).
- **Dual-Location Extension Discovery**: Automatic Portable Mode (`<exe_dir>/extensions/`) vs System User Profile (`%APPDATA%/Opsis/extensions`, `~/.config/opsis/extensions`, `~/Library/Application Support/Opsis/extensions`) discovery.
- **Universal Extension Bundles (`.opx`)**: Cross-platform ZIP package format containing metadata and multi-architecture native dynamic libraries (`.dll`, `.so`, `.dylib`).
- **Zero-Overhead Dynamic Loading (`src/loader.rs`)**: Native dynamic library loading via `libloading` with direct memory and GPU access.
- **Public Extension API (`opsis_extension_api`)**: Modular capability traits including `ViewportProvider`, `OverlayProvider`, `ImageFilterProvider`, and `InputInterceptor`.
- **Post-Processing Image Filter Pipeline**: Sequential pixel filter pipeline in `src/manager.rs` allowing extensions to apply real-time color channel isolations and compositing effects while preserving native pan and zoom.
- **Native Floating Window Spawning**: Extensions can spawn detached, standalone native OS windows at runtime via `launch_window`.
- **Built-in Native Settings Window (`src/settings.rs`)**: Vertical tab navigation (General, Appearance, Extensions, Shortcuts, About) with live extension status inspection.
- **Drag-and-Drop Extension Installer**: Interactive drop zone in Settings &rarr; Extensions allowing instant installation, hot-loading, and list refresh for `.opx`, `.dll`, `.so`, and `.dylib` files.
- **Automated Extension Deployment (`build.rs`)**: Build script that automatically copies the root `extensions/` directory into the active `target/<profile>/extensions/` output directory during `cargo build`.
