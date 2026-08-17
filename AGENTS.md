# AGENTS.md

Guidance and instructions for AI coding agents working on the **Opsis** codebase.

---

## Overview & Architecture

**Opsis** is an ultra-lightweight, portable, high-performance image viewer built with Rust, **Freya** (powered by Skia 2D graphics and Winit windowing), and a pure **Microkernel Extension-First Architecture**.

- **Minimal Microkernel**: The host binary contains zero hardcoded features, viewports, or widgets. The host purely initializes the [`ExtensionManager`](src/manager.rs) and delegates all visualization, UI overlays, and feature handling to modular extensions.
- **Dedicated Window Host**: Window creation, lifecycle management, and Base Canvas rendering are isolated within [`src/window.rs`](src/window.rs).
- **Universal Extension Bundles (`.opx`)**: Extensions can be packaged as cross-platform `.opx` archives (ZIP archives containing `manifest.json` and multi-architecture native dynamic libraries) that run seamlessly across Windows, Linux, and macOS.
- **Comprehensive Architecture Guide**: For exhaustive dataflow diagrams, transformation math, subsystem deep dives, and instant lookup tables, see [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

### Codebase Map

| File / Directory                                             | Purpose                                                                                                                                                             |
|:------------------------------------------------------------ |:------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [`Cargo.toml`](Cargo.toml)                                   | Root workspace definition and host dependency configuration.                                                                                                        |
| [`crates/opsis_extension_api`](crates/opsis_extension_api)   | Public API crate defining extension traits (`OpsisExtension`, `SidebarTabProvider`, `ActionHandler`, `ImageFilterProvider`, `OverlayProvider`, `ViewportProvider`, `InputInterceptor`), context structs, and the FFI constructor. |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)               | Exhaustive codebase architecture manual, subsystem lookup matrix, Mermaid dataflow diagrams, and technical gotchas for AI agents.                                  |
| [`docs/EXTENSIONS.md`](docs/EXTENSIONS.md)                   | Official developer guide for building, packaging, and publishing Opsis extensions.                                                                                  |
| [`src/main.rs`](src/main.rs)                                 | Pure microkernel entry point: initializes CLI args, log levels, extension manager, and runs window host.                                                            |
| [`src/window.rs`](src/window.rs)                             | Dedicated Window Host: Freya window creation, lifecycle, Base Canvas watermark, `N`-Panel sidebar drawer, and input dispatching.                                     |
| [`src/canvas.rs`](src/canvas.rs)                             | Core 2D Canvas: Skia rendering, cursor-centered scroll zoom engine, mouse drag pan, format decoding, error cards, and file drag-and-drop.                           |
| [`src/file_io.rs`](src/file_io.rs)                           | Sub-millisecond dimension header parsing, on-demand `OnceLock` RGBA buffer decoding, native file dialogs, and folder cycling.                                     |
| [`src/hotkeys.rs`](src/hotkeys.rs)                           | Centralized command registry, hotkey dispatching, interactive rebinding, and `keybindings.json` lazy persistence.                                                  |
| [`src/config.rs`](src/config.rs)                             | Application settings (`AppSettings`: `dark_mode`, `show_watermark`, `acrylic_background`) and `settings.json` disk persistence.                                    |
| [`src/ui/`](src/ui/)                                         | Opsis UI Design System: semantic theme tokens ([`src/ui/theme.rs`](src/ui/theme.rs)), widget primitives ([`src/ui/components.rs`](src/ui/components.rs)), Skia/Win32 acrylic blur ([`src/ui/acrylic.rs`](src/ui/acrylic.rs)), and dropdown helpers ([`src/ui/helpers.rs`](src/ui/helpers.rs)). |
| [`src/log.rs`](src/log.rs)                                   | Configurable console logging hierarchy (Off, Error, Warn, Info, Debug, Trace), uptime timestamping, and CLI flag parser.                                           |
| [`src/settings.rs`](src/settings.rs)                         | Built-in native Settings window with vertical tabs (General, Appearance, Extensions, Shortcuts, About), hoisted reactive hook state, and drag-and-drop installer.   |
| [`src/manager.rs`](src/manager.rs)                           | Discovers, loads, and manages active extensions and orchestrates hook dispatching asynchronously in background threads.                                             |
| [`src/bundle.rs`](src/bundle.rs)                             | `.opx` ZIP bundle extraction, platform auto-detection, and cache management.                                                                                        |
| [`src/loader.rs`](src/loader.rs)                             | Zero-overhead native dynamic library loading via `libloading`.                                                                                                      |
| [`build.rs`](build.rs)                                       | Build script embedding Windows icon/PE metadata and automatically copying `extensions/` into target directories.                                                    |
| [`assets/logo.png`](assets/logo.png)                         | Embedded watermark asset rendered on the Base Window Canvas.                                                                                                        |

