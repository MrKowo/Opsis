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
- **Comprehensive Extension Developer Guide (`docs/EXTENSIONS.md`, `crates/opsis_extension_api/README.md`)**:
  - Full developer documentation covering microkernel architecture, `OpsisExtension` lifecycle, and C-ABI dynamic constructor export.
  - In-depth guides and code examples for all capability traits: `SidebarTabProvider` (`N`-Panel), `ActionDefinition`/`ActionHandler` (rebindable shortcuts), `ImageFilterProvider` (pixel pipeline), `OverlayProvider` (HUDs), `ViewportProvider`, and `InputInterceptor`.
  - Detailed instructions for spawning native floating OS windows via `ctx.launch_window`.
  - Universal `.opx` ZIP bundle specification with multi-architecture binary directory structure and platform key matrix.
  - Complete, runnable end-to-end sample extension template.

### Fixed

- **Navigation Across Error & Corrupted States (`src/canvas.rs`, `src/window.rs`)**: Retained `last_file_path` and added `active_path()` so folder cycling (`NextImage` / `PrevImage`) continues functioning seamlessly even while displaying an error screen for corrupted files.
- **Corrupted Image Error State Display (`src/file_io.rs`, `src/canvas.rs`, `src/window.rs`)**: Fixed an issue where corrupted image files with unreadable dimensions or invalid payloads fell back to dummy dimensions and rendered a blank viewport; now properly sets `self.image = None`, returns descriptive error results, and renders the dedicated error card.
- **Redundant Per-Frame RGBA Decoding (`src/file_io.rs`, `src/canvas.rs`)**: Wrapped RGBA pixel buffer decoding in `Arc<OnceLock<Option<Bytes>>>` within `LoadedImage`, eliminating repeated 230ms full raster decodings on every render frame and reducing subsequent frame retrieval to 0.00ms.
- **State Borrow Conflict in Key Rebinding (`src/settings.rs`, `src/window.rs`)**: Scoped state `read()` borrows before mutating state via `set()` / `with_mut()`, preventing immutable-while-mutable borrow panics during in-app shortcut assignment.

### Changed

