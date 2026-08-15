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
- **Application Icon & Windows Resource Embedding**: Embedded 256x256 icon resource via `winres` for native Windows Explorer and Taskbar support, with runtime window icon configuration across all windows.
- **Universal Window Closing (`Q` / `Escape`)**: Added universal shortcut to cleanly close focused windows or exit the application.
- **Dynamic 50/50 Shortcuts Table & Empty Watermark Hints**: Structured 50/50 centered layout for Settings shortcuts with right-justified keys, and added file open (`O`) hint to base canvas watermark.
- **Executable PE Metadata & Settings Versioning**: Embedded `OriginalFilename`, `InternalName`, `ProductName`, `ProductVersion`, and `FileVersion` in Windows binary properties via `build.rs`, and exposed dynamic package version in Settings &rarr; About.
- **Canonical Extension Discovery & Safe Bundle Caching**: Deduplicated candidate scan directories via canonical path hashing in `src/manager.rs`, added timestamp-based cache revalidation, and graceful lock fallback for loaded dynamic libraries in `src/bundle.rs`.

## [Unreleased]

### Added

- **Parallel Extension Loading (`src/manager.rs`, `src/window.rs`)**: Extension discovery, archive extraction, dynamic library loading (`libloading`), and capability registration now execute concurrently in a dedicated background worker thread without blocking window creation or image display.
- **Instantaneous Image Display & Fast Header Parsing (`src/file_io.rs`, `src/canvas.rs`)**: Sub-millisecond image dimension extraction from file headers (`into_dimensions()`) bypassing redundant full raster decoding during startup, with on-demand RGBA buffer decoding when extension image filters are active.
- **Reactive UI Re-render Channel (`src/window.rs`)**: Freya asynchronous state updates seamlessly mounting extension viewports, overlays, and filter hooks the moment background loading completes.
- **Live Extension Loading Status Indicator (`src/settings.rs`)**: Added real-time background scanning and loading feedback in the Settings &rarr; Extensions panel.
- **Registry Append & Capability Introspection (`crates/opsis_extension_api`)**: Added `ExtensionRegistry::append` and capability helper queries (`has_image_filters`, `has_viewport_providers`, etc.) to support thread-safe incremental extension registration.
- **Folder Image Cycling (`src/file_io.rs`, `src/window.rs`, `src/settings.rs`)**: Automatic directory scanning, natural alphanumeric sorting, and seamless wrap-around navigation through all supported images in the current folder via `ArrowRight`, `ArrowLeft`, `PageDown`, `PageUp`, `Space`, `Backspace`, `N`, `P`, `D`, and `A`.
- **Centralized Command & Rebindable Hotkey Architecture (`src/hotkeys.rs`, `src/manager.rs`, `src/window.rs`, `src/settings.rs`, `crates/opsis_extension_api`)**:
  - Unified `HotkeyRegistry` managing Core host actions and dynamic extension commands.
  - Public extension API (`ActionDefinition`, `ActionHandler`, `ExtensionRegistry::register_action`) allowing extensions to register custom commands with metadata, descriptions, and action callbacks.
  - Dynamic **Settings &rarr; Shortcuts** view presenting only genuine, key-mappable actions grouped by category.
  - Interactive in-app key rebinding with listening mode (`Press key...`), cancellation via `Escape`, custom override badges, inline per-action reset, and global "Reset All Defaults".
  - Zero-config code default fallbacks with lazy JSON persistence (`keybindings.json`) created only upon user customization and cleanly removed on factory reset.

- **Opsis UI Design System & Blender-Inspired Layout Architecture (`src/ui/`, `src/window.rs`, `src/settings.rs`, `crates/opsis_extension_api`)**:
  - **Theme Token Engine (`src/ui/theme.rs`)**: Centralized semantic color palette (`surface_base`, `surface_panel`, `surface_card`, `surface_element`, `accent_primary`, `accent_muted`, `text_primary`, `text_secondary`, `border_subtle`, etc.) and typography / radius metrics.
  - **Reusable Widget Primitives (`src/ui/components.rs`)**: Standardized `key_badge`, `status_pill`, `button_primary`, `button_secondary`, `button_toggle`, `button_icon`, `section_header`, and `info_row`.
  - **Top Header Bar (`src/window.rs`)**: Sleek top toolbar featuring app title, active filename chip, dimension pills, format badge, file size badge, zoom percentage, zoom controls, sidebar toggle (`N`), and settings shortcut (`S`).
  - **Collapsible `N`-Panel Sidebar (`src/window.rs`)**: Blender-inspired docked right drawer with vertical/horizontal category tabs (`Details`, `Tools`, `Plugins`), real-time image properties (dimensions, aspect ratio, megapixels, file size, path), quick action buttons, and active extension inspectors.
  - **Zen Mode (`Tab`)**: Instant toggle hiding all UI toolbars for an uninterrupted, distraction-free borderless image viewing canvas.
  - **Extension Sidebar API (`crates/opsis_extension_api`)**: Added `SidebarTabProvider` trait and `register_sidebar_tab_provider` enabling third-party extensions to inject custom tabs into the `N`-Panel.
- **Configurable Console Logging Levels & CLI Flags (`src/log.rs`, `src/main.rs`)**:
  - Six-tier logging hierarchy: `Off` (0), `Error` (1), `Warn` (2), `Info` (3), `Debug` (4), `Trace` (5).
  - CLI flags for fine-grained verbosity: `--log-level <level>` (or `--log-level=<level>`), `-v` / `--verbose` (debug), `-vv` / `--trace` (trace), `-q` / `--quiet` (error), and `--silent` (off).
  - Environment variable fallback (`OPSIS_LOG` / `RUST_LOG`).
  - Built-in `--help` / `-h` screen and `--version` / `-V` metadata flags.

### Fixed

- **Navigation Across Error & Corrupted States (`src/canvas.rs`, `src/window.rs`)**: Retained `last_file_path` and added `active_path()` so folder cycling (`NextImage` / `PrevImage`) continues functioning seamlessly even while displaying an error screen for corrupted files.
- **Corrupted Image Error State Display (`src/file_io.rs`, `src/canvas.rs`, `src/window.rs`)**: Fixed an issue where corrupted image files with unreadable dimensions or invalid payloads fell back to dummy dimensions and rendered a blank viewport; now properly sets `self.image = None`, returns descriptive error results, and renders the dedicated error card.
- **Redundant Per-Frame RGBA Decoding (`src/file_io.rs`, `src/canvas.rs`)**: Wrapped RGBA pixel buffer decoding in `Arc<OnceLock<Option<Bytes>>>` within `LoadedImage`, eliminating repeated 230ms full raster decodings on every render frame and reducing subsequent frame retrieval to 0.00ms.
- **State Borrow Conflict in Key Rebinding (`src/settings.rs`, `src/window.rs`)**: Scoped state `read()` borrows before mutating state via `set()` / `with_mut()`, preventing immutable-while-mutable borrow panics during in-app shortcut assignment.

### Changed

- **Minimalist Default Viewport (`src/window.rs`, `src/ui/components.rs`)**: Removed the default top header toolbar from the primary window canvas for an uncluttered image viewing experience, while preserving the reusable `toolbar_container` component in `src/ui/components.rs` for extensions and overlays to utilize.
- **Binary Output Name (`Cargo.toml`, `build.rs`)**: Restored the compiled executable and Windows PE metadata name to `opsis.exe` (`opsis`), removing version suffixing from the binary output file name.