---

## Extension Architecture & Development Guide

Extensions are native dynamic libraries (`.dll`, `.so`, `.dylib`) or universal `.opx` packages that interact with the host through the public [`opsis_extension_api`](crates/opsis_extension_api) crate.

### 1. The Core Lifecycle: `OpsisExtension`

Every extension must implement the [`OpsisExtension`](crates/opsis_extension_api/src/lib.rs) trait:

```rust
use opsis_extension_api::{
    ExtensionManifest, ExtensionRegistry, OpsisExtension, CURRENT_API_VERSION,
};

pub struct MyExtension;

impl OpsisExtension for MyExtension {
    fn manifest(&self) -> ExtensionManifest {
        ExtensionManifest {
            id: "author.my-extension".to_string(),
            name: "My Extension".to_string(),
            version: "0.1.0".to_string(),
            author: "Author Name".to_string(),
            description: "Detailed description of the extension.".to_string(),
            api_version: CURRENT_API_VERSION,
        }
    }

    fn on_init(&mut self, registry: &mut ExtensionRegistry) -> Result<(), String> {
        // Register capability providers here
        Ok(())
    }

    fn on_unload(&mut self) {
        // Cleanup resources before unloading
    }
}
```

### 2. Capability Traits

During `on_init`, an extension registers one or more capabilities with the [`ExtensionRegistry`](crates/opsis_extension_api/src/lib.rs):

- **`SidebarTabProvider`**: Injects custom category tabs into the collapsible `N`-Panel sidebar drawer.
  ```rust
  impl SidebarTabProvider for MyTab {
      fn tab_title(&self) -> String { "My Tab".to_string() }
      fn render_tab(&self, ctx: &OverlayContext) -> Option<Element> { /* Freya Element */ }
  }
  ```

- **`ActionHandler` & `ActionDefinition`**: Registers named commands that automatically appear in **Settings &rarr; Shortcuts** for user key rebinding.
  ```rust
  impl ActionHandler for MyHandler {
      fn execute(&mut self, action_id: &str, ctx: &InputContext) -> EventAction {
          if action_id == "my.action" { return EventAction::Handled; }
          EventAction::Pass
      }
  }
  ```

- **`ImageFilterProvider`**: Applies real-time, non-destructive post-processing pixel filters to decoded RGBA buffers.
  ```rust
  impl ImageFilterProvider for MyFilter {
      fn apply_filter(&self, rgba: &[u8], width: u32, height: u32) -> Option<bytes::Bytes> {
          // Return Some(transformed_bytes) or None if inactive
      }
  }
  ```

- **`OverlayProvider`**: Injects HUD elements, floating toolbars, status bars, and UI widgets layered on top of the viewport.
  ```rust
  impl OverlayProvider for MyOverlay {
      fn render_overlay(&self, ctx: &OverlayContext) -> Option<Element> {
          // Return optional Freya Element overlay
      }
  }
  ```

