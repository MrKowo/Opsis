# Opsis Extension Developer Guide

Welcome to the **Opsis Extension Development Guide**!

**Opsis** is built on a **pure microkernel, extension-first architecture**. The host binary contains zero hardcoded features, viewports, or widgets—it solely initializes the runtime and delegates all visualization, image filtering, UI panels, toolbars, floating windows, and custom shortcuts to modular extensions.

Whether you want to build a custom color-grading tool, an EXIF metadata inspector, a 3D model visualizer, or custom productivity hotkeys, this guide will walk you through building, testing, packaging, and publishing your extension.

---

## Table of Contents

1. [Architecture Overview](#1-architecture-overview)
2. [Quickstart & Project Setup](#2-quickstart--project-setup)
3. [The Core Extension Lifecycle](#3-the-core-extension-lifecycle)
4. [Capability Traits & Extension Points](#4-capability-traits--extension-points)
   - [Sidebar Tabs (The `N`-Panel)](#41-sidebar-tabs-the-n-panel)
   - [Commands & Rebindable Hotkeys](#42-commands--rebindable-hotkeys)
   - [Image Filters & Pixel Transformations](#43-image-filters--pixel-transformations)
   - [UI Overlays & Floating HUDs](#44-ui-overlays--floating-huds)
   - [Primary Viewport Renderers](#45-primary-viewport-renderers)
   - [Raw Input Interceptors](#46-raw-input-interceptors)
5. [Native Floating Windows](#5-native-floating-windows)
6. [Universal `.opx` Bundling & Packaging](#6-universal-opx-bundling--packaging)
7. [Testing, Discovery & Debugging](#7-testing-discovery--debugging)
8. [Complete End-to-End Example](#8-complete-end-to-end-example)

---

## 1. Architecture Overview

Opsis extensions are compiled as native dynamic libraries (`.dll` on Windows, `.so` on Linux, `.dylib` on macOS) or packaged into cross-platform `.opx` archives.

```
┌─────────────────────────────────────────────────────────────┐
│                         Opsis Host                          │
├─────────────────────────────────────────────────────────────┤
│  • Microkernel Runtime          • Non-blocking Worker Pool │
│  • Freya / Skia Canvas Host     • Hotkey Engine             │
│  • Dual-Location Discovery      • .opx Extraction Cache     │
└──────────────┬───────────────────────────────┬──────────────┘
               │                               │
               ▼                               ▼
   ┌───────────────────────┐       ┌───────────────────────┐
   │  opsis_extension_api  │       │  opsis_extension_api  │
   ├───────────────────────┤       ├───────────────────────┤
   │  Extension: Channels  │       │  Extension: Custom    │
   │  • ImageFilterProvider│       │  • SidebarTabProvider │
   │  • SidebarTabProvider │       │  • ActionHandler      │
   │  • ActionHandler      │       │  • OverlayProvider    │
   └───────────────────────┘       └───────────────────────┘
```

### Key Architectural Principles:
- **Zero Runtime Overhead**: Extensions are loaded dynamically via `libloading` with direct memory and hardware-accelerated GPU access.
- **Asynchronous Non-Blocking Discovery**: Extensions load in parallel in a background worker thread upon application start, ensuring the host window and canvas open instantly.
- **Modular Capabilities**: Extensions only register what they need—from a single keyboard shortcut up to a full custom rendering engine.

---

## 2. Quickstart & Project Setup

### 2.1 Create a New Crate

Create a new Rust library project:

```bash
cargo new --lib opsis_my_extension
cd opsis_my_extension
```

### 2.2 Configure `Cargo.toml`

Configure your crate to output a C-compatible dynamic library (`cdylib`) and add dependencies:

```toml
[package]
name = "opsis_my_extension"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <you@example.com>"]
description = "A custom Opsis extension"

[lib]
crate-type = ["cdylib"]

[dependencies]
opsis_extension_api = { git = "https://github.com/MrKowo/Opsis.git" }
freya = { version = "0.4.1", default-features = false, features = ["engine"] }
bytes = "1.10"
serde = { version = "1.0", features = ["derive"] }
```

---

## 3. The Core Extension Lifecycle

Every extension implements the `OpsisExtension` trait and exports the dynamic constructor symbol `opsis_extension_create`.

```rust
use opsis_extension_api::{
    ExtensionManifest, ExtensionRegistry, OpsisExtension, CURRENT_API_VERSION,
};

pub struct MyExtension;

impl OpsisExtension for MyExtension {
    /// Returns metadata identifying this extension.
    fn manifest(&self) -> ExtensionManifest {
        ExtensionManifest {
            id: "author.my-extension".to_string(),
            name: "My Extension".to_string(),
            version: "0.1.0".to_string(),
            author: "Your Name".to_string(),
            description: "Detailed description of what this extension does.".to_string(),
            api_version: CURRENT_API_VERSION,
        }
    }

    /// Called on initialization. Register your capability providers here.
    fn on_init(&mut self, registry: &mut ExtensionRegistry) -> Result<(), String> {
        // Register capability providers with the host
        Ok(())
    }

    /// Optional cleanup hook invoked before unloading.
    fn on_unload(&mut self) {
        // Teardown background workers, temporary files, etc.
    }
}

/// Exported dynamic constructor for the extension loader.
///
/// # Safety
/// Must be invoked by the host runtime loader with a compatible `opsis_extension_api`.
#[no_mangle]
pub unsafe extern "Rust" fn opsis_extension_create() -> Box<dyn OpsisExtension> {
    Box::new(MyExtension)
}
```

---

## 4. Capability Traits & Extension Points

During `on_init`, an extension registers capability providers with the `ExtensionRegistry`.

### 4.1 Sidebar Tabs (The `N`-Panel)

Extensions can inject custom tab panels into the collapsible right sidebar (opened via <kbd>N</kbd>).

```rust
use freya::prelude::*;
use opsis_extension_api::{OverlayContext, SidebarTabProvider};

pub struct ToolTab;

impl SidebarTabProvider for ToolTab {
    fn tab_title(&self) -> String {
        "Tools".to_string()
    }

    fn tab_icon(&self) -> Option<String> {
        Some("wrench".to_string())
    }

    fn render_tab(&self, _ctx: &OverlayContext) -> Option<Element> {
        Some(
            rect()
                .width(Size::fill())
                .padding(Gaps::new_all(12.0))
                .spacing(8.0)
                .direction(Direction::vertical())
                .children([
                    label()
                        .text("Custom Extension Panel")
                        .font_size(13.0)
                        .color(Color::from_rgb(240, 240, 245))
                        .into(),
                    label()
                        .text("Perform operations or inspect active images.")
                        .font_size(11.0)
                        .color(Color::from_rgb(160, 160, 175))
                        .into(),
                ])
                .into(),
        )
    }
}
```

Register the tab in `on_init`:

```rust
registry.register_sidebar_tab_provider(Box::new(ToolTab));
```

---

### 4.2 Commands & Rebindable Hotkeys

Register named actions with default keybindings. Actions automatically appear in the user's **Settings &rarr; Shortcuts** window and support interactive in-app rebinding.

```rust
use opsis_extension_api::{
    ActionDefinition, ActionHandler, EventAction, ExtensionRegistry, InputContext,
};

pub struct InvertActionHandler;

impl ActionHandler for InvertActionHandler {
    fn execute(&mut self, action_id: &str, ctx: &InputContext) -> EventAction {
        if action_id == "my_ext.invert" {
            println!("Invert command executed on: {:?}", ctx.image_path);
            return EventAction::Handled;
        }
        EventAction::Pass
    }
}
```

Register the action in `on_init`:

```rust
let action_def = ActionDefinition {
    id: "my_ext.invert".to_string(),
    name: "Invert Image Colors".to_string(),
    category: "Filters".to_string(),
    default_keys: vec!["i".to_string()],
    description: "Inverts all RGB color channels in the active image.".to_string(),
};

registry.register_action(action_def, Box::new(InvertActionHandler));
```

---

### 4.3 Image Filters & Pixel Transformations

The image filter pipeline allows non-destructive, real-time transformations on decoded RGBA raster buffers (e.g. color grading, channel isolation, edge detection).

```rust
use bytes::Bytes;
use opsis_extension_api::ImageFilterProvider;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct GrayscaleFilter {
    pub enabled: Arc<AtomicBool>,
}

impl ImageFilterProvider for GrayscaleFilter {
    fn apply_filter(&self, rgba: &[u8], _width: u32, _height: u32) -> Option<Bytes> {
        if !self.enabled.load(Ordering::Relaxed) {
            return None; // Passthrough without copying or modifying
        }

        let mut output = rgba.to_vec();
        for chunk in output.chunks_exact_mut(4) {
            let r = chunk[0] as f32;
            let g = chunk[1] as f32;
            let b = chunk[2] as f32;
            // Standard luminance weights
            let gray = (0.2126 * r + 0.7152 * g + 0.0722 * b) as u8;
            chunk[0] = gray;
            chunk[1] = gray;
            chunk[2] = gray;
        }

        Some(Bytes::from(output))
    }
}
```

Register the filter in `on_init`:

```rust
registry.register_image_filter_provider(Box::new(GrayscaleFilter { enabled }));
```

---

### 4.4 UI Overlays & Floating HUDs

Render interactive HUD widgets, floating toolbars, status indicators, or histograms layered on top of the primary image viewport.

```rust
use freya::prelude::*;
use opsis_extension_api::{OverlayContext, OverlayProvider};

pub struct WatermarkOverlay;

impl OverlayProvider for WatermarkOverlay {
    fn render_overlay(&self, ctx: &OverlayContext) -> Option<Element> {
        let (w, h) = ctx.window_size;
        Some(
            rect()
                .position(Position::Absolute)
                .bottom(Size::px(16.0))
                .left(Size::px(16.0))
                .background(Color::from_argb(180, 20, 20, 24))
                .padding(Gaps::new_all(8.0))
                .corner_radius(6.0)
                .child(
                    label()
                        .text(format!("Window: {:.0}x{:.0} px", w, h))
                        .font_size(11.0)
                        .color(Color::from_rgb(200, 200, 210)),
                )
                .into(),
        )
    }
}
```

Register the overlay in `on_init`:

```rust
registry.register_overlay_provider(Box::new(WatermarkOverlay));
```

---

### 4.5 Primary Viewport Renderers

Replace or augment the base 2D image renderer (e.g. providing a 3D viewport, node compositor canvas, or PDF page viewer).

```rust
use freya::prelude::*;
use opsis_extension_api::{ViewportContext, ViewportProvider};

pub struct CustomViewport;

impl ViewportProvider for CustomViewport {
    fn render_viewport(&self, ctx: &ViewportContext) -> Option<Element> {
        if let Some(ref path) = ctx.image_path {
            if path.extension().and_then(|s| s.to_str()) == Some("custom") {
                return Some(
                    rect()
                        .width(Size::fill())
                        .height(Size::fill())
                        .main_align(Alignment::Center)
                        .cross_align(Alignment::Center)
                        .child(label().text("Custom Format Viewport"))
                        .into(),
                );
            }
        }
        None // Fall back to core Skia viewport
    }
}
```

---

### 4.6 Raw Input Interceptors

Intercept raw keyboard keys, pointer motion, clicks, and scroll wheel deltas before host dispatch.

```rust
use opsis_extension_api::{EventAction, InputContext, InputEvent, InputInterceptor};

pub struct KeyInterceptor;

impl InputInterceptor for KeyInterceptor {
    fn on_input(&mut self, event: &InputEvent, _ctx: &InputContext) -> EventAction {
        match event {
            InputEvent::KeyDown(key) if key == "g" => {
                println!("'G' pressed - triggering extension logic");
                EventAction::Handled // Stop further propagation
            }
            _ => EventAction::Pass,
        }
    }
}
```

---

## 5. Native Floating Windows

Extensions can spawn detached, standalone OS windows at runtime using the host's `launch_window` callback provided on `OverlayContext` / `InputContext`.

```rust
use freya::prelude::*;
use opsis_extension_api::OverlayContext;
use std::sync::Arc;

pub fn open_color_picker_window(ctx: &OverlayContext) {
    if let Some(ref launcher) = ctx.launch_window {
        launcher(
            "Color Inspector - Opsis".to_string(),
            (400.0, 480.0),
            Arc::new(|trigger_redraw| {
                rect()
                    .width(Size::fill())
                    .height(Size::fill())
                    .background(Color::from_rgb(24, 24, 28))
                    .padding(Gaps::new_all(16.0))
                    .direction(Direction::vertical())
                    .spacing(12.0)
                    .children([
                        label()
                            .text("Interactive Color Tool")
                            .font_size(15.0)
                            .font_weight(FontWeight::BOLD)
                            .color(Color::from_rgb(255, 255, 255))
                            .into(),
                    ])
                    .into()
            }),
        );
    }
}
```

---

## 6. Universal `.opx` Bundling & Packaging

Opsis supports universal **`.opx` bundles**—a standard ZIP archive format containing extension metadata and compiled multi-architecture native binaries.

### 6.1 Package Directory Structure

```
my_extension.opx (ZIP archive)
├── manifest.json
└── bin/
    ├── windows-x86_64/
    │   └── my_extension.dll
    ├── linux-x86_64/
    │   └── libmy_extension.so
    ├── macos-aarch64/
    │   └── libmy_extension.dylib
    └── macos-x86_64/
        └── libmy_extension.dylib
```

### 6.2 `manifest.json` Format

```json
{
  "id": "author.my_extension",
  "name": "My Extension",
  "version": "0.1.0",
  "author": "Your Name",
  "description": "Interactive image processing and sidebar tool.",
  "api_version": 1
}
```

### 6.3 Platform Key Matrix

| Platform Key | OS | Architecture | File Extension |
| :--- | :--- | :--- | :--- |
| `windows-x86_64` | Windows | 64-bit x86 | `.dll` |
| `windows-aarch64` | Windows | ARM64 | `.dll` |
| `linux-x86_64` | Linux | 64-bit x86 | `.so` |
| `linux-aarch64` | Linux | ARM64 | `.so` |
| `macos-x86_64` | macOS | Intel | `.dylib` |
| `macos-aarch64` | macOS | Apple Silicon (M-series) | `.dylib` |

### 6.4 Packaging Script Example

You can package your `.opx` bundle with Python, Bash, or PowerShell:

```bash
# Build release dynamic library
cargo build --release

# Prepare bundle directory
mkdir -p bundle/bin/windows-x86_64
cp target/release/opsis_my_extension.dll bundle/bin/windows-x86_64/
cp manifest.json bundle/

# Create .opx archive (standard ZIP)
cd bundle
zip -r ../opsis_my_extension.opx manifest.json bin/
cd ..
```

---

## 7. Testing, Discovery & Debugging

### 7.1 Discovery Locations

Opsis automatically discovers and loads extensions from two locations:

1. **Portable Mode**: `<exe_directory>/extensions/`
   - Ideal during development: build directly to target and test immediately.
2. **System User Profile**:
   - **Windows**: `%APPDATA%\Opsis\extensions\`
   - **Linux**: `~/.config/opsis/extensions/`
   - **macOS**: `~/Library/Application Support/Opsis/extensions/`

### 7.2 Drag-and-Drop Installation

You can test your extension instantly by opening **Settings &rarr; Extensions** (<kbd>S</kbd>) in Opsis and dragging your `.opx`, `.dll`, `.so`, or `.dylib` file directly into the drop zone.

### 7.3 Dev Console Logging

Launch Opsis with debug flags to inspect the extension discovery, loading duration, and capability registration in real time:

```bash
# Launch with verbose debug logs
opsis -v

# Launch with deepest trace logs
opsis --log-level=trace

# Or use environment variable
OPSIS_LOG=debug opsis
```

Example log output:
```text
[0.014s] [DEBUG] [Opsis Extensions] Discovered 1 extension candidates across 2 scan directories
[0.018s] [INFO] [Opsis Extensions] Loaded extension: 'author.my_extension' v0.1.0 in 3.42ms
```

---

## 8. Complete End-to-End Example

Here is a complete, production-ready extension that registers an **Invert Colors** filter, an **`N`-Panel Sidebar Tab**, and a **Rebindable Keyboard Shortcut**:

```rust
use bytes::Bytes;
use freya::prelude::*;
use opsis_extension_api::{
    ActionDefinition, ActionHandler, EventAction, ExtensionManifest, ExtensionRegistry,
    ImageFilterProvider, InputContext, OpsisExtension, OverlayContext, SidebarTabProvider,
    CURRENT_API_VERSION,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct InvertExtension {
    inverted: Arc<AtomicBool>,
}

impl Default for InvertExtension {
    fn default() -> Self {
        Self {
            inverted: Arc::new(AtomicBool::new(false)),
        }
    }
}

// 1. Image Filter Implementation
struct InvertFilter {
    inverted: Arc<AtomicBool>,
}

impl ImageFilterProvider for InvertFilter {
    fn apply_filter(&self, rgba: &[u8], _width: u32, _height: u32) -> Option<Bytes> {
        if !self.inverted.load(Ordering::Relaxed) {
            return None;
        }
        let mut out = rgba.to_vec();
        for chunk in out.chunks_exact_mut(4) {
            chunk[0] = 255 - chunk[0];
            chunk[1] = 255 - chunk[1];
            chunk[2] = 255 - chunk[2];
        }
        Some(Bytes::from(out))
    }
}

// 2. Hotkey Action Handler
struct InvertActionHandler {
    inverted: Arc<AtomicBool>,
}

impl ActionHandler for InvertActionHandler {
    fn execute(&mut self, action_id: &str, _ctx: &InputContext) -> EventAction {
        if action_id == "invert_tool.toggle" {
            let current = self.inverted.load(Ordering::Relaxed);
            self.inverted.store(!current, Ordering::Relaxed);
            return EventAction::Handled;
        }
        EventAction::Pass
    }
}

// 3. Sidebar Tab View
struct InvertSidebarTab {
    inverted: Arc<AtomicBool>,
}

impl SidebarTabProvider for InvertSidebarTab {
    fn tab_title(&self) -> String {
        "Invert".to_string()
    }

    fn render_tab(&self, _ctx: &OverlayContext) -> Option<Element> {
        let is_on = self.inverted.load(Ordering::Relaxed);
        let state_text = if is_on { "ACTIVE" } else { "OFF" };
        let state_color = if is_on {
            Color::from_rgb(100, 220, 140)
        } else {
            Color::from_rgb(160, 160, 170)
        };

        Some(
            rect()
                .width(Size::fill())
                .padding(Gaps::new_all(12.0))
                .spacing(10.0)
                .direction(Direction::vertical())
                .children([
                    label()
                        .text("Color Inversion Tool")
                        .font_size(13.0)
                        .font_weight(FontWeight::BOLD)
                        .color(Color::from_rgb(240, 240, 245))
                        .into(),
                    label()
                        .text(format!("Filter Status: {state_text}"))
                        .font_size(11.0)
                        .color(state_color)
                        .into(),
                ])
                .into(),
        )
    }
}

// 4. Main Extension Entry Point
impl OpsisExtension for InvertExtension {
    fn manifest(&self) -> ExtensionManifest {
        ExtensionManifest {
            id: "community.invert-colors".to_string(),
            name: "Invert Colors".to_string(),
            version: "0.1.0".to_string(),
            author: "Opsis Developer".to_string(),
            description: "Real-time color inversion filter with shortcut and sidebar tab.".to_string(),
            api_version: CURRENT_API_VERSION,
        }
    }

    fn on_init(&mut self, registry: &mut ExtensionRegistry) -> Result<(), String> {
        // Register Image Filter
        registry.register_image_filter_provider(Box::new(InvertFilter {
            inverted: Arc::clone(&self.inverted),
        }));

        // Register Rebindable Hotkey Action
        registry.register_action(
            ActionDefinition {
                id: "invert_tool.toggle".to_string(),
                name: "Toggle Color Inversion".to_string(),
                category: "Color Tools".to_string(),
                default_keys: vec!["i".to_string()],
                description: "Toggles color inversion filter on the active image.".to_string(),
            },
            Box::new(InvertActionHandler {
                inverted: Arc::clone(&self.inverted),
            }),
        );

        // Register Sidebar Tab
        registry.register_sidebar_tab_provider(Box::new(InvertSidebarTab {
            inverted: Arc::clone(&self.inverted),
        }));

        Ok(())
    }
}

#[no_mangle]
pub unsafe extern "Rust" fn opsis_extension_create() -> Box<dyn OpsisExtension> {
    Box::new(InvertExtension::default())
}
```

---

## Need Help or Have Questions?

- Browse existing extensions at the [Opsis Extensions Repository](https://github.com/MrKowo/Opsis-extensions).
- Check the [`opsis_extension_api`](https://github.com/MrKowo/Opsis/tree/main/crates/opsis_extension_api) crate source.
- File issues or feature requests on the [Opsis Issue Tracker](https://github.com/MrKowo/Opsis/issues).
