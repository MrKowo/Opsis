# AGENTS.md

Guidance and instructions for AI coding agents working on the **Opsis** codebase.

---

## Overview & Architecture

**Opsis** is an ultra-lightweight, portable, high-performance image viewer built with Rust, **Freya** (powered by Skia 2D graphics and Winit windowing), and a pure **Microkernel Extension-First Architecture**.

- **Minimal Microkernel**: The host binary contains zero hardcoded features, viewports, or widgets. The host purely initializes the [`ExtensionManager`](src/manager.rs) and delegates all visualization, UI overlays, and feature handling to modular extensions.
- **Dedicated Window Host**: Window creation, lifecycle management, and Base Canvas rendering are isolated within [`src/window.rs`](src/window.rs).
- **Universal Extension Bundles (`.opx`)**: Extensions can be packaged as cross-platform `.opx` archives (ZIP archives containing `manifest.json` and multi-architecture native dynamic libraries) that run seamlessly across Windows, Linux, and macOS.

### Codebase Map

| File / Directory                                             | Purpose                                                                                                                                                             |
|:------------------------------------------------------------ |:------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [`Cargo.toml`](Cargo.toml)                                   | Root workspace definition and host dependency configuration.                                                                                                        |
| [`crates/opsis_extension_api`](crates/opsis_extension_api)   | Public API crate defining extension traits (`OpsisExtension`, `ViewportProvider`, `OverlayProvider`, `InputInterceptor`), context structs, and the FFI constructor. |
| [`src/main.rs`](src/main.rs)                                 | Pure microkernel entry point: initializes extension manager and runs window host.                                                                                   |
| [`src/window.rs`](src/window.rs)                             | Dedicated Window Host: Freya window creation, lifecycle, Base Canvas watermark, and input dispatching.                                                              |
| [`src/settings.rs`](src/settings.rs)                         | Built-in native Settings window with vertical tabs (General, Appearance, Extensions, Shortcuts, About).                                                             |
| [`src/manager.rs`](src/manager.rs)                           | Discovers, loads, and manages active extensions and orchestrates hook dispatching.                                                                                  |
| [`src/bundle.rs`](src/bundle.rs)                             | `.opx` ZIP bundle extraction, platform auto-detection, and cache management.                                                                                        |
| [`src/loader.rs`](src/loader.rs)                             | Zero-overhead native dynamic library loading via `libloading`.                                                                                                      |
| [`assets/logo.png`](assets/logo.png)                         | Embedded watermark asset rendered on the Base Window Canvas.                                                                                                        |

---

## Extension Architecture & Development Guide

Extensions are native dynamic libraries (`.dll`, `.so`, `.dylib`) or universal `.opx` packages that interact with the host through the public [`opsis_extension_api`](crates/opsis_extension_api) crate.

### 1. The Core Lifecycle: `OpsisExtension`

Every extension must implement the [`OpsisExtension`](crates/opsis_extension_api/src/lib.rs) trait:

```rust
use opsis_extension_api::{ExtensionManifest, ExtensionRegistry, OpsisExtension};

pub struct MyExtension;

impl OpsisExtension for MyExtension {
    fn manifest(&self) -> ExtensionManifest {
        ExtensionManifest {
            id: "author.my-extension".to_string(),
            name: "My Extension".to_string(),
            version: "0.1.0".to_string(),
            author: "Author Name".to_string(),
            description: "Detailed description of the extension.".to_string(),
            api_version: 1,
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

- **`ViewportProvider`**: Provides the primary canvas rendering (e.g. 2D image renderer, 3D viewport, node-based compositing canvas).
  
  ```rust
  impl ViewportProvider for MyViewport {
      fn render_viewport(&self, ctx: &ViewportContext) -> Element {
          // Return Freya Element tree
      }
  }
  ```
- **`OverlayProvider`**: Injects HUD elements, toolbars, status bars, and floating UI widgets layered on top of the viewport.
  
  ```rust
  impl OverlayProvider for MyOverlay {
      fn render_overlay(&self, ctx: &OverlayContext) -> Option<Element> {
          // Return optional Freya Element overlay
      }
  }
  ```
- **`InputInterceptor`**: Intercepts keyboard, pointer, and scroll events. Returning `EventAction::Handled` stops further propagation.
  
  ```rust
  impl InputInterceptor for MyInputHandler {
      fn on_input(&mut self, event: &InputEvent, ctx: &InputContext) -> EventAction {
          if let InputEvent::KeyDown(ref key) = event {
              if key == "s" {
                  // Handle shortcut or spawn native window via ctx.launch_window
                  return EventAction::Handled;
              }
          }
          EventAction::Pass
      }
  }
  ```

### 3. Native Floating Windows

Extensions can spawn standalone, detached native OS windows at runtime by calling `ctx.launch_window`:

```rust
if let Some(ref launch) = ctx.launch_window {
    let builder: WindowBuilderFn = Arc::new(move || {
        // Return Freya Element tree for the new window
        my_window_view()
    });
    (launch)(
        "Window Title".to_string(),
        (480.0, 560.0), // width, height
        builder,
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

- **Raw Dynamic Library**: Build with `crate-type = ["cdylib"]` and place the compiled `.dll` / `.so` / `.dylib` into `<exe_dir>/extensions/`.
- **Universal `.opx` Bundle**: Package into a ZIP archive with a root `manifest.json` and platform-specific binaries inside `bin/<platform-key>/` (e.g. `bin/windows-x86_64/plugin.dll`, `bin/linux-x86_64/plugin.so`, `bin/macos-aarch64/plugin.dylib`).

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
