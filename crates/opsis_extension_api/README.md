# opsis_extension_api

[![Crates.io](https://img.shields.io/badge/crates.io-v0.1.0-orange)](https://crates.io)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../../LICENSE)

Official public API crate for building native modular extensions and universal `.opx` plugins for **[Opsis](https://github.com/MrKowo/Opsis)**.

---

## Overview

Opsis uses a **pure microkernel extension-first architecture**. Extensions are native dynamic libraries (`.dll`, `.so`, `.dylib`) or universal `.opx` archives that provide:

- **`SidebarTabProvider`**: Custom tabs in the Blender-style collapsible `N`-Panel.
- **`ActionHandler` / `ActionDefinition`**: Rebindable keyboard actions and commands integrated into the Settings shortcut manager.
- **`ImageFilterProvider`**: High-performance, non-destructive RGBA pixel transformation pipeline.
- **`OverlayProvider`**: Viewport HUDs, floating status widgets, and toolbars.
- **`ViewportProvider`**: Custom primary 2D/3D canvas renderers.
- **`InputInterceptor`**: Low-level keyboard and pointer event interception.
- **Native Floating Windows**: Spawning standalone detached OS windows via `ctx.launch_window`.

---

## Quickstart

Add this crate to your extension's `Cargo.toml`:

```toml
[package]
name = "opsis_my_extension"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
opsis_extension_api = { git = "https://github.com/MrKowo/Opsis.git" }
freya = { version = "0.4.1", default-features = false, features = ["engine"] }
bytes = "1.10"
serde = { version = "1.0", features = ["derive"] }
```

### Implementing `OpsisExtension`

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
            description: "A custom Opsis extension.".to_string(),
            api_version: CURRENT_API_VERSION,
        }
    }

    fn on_init(&mut self, registry: &mut ExtensionRegistry) -> Result<(), String> {
        // Register capability providers here
        Ok(())
    }
}

/// Exported dynamic constructor
#[no_mangle]
pub unsafe extern "Rust" fn opsis_extension_create() -> Box<dyn OpsisExtension> {
    Box::new(MyExtension)
}
```

---

## Documentation & Guides

For the complete developer guide covering all capability traits, universal `.opx` bundling, Freya UI integration, and testing workflows, see:

&rarr; **[Opsis Extension Developer Guide](../../docs/EXTENSIONS.md)**

---

## License

Licensed under the [MIT License](../../LICENSE).
