# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-08-14

### Added

- **Microkernel Architecture**: Shell application boots directly into the Extension Manager as its foundational layer.
- **Universal Extension Bundles (`.opx`)**: Cross-platform ZIP package format with automatic OS/arch detection (`windows`, `linux`, `macos`).
- **Binary-Adjacent Discovery**: Discovers and loads extensions dynamically from `<exe_dir>/extensions/` for portable deployments.
- **Extension API (`opsis_extension_api`)**: Trait contracts for pluggable viewports (`ViewportProvider`), UI overlays (`OverlayProvider`), and inputs (`InputInterceptor`).
- **Native Dynamic Loading**: Zero-copy runtime loading via `libloading` with direct memory and GPU hook access.
- **Base Window Canvas**: Foundational dark canvas with watermark.
- **Extension Manager UI Extension (`opsis_extension_manager_ui`)**: First native extension providing a floating overlay button and inspection drawer for all currently loaded extensions.
- **Native Floating Window Capability**: Enabled dynamic runtime OS window creation (`launch_window`) in `OverlayContext`, powering the Extension Manager as a native detached floating window.
- **Base Canvas Keyboard Shortcut & Clean Overlay**: Replaced the overlay pill button with a subtle `"Press S to open settings"` watermark hint and global `S` key listener to launch the native floating settings window.
- **Instant Application Shutdown**: Configured fast-path window close hook (`with_on_close`) to ensure immediate process termination on window close.
- **Pure Microkernel Input Dispatch**: Extracted all Extension Manager window logic out of host into `opsis_extension_manager_ui` via `InputInterceptor`, ensuring `main.rs` remains 100% agnostic.
- **Dedicated Window & Manager Host Separation**: Reorganized host codebase into flat, focused root files (`window.rs`, `manager.rs`, `bundle.rs`, `loader.rs`, and a minimal 20-line `main.rs`).
- **Vertical-Tab Settings & Extension Manager Window**: Built a native floating Settings window (`720x520` px) in `opsis_extension_manager_ui` with 5 vertical navigation tabs (General, Appearance, Extensions with live discovery inspector, Shortcuts, and About).
- **Reactive Tab Navigation & Clean Typography**: Wired `use_state` with `.on_press` event handlers for tab navigation and removed all emojis across the settings window.
- **Built-in Native Settings Window**: Integrated the vertical-tab Settings window directly into the host as `src/settings.rs`, using direct Freya hooks with zero cross-DLL overhead.
- **2D Image Viewport & File Opening Extension (`opsis_image_viewer`)**: Pluggable native extension providing core 2D image rendering, native file picker dialogs (`rfd`), cursor-centered scroll wheel zooming, and click-and-drag panning.
- **Floating Image Status HUD**: Non-intrusive floating HUD overlay displaying image filename, format badge, dimensions, file size, zoom percentage, and quick-action buttons (Zoom In, Zoom Out, 1:1, Fit to Window, Open Image).
- **Keyboard Shortcuts & Drag-and-Drop**: Added hotkey support (`O` for file open, `+`/`-` for zoom, `0` for 100%, `F` for fit, `Escape` to clear) and native file drag-and-drop loading (`on_file_drop`).
- **Core 2D Canvas & File I/O Migration (`src/canvas.rs`, `src/file_io.rs`)**: Integrated 2D image rendering, Skia GPU viewport, cursor-centered scroll zooming, mouse-drag panning, file drag-and-drop, and native file dialogs directly into the core host binary, reserving overlay rendering and metadata HUDs for modular extensions.
- **Settings Placeholder Cleanup**: Removed mock/placeholder configuration cards from General and Appearance tabs while preserving the 5-tab navigation rail, live Extensions inspector, real working Shortcuts list, and About pane.
