<p align="center">
  <img width="256" height="256" alt="Opsis logo" src="https://raw.githubusercontent.com/MrKowo/Opsis/main/assets/branding/logo.png" />
</p>

# Opsis

> **_Opsis_**: From Ancient Greek _ὄψις_ (ópsis), meaning *aspect*, *appearance*, *vision*, *spectacle*.

**Opsis** is an ultra-lightweight, fast, and modular image viewer built with **Rust**, **Freya** (powered by **Skia 2D** hardware-accelerated graphics and **Winit** windowing), and an **Extension-First Architecture**.

It is designed to be portable, minimal, and fully customizable through universal `.opx` extension bundles and native dynamic plugins. Nothing more, nothing less.

**AI use disclosure**: Opsis is built making large use of LLM technology. Human contributions will always be welcome!

---

## Features

- **Blazing Fast GPU Canvas**: Native Skia 2D raster and vector rendering pipeline.
- **Minimal Core**: Distraction-free image viewer with zero unnecessary UI clutter.
- **Smooth Viewport Controls**: Cursor-centered scroll zooming (5% to 5000%), click-and-drag panning, 1:1 pixel scaling, and window auto-fit.
- **Native File Dialog & Drag-and-Drop**: Open any supported image via native dialog (<kbd>O</kbd>), CLI argument, or by dragging and dropping files onto the window.
- **Broad Format Support**: PNG, JPEG, WebP, BMP, GIF, ICO, TIFF, TGA, HDR, AVIF, and SVG.
- **Modular Extension System**: Pluggable capabilities via [`opsis_extension_api`](crates/opsis_extension_api) for custom Viewports (`ViewportProvider`), UI overlays and HUDs (`OverlayProvider`), and hotkey handlers (`InputInterceptor`).
- **Universal Extension Bundles (`.opx`)**: Cross-platform ZIP packages containing multi-architecture native binaries (`windows`, `linux`, `macos`) with zero runtime overhead.
- **Built-in Settings Window**: Vertical tab navigation to inspect installed extensions, view keyboard shortcuts, and manage preferences.

---

## Keyboard Shortcuts

| Key | Action |
| :--- | :--- |
| <kbd>O</kbd> | Open native image file dialog |
| <kbd>+</kbd> / <kbd>=</kbd> | Zoom in (+25%) |
| <kbd>-</kbd> / <kbd>_</kbd> | Zoom out (-25%) |
| <kbd>0</kbd> | Reset zoom to 100% (1:1 pixel scale) |
| <kbd>F</kbd> | Fit image to window |
| <kbd>Escape</kbd> | Clear active image (return to base watermark) |
| <kbd>S</kbd> | Open Settings & Extension Manager window |

---

## Building from Source

### Prerequisites
- [Rust toolchain](https://rustup.rs/) (edition 2021, 1.80+ recommended)
- C++ build tools / CMake (for Skia compilation via Freya)

### Clone & Build

```bash
# Clone repository
git clone https://github.com/MrKowo/Opsis.git
cd Opsis

# Release build (recommended for performance)
cargo build --release

# Debug build
cargo build
```

---

## Running

The compiled executable is located at `target/release/opsis.exe` (on Windows).

```bash
# Launch empty viewer
cargo run --release

# Open a specific image
cargo run --release -- path/to/image.png
```

---

## Architecture & Dependencies

Opsis is composed of a microkernel host and public extension API crates:

| Component | Role |
| :--- | :--- |
| **`src/`** | Core host application (Skia canvas, native file I/O, window lifecycle, settings, and extension manager). |
| **[`crates/opsis_extension_api`](crates/opsis_extension_api)** | Public traits and ABI contracts for building custom Opsis extensions. |

### Core Dependencies
- **`freya`** — Modern GUI library for Rust powered by Skia 2D and Winit.
- **`image`** — Multi-format image decoding and metadata extraction.
- **`rfd`** — Native cross-platform file picker dialogs.
- **`libloading`** — Zero-overhead dynamic library loading for extensions.
- **`zip`** — Extraction and discovery for universal `.opx` extension packages.
- **`serde` / `serde_json`** — Serialization for extension manifests and configuration.

---

## Extension Development

Third-party extensions implement the [`OpsisExtension`](crates/opsis_extension_api/src/lib.rs) trait from `opsis_extension_api`:

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
            description: "Custom overlay or viewport extension.".to_string(),
            api_version: 1,
        }
    }

    fn on_init(&mut self, registry: &mut ExtensionRegistry) -> Result<(), String> {
        // Register ViewportProvider, OverlayProvider, or InputInterceptor
        Ok(())
    }

    fn on_unload(&mut self) {}
}

#[no_mangle]
pub unsafe extern "Rust" fn opsis_extension_create() -> Box<dyn OpsisExtension> {
    Box::new(MyExtension)
}
```

Extensions can be distributed as raw dynamic libraries (`.dll`, `.so`, `.dylib`) placed in the `extensions/` directory, or packaged into universal `.opx` ZIP archives containing a `manifest.json` and platform-specific binaries.

---

## Acknowledgments

Thank you to [Oculante](https://github.com/woelper/oculante) for providing the base inspiration for this project, and the [Freya](https://github.com/marc2332/freya) team for the high-performance GUI engine.

---

## License

Opsis is licensed under the [MIT License](LICENSE).