- **Minimalist Default Viewport (`src/window.rs`, `src/ui/components.rs`)**: Removed the default top header toolbar from the primary window canvas for an uncluttered image viewing experience, while preserving the reusable `toolbar_container` component in `src/ui/components.rs` for extensions and overlays to utilize.
- **Binary Output Name (`Cargo.toml`, `build.rs`)**: Restored the compiled executable and Windows PE metadata name to `opsis.exe` (`opsis`), removing version suffixing from the binary output file name.
- **Settings Window UI Component Migration (`src/settings.rs`)**: Refactored the native floating Settings window to completely utilize the Opsis UI Design System (`src/ui/components.rs` and `src/ui/theme.rs`), including `button_toggle` tabs, `key_badge` shortcut triggers, `status_pill` badges, `button_secondary` / `button_icon` controls, and `section_header` / `info_row` elements.
- **Interactive Switch Toggle & Row Component (`src/ui/components.rs`, `src/settings.rs`)**: Added `switch_toggle` and `switch_row` pill slider switches with smooth active/inactive thumb alignment and accent highlighting, replacing static rows with interactive switches in Settings.
- **Dropdown Menu & Select Components (`src/ui/components.rs`)**: Added `dropdown_menu`, `dropdown_select`, `dropdown_item`, and `dropdown_row` components providing customizable, elevated popover option lists with selection checkmarks and theme tokens.
- **UI Scale Dropdown in Settings (`src/settings.rs`)**: Integrated `dropdown_row` into Settings &rarr; General for selecting interface scaling multipliers from 1.00x to 4.00x in 0.25x steps.
- **Floating Dropdown Popover & Scrollable Capping (`src/ui/components.rs`)**: Updated `dropdown_menu` to use absolute layering (`Position::Absolute`, `layer(20)`, elevated shadow) so open menus float on top without displacing underlying layout widgets, centered option text alignment, and automatic `ScrollView` capping beyond 5 visible items.
- **Windows 11 Fluent UI Dropdown Alignment (`src/ui/components.rs`)**: Redesigned `dropdown_row` into a horizontal row with left-aligned setting title and right-aligned 180px ComboBox, with Fluent left accent indicator pills, right checkmarks, and elevated floating popovers matching Windows 11 Settings.
- **Dropdown List Element Box Containment (`src/ui/components.rs`)**: Fixed height calculations (`DROPDOWN_ITEM_ROW_HEIGHT = 28.0px`, exact item spacing, and 34px vertical offset) preventing list items from shifting down and overflowing outside the dropdown card container.
- **Dropdown Popover South Shift Fix (`src/ui/components.rs`)**: Adjusted `offset_y` from `34.0px` to `2.0px` on the floating popup element, eliminating double-counting of the trigger header's flex layout height that caused a one-slot downward displacement.
- **Dropdown Click-Away Dismissal & Outline Removal (`src/ui/components.rs`, `src/settings.rs`)**: Added an invisible layered backdrop catching outside clicks to close open dropdowns without changing selections, handled Escape dismissal, and removed bright focus outlines for clean, neutral borders.
- **Dropdown Hover Highlighting & Root Click Dismissal (`src/ui/components.rs`, `src/settings.rs`)**: Added interactive cursor hover highlighting across dropdown items via `on_pointer_enter`/`on_pointer_leave`, stopped click event bubbling on dropdown triggers/items, and connected window/pane root click catchers for reliable click-off dismissal.
- **Fluent Selected Option Over-Trigger Alignment (`src/ui/components.rs`)**: Positioned the floating dropdown popover vertically so that the currently selected option appears directly under the cursor and perfectly overlaps the trigger button box, matching Windows 11 ComboBox behavior with centered scrolling for long lists.
- **Dropdown Popover Background Alignment & Offset Fix (`src/ui/components.rs`)**: Fixed the upward offset mismatch by anchoring the popover card cleanly at `offset_y(2.0)` directly underneath the trigger button, unifying the background container and preventing upward displacement.
- **Controlled Scroll Position Auto-Centering (`src/ui/components.rs`, `src/settings.rs`)**: Integrated `ScrollController` into `ScrollView` within `dropdown_menu` and `dropdown_select`, dynamically centering the scroll view on the currently selected option upon opening long option lists.
- **Unified Popover Vertical Shift & Scroll Slot Alignment (`src/ui/components.rs`)**: Coordinated the floating popover card's vertical `offset_y` with the `ScrollView` scroll position so that the selected option aligns directly under the cursor and overlays the trigger button across both short and long option lists.
- **Torin Absolute Position Top-Offset Fix (`src/ui/components.rs`)**: Switched dropdown popover positioning to `Position::new_absolute().top(top_offset).left(0.0)`, ensuring Torin properly shifts the entire background card, borders, and shadow together with the option items.
- **Modular Dropdown Layout Metrics (`src/ui/components.rs`, `src/settings.rs`)**: Extracted `DropdownLayoutMetrics` helper struct to encapsulate and unify popup vertical positioning, visual slot mapping, and scroll offset calculations with dedicated unit tests.
- **Dedicated UI Helpers Module (`src/ui/helpers.rs`)**: Created `src/ui/helpers.rs` as the central home for UI helper structs, layout calculators, metric types (`DropdownLayoutMetrics`, `DropdownSelectProps`, `DropdownRowProps`), and constants (`MAX_DROPDOWN_VISIBLE_ITEMS`, `DROPDOWN_ITEM_ROW_HEIGHT`).
- **Global Non-Highlighting Scrollbar Definition (`src/ui/theme.rs`, `src/ui/mod.rs`, `src/window.rs`, `src/settings.rs`)**: Configured a unified global scrollbar theme via `Theme::create_freya_theme` and `use_init_opsis_theme()`, removing distracting track hover flashing with transparent backgrounds, subtle 8px thumb sizing, and non-highlighting idle/hover thumb colors across all application windows and scroll views.
- **Static Scrollbar Thumb & Background Elimination (`src/ui/components.rs`, `src/ui/helpers.rs`)**: Suppressed Freya's built-in hover expansion and black background rectangle by rendering a custom 4px pill overlay (`ScrollbarMetrics`) that remains constant and completely unchanged when hovered.
- **Interactive Draggable Dropdown Scrollbar (`src/ui/components.rs`)**: Added `dropdown_scrollable_list` supporting real-time pointer click-to-jump, global drag scrolling, tactile active thumb styling, and mouse wheel synchronization while maintaining the clean, non-expanding visual profile.
- **Hook-Safe Stateless Dropdown List (`src/ui/components.rs`)**: Resolved crash on opening dropdowns by eliminating conditional hook calls (`use_state`) within the conditional popup branch, driving position and drag synchronization directly through parent-owned `ScrollController`.
- **Dropdown Scrollbar Gutter Padding & Layer Priority (`src/ui/components.rs`)**: Added 8px right padding to list items in scrollable dropdowns to prevent row hover backgrounds from rendering over the scrollbar, and elevated the transparent interactive track to `layer(999)` with global capture dragging for immediate responsiveness.
- **Continuous Global Drag Session Tracking (`src/ui/components.rs`)**: Implemented absolute window coordinate delta drag tracking with drag session anchors, enabling smooth, continuous, and responsive thumb dragging across the full window area.
- **Vanilla ScrollView Restoration & Simplification (`src/ui/components.rs`, `src/ui/helpers.rs`)**: Removed custom manual scrollbar overlay and helper metrics in favor of clean, native Freya `ScrollView::new_controlled(ctrl)` driven by the global scrollbar theme, restoring full native dragging, wheel scrolling, and clipping with zero complexity.
- **Constant-Thickness Brighter-on-Hover Draggable Scrollbar (`src/ui/components.rs`, `src/ui/theme.rs`, `src/ui/helpers.rs`, `src/settings.rs`)**: Configured the scrollbar to maintain a static 4px thickness without expanding on hover or drag, dynamically brightening to `argb(190, 210, 225, 245)` on hover and `argb(245, 230, 240, 255)` on active drag across a transparent track with full continuous window-level delta drag tracking.
- **Robust Out-of-Bounds Drag & Release Handling (`src/ui/components.rs`, `src/settings.rs`)**: Added `is_primary` pointer button release detection in global drag event callbacks and window-level `on_press` handlers, preventing ghost drags when the cursor leaves the scroll view or window bounds.
- **Scoped Signal Peek on Drag Initiation (`src/ui/components.rs`, `src/settings.rs`)**: Resolved crash on initiating drag by replacing active `Signal::read()` borrows with scoped `Signal::peek()`, eliminating `BorrowMutError` when modifying drag state during pointer move and down events.
- **Continuous Mouse Hold Pointer Tracking (`src/ui/components.rs`)**: Removed button check from continuous move events so that mouse hold dragging responds smoothly and uninterruptedly throughout mouse movement.
- **Preserve Open Dropdown on Drag Release (`src/settings.rs`)**: Updated window background `on_press` dismissal handlers to check `was_dragging`, ensuring releasing a scrollbar drag outside the popup area clears the drag session without prematurely closing the dropdown.
- **Global Capture Pointer Press & Zero-Scroll Synchronization (`src/ui/components.rs`)**: Attached `on_capture_global_pointer_press` to catch mouse release events anywhere on the screen, and fixed scroll position reading to prevent snapping back when reaching top scroll offset (`y = 0`).
- **Versatile Button Component & Default General Tab (`src/ui/components.rs`, `src/ui/helpers.rs`, `src/settings.rs`)**: Added standard `button` component with variant support (`Primary`, `Secondary`, `Danger`) and interactive `button_row` widget in the General settings tab (`Restore Initial Settings`), and configured the Settings window to open the General tab by default.
- **Placeholder Example UI in General Settings Tab (`src/settings.rs`)**: Replaced the contents of the General tab with an "Example Section" containing interactive "Toggle example", "Dropdown example", and "Button example" widgets.
- **Reusable Section & Section Header Component (`src/ui/components.rs`)**: Redesigned `section` / `section_header` to render clean, semantic text on top of a subtle 1px divider line spanning the full width of the container.
- **Cross-Platform Hardware-Accelerated Acrylic Backdrop Blur (`src/ui/acrylic.rs`, `src/ui/theme.rs`, `src/settings.rs`)**: Implemented a universal Skia-powered backdrop blur rendering system (`SaveLayerRec` with Gaussian `image_filters::blur` and semi-transparent alpha tinting) across Windows, Linux, and macOS, with reusable `acrylic_surface` and `acrylic_panel` components and integration into the Settings categories sidebar drawer.
- **Settings Window Transparency & Frosted Sidebar Surface (`src/settings.rs`, `src/ui/acrylic.rs`)**: Enabled window-level alpha transparency (`with_transparency(true)`) and transparent window root in the Settings window, simplified `acrylic_surface` to directly host child elements on the canvas, and applied semi-transparent active tab accents so the acrylic sidebar shows through seamlessly.
- **Native OS Acrylic Blur Hook & Standalone Acrylic Test Window (`src/ui/acrylic.rs`, `src/settings.rs`)**: Added `apply_windows_acrylic` hook supporting Windows 11 `DWMWA_SYSTEMBACKDROP_TYPE` (Acrylic) and Windows 10 `SetWindowCompositionAttribute` blur-behind, and spawned a dedicated floating `Acrylic Test Window` upon opening settings with interactive blur sigma and tint controls.
- **Unified Settings Window Acrylic Glass Integration (`src/settings.rs`)**: Integrated frosted acrylic backdrop blur directly into the main Settings window with translucent sidebar drawer and calibrated translucent content pane, and cleaned up the temporary secondary test window.
- **Modular UI Component Library & Settings Window Rebuild (`src/ui/components.rs`, `src/settings.rs`)**: Implemented standalone `divider` / `vertical_divider`, elevated `card` / `card_row`, `pane_header`, `expandable_card` accordion, `file_dropzone`, `empty_state`, generalized `text_field` string input, and customizable `table` / `table_header` / `table_row` components, and rebuilt all 5 Settings panes (General, Appearance, Extensions, Shortcuts, About) with real-time search filtering and modular UI assembly.
- **Settings Tab Hook Hoisting & Crash Prevention (`src/settings.rs`)**: Hoisted all reactive state hooks (`expanded_items`, `status_banner`, `search_query`) to the root of `settings_window_view` to guarantee deterministic hook order and eliminate tab-switching crashes.
- **Hook-Free Lightweight `text_field` Component (`src/ui/components.rs`, `src/settings.rs`)**: Replaced heavy nested component invocation in `text_field` with a streamlined, hook-free layout primitive that renders search and string fields deterministically without modifying parent component hook counts.
- **Cursor-Centered Zoom & Universal Global Drag Release (`src/canvas.rs`, `src/window.rs`)**: Implemented cursor-anchored zoom calculations (`zoom_at`) adjusting pan offsets relative to mouse cursor coordinates on wheel scroll, and attached global pointer release listeners (`on_capture_global_pointer_press` and `on_global_pointer_press`) to reliably end canvas panning even when the mouse is released outside the window frame.
- **Natural Mouse Wheel Scroll Direction (`src/canvas.rs`)**: Corrected mouse wheel zoom direction so scrolling up/forward zooms in and scrolling down/backward zooms out.
- **Opaque Solid Surface for Settings Window (`src/settings.rs`)**: Removed the acrylic backdrop blur and window transparency from the Settings window, transitioning the root window, sidebar drawer, and content panes to solid theme surface colors (`Theme::surface_base()` and `Theme::surface_panel()`).
- **Aspect-Ratio & 75% Screen Scaling Window Sizing (`src/canvas.rs`, `src/window.rs`)**: Implemented `calculate_target_window_size` and `resize_window_to_image_aspect`, dynamically scaling the main OS window to match the opened image's aspect ratio bounded within 75% of the active monitor's resolution (min 400x300 px) on startup CLI args, file picker (`O`), and drag-and-drop, while maintaining fixed window dimensions during rapid folder cycling.
- **Full Viewport Auto-Fit on Startup & Dynamic Resize (`src/canvas.rs`, `src/window.rs`)**: Updated `calculate_initial_zoom` and `fit_to_window` to compute exact viewport boundary fit ratios without arbitrary margins, and hooked window `on_sized` layout events to continuously auto-fit unpanned images upon OS window creation and resizing.
- **Dedicated `AppSettings` Configuration System (`src/config.rs`, `src/manager.rs`)**: Introduced a standalone `AppSettings` struct with automatic `settings.json` persistence in the config directory, cleanly separating general application visual preferences from extension plugin management.
- **Canvas Acrylic Background Option & Settings Appearance Toggle (`src/settings.rs`, `src/canvas.rs`, `src/window.rs`)**: Added an **"Enable Acrylic Background"** switch in **Settings &rarr; Appearance** under Visual Preferences. When enabled, the canvas viewport composites a hardware-accelerated Skia Gaussian blur backdrop with theme acrylic tinting behind images and watermark empty states with 100% cross-OS compatibility and real-time live synchronization between the Settings window and the main canvas.
- **Ambient Blurred Image Backdrop Engine (`src/ui/acrylic.rs`, `src/canvas.rs`)**: Implemented `render_ambient_blurred_backdrop`, rendering a scaled, Gaussian-blurred (`blur_sigma: 48.0`) frosted layer of the active image behind the sharp foreground canvas viewport.
- **Native OS Desktop Backdrop Acrylic Blur Integration (`src/window.rs`, `src/canvas.rs`)**: Configured the main application window with transparency support (`with_transparency(true)`) and dynamic native Win32 DWM acrylic backdrop blur (`apply_windows_acrylic`) with semi-transparent alpha canvas tinting, allowing the actual operating system desktop and background windows behind Opsis to be blurred in real-time when acrylic is enabled.
- **Centralized `Theme::canvas_background` Single Source of Truth (`src/ui/theme.rs`, `src/canvas.rs`, `src/window.rs`, `src/ui/acrylic.rs`)**: Unified all canvas, root window, and OS compositor tinting under `Theme::canvas_background(acrylic_enabled)`, `Theme::ACRYLIC_ALPHA`, and `Theme::ACRYLIC_TINT_RGB`, ensuring consistent styling and a single source of truth across the entire codebase.
- **Synchronized OS Title Bar Caption Tint (`src/ui/acrylic.rs`, `src/window.rs`)**: Extended `apply_windows_acrylic` with `DWMWA_CAPTION_COLOR` and `DWMWA_TEXT_COLOR` attributes, dynamically tinting the native OS window title bar and caption buttons to match `Theme::ACRYLIC_TINT_RGB` when acrylic is enabled and `Theme::surface_base()` when disabled.
- **Seamless Title Bar Acrylic Frame Extension (`src/ui/acrylic.rs`)**: Implemented `DwmExtendFrameIntoClientArea` with margins `(-1, -1, -1, -1)` and `DWMWA_COLOR_DEFAULT` (`0xFFFFFFFE`), extending the translucent acrylic backdrop material across the entire window frame and title bar so the top bar matches the background tint instead of turning opaque black.
- **Comprehensive Architecture Reference Manual for AI Coding Agents (`docs/ARCHITECTURE.md`, `AGENTS.md`)**: Created an exhaustive, structured architecture reference document with subsystem lookup tables, end-to-end Mermaid dataflows, module deep dives, Freya/Dioxus hook invariants, transformation math, and developer rules to eliminate superfluous file exploration for AI agents and human contributors.
