- **`ViewportProvider`**: Provides primary canvas rendering (e.g. 2D image renderer, 3D viewport, node-based compositing canvas).
  ```rust
  impl ViewportProvider for MyViewport {
      fn render_viewport(&self, ctx: &ViewportContext) -> Option<Element> {
          // Return optional Freya Element tree
      }
  }
  ```

- **`InputInterceptor`**: Intercepts keyboard, pointer, and scroll events before host handling. Returning `EventAction::Handled` stops further propagation.
  ```rust
  impl InputInterceptor for MyInputHandler {
      fn on_input(&mut self, event: &InputEvent, ctx: &InputContext) -> EventAction {
          EventAction::Pass
      }
  }
  ```

### 3. Native Floating Windows

Extensions can spawn standalone, detached native OS windows at runtime via `ctx.launch_window`:

```rust
if let Some(ref launcher) = ctx.launch_window {
    launcher(
        "My Custom Tool".to_string(),
        (480.0, 560.0),
        Arc::new(|trigger_redraw| {
            // Return Freya Element tree for the new window
            my_window_view(trigger_redraw)
        }),
    );
}
```

### 4. Exporting the Constructor

The library must expose the C-ABI dynamic constructor symbol:

```rust
/// Exported dynamic constructor for the extension.
///
/// # Safety
/// Must be invoked by the host runtime loader with a compatible `opsis_extension_api` version.
#[no_mangle]
pub unsafe extern "Rust" fn opsis_extension_create() -> Box<dyn OpsisExtension> {
    Box::new(MyExtension)
}
```

### 5. Packaging & Deployment

- **Raw Dynamic Library**: Build with `crate-type = ["cdylib"]` and place the compiled `.dll` / `.so` / `.dylib` into `<exe_dir>/extensions/` or user profile directory.
- **Universal `.opx` Bundle**: Package into a ZIP archive with a root `manifest.json` and platform-specific binaries inside `bin/<platform-key>/` (e.g. `bin/windows-x86_64/plugin.dll`, `bin/linux-x86_64/plugin.so`, `bin/macos-aarch64/plugin.dylib`).

---

## Documentation Synchronization Policy

Whenever working on or modifying the extension API, agents must strictly follow this policy:

- **MANDATORY DOCS UPDATE**: Whenever any trait, struct, method, enum variant, or capability in [`crates/opsis_extension_api`](crates/opsis_extension_api) is added, modified, renamed, or removed, agents **MUST ALWAYS update the developer documentation**:
  - [`docs/EXTENSIONS.md`](docs/EXTENSIONS.md): Update API references, trait guides, code snippets, and runnable examples.
  - [`crates/opsis_extension_api/README.md`](crates/opsis_extension_api/README.md): Update quickstart snippets and trait listings.
- **ACCURACY & INTEGRITY**: All code examples in [`docs/EXTENSIONS.md`](docs/EXTENSIONS.md) must be valid, idiomatic Rust code compatible with the current version of [`opsis_extension_api`](crates/opsis_extension_api) and `freya`.

---

## Setup & Workflow Commands

- **Check Workspace**: `cargo check --workspace`
- **Run Tests**: `cargo test --workspace`
- **Clippy Linter**: `cargo clippy --workspace`
- **Run Application**: `cargo run`
- **Release Build**: `cargo build --release`

Always verify changes with `cargo check --workspace` and `cargo test --workspace` before marking a task complete.

---

## Changelog Policy

- **ALWAYS APPEND ONLY**: When modifying or updating [`CHANGELOG.md`](CHANGELOG.md), agents must **ONLY APPEND** new release notes or entries to the file.
- **NEVER OVERWRITE OR DELETE**: Existing changelog history and past version entries in [`CHANGELOG.md`](CHANGELOG.md) must remain intact and must never be deleted, truncated, or overwritten (unless explicitly granted an exception by the user).
- **CONCISE & STRUCTURED**: Keep changelog entries crisp, concise, and structured under standard `Keep a Changelog` headers (`Added`, `Changed`, `Fixed`, `Removed`).
