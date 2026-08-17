# Extensions Pipeline Subsystem Specification

The Extensions Pipeline is the upstream orchestrator responsible for discovering, validating, loading, and managing the lifecycle of modular `.opx` extension packages and dynamic shared libraries (`.dll`, `.so`, `.dylib`). It bridges host subsystems (**Media Pipeline**, **Render Pipeline**, and **Window & UI Management**) with third-party capabilities across a strict, stable C-ABI FFI boundary.

It is designed for instantaneous application boot ($< 15\text{ ms}$ cold start), deterministic capability registration, complete memory isolation across boundaries, and hot-reloadable plugin development workflows.

---

## 1. Architectural Principles

* **Strict C-ABI Boundary:** All communication between the host core and extensions occurs via C-compatible structs (`#[repr(C)]`), raw pointers, explicit length-delimited slices (`FfiSlice`), and function pointer VTables. Extensions do not share Rust standard library allocators or crate dependencies with the host.
* **Unified Multi-Subsystem Manifest:** An extension is not restricted to a single responsibility. A single `.opx` package can bundle capabilities across decoders (Media Pipeline), custom SPIR-V shaders & mesh pipelines (Render Pipeline), and UI panels & tools (Window & UI Management).
* **Zero-Delay Lazy Initialization:** Extensions are discovered via fast metadata manifests (`manifest.json` headers) on boot in $< 1\text{ ms}$. Heavy dynamic library loading and GPU shader compilation occur strictly on demand when a relevant media format is opened or a tool is activated.
* **Host-Owned Allocations & Cooperative Memory Safety:** All pixel buffers, geometry buffers, and UI strings allocated by an extension are paired with explicit `free_callback` function pointers in their FFI descriptors, ensuring the allocating library frees its own heap memory without undefined allocator behavior.
* **Cooperative Cancellation & Fault Isolation:** Background plugin operations (e.g. progressive image decoding) receive cancellation tokens (`FfiContext`). Host wrapper layers guard all FFI invocations with exception/panic boundaries (`std::panic::catch_unwind`).

---

## 2. Multi-Component Package Model (`.opx`)

Opsis extensions are packaged as modular `.opx` archives containing native libraries, Python scripts, and SPIR-V shaders. For a complete guide on packaging and authoring extensions, see the **[Extension Specification & Authoring Guide](Extension%20Specification.md)**.

### 2.1 Manifest Schema Summary (`manifest.json`)

```json
{
  "schema_version": 1,
  "id": "com.opsis.gltf-suite",
  "name": "glTF 2.0 & 3D Viewer Suite",
  "version": "1.2.0",
  "author": "Opsis Contributors",
  "description": "Adds support for decoding and viewing 3D glTF models with real-time PBR shading and custom UI tools.",
  "entry_points": {
    "native_library": "bin/gltf_suite",
    "script": "scripts/material_inspector.py"
  },
  "capabilities": {
    "media_decoders": [
      {
        "id": "opsis.decoder.gltf",
        "extensions": [".gltf", ".glb"],
        "priority": 100,
        "provider": "native"
      }
    ],
    "render_pipelines": [
      {
        "id": "opsis.render.mesh_pbr",
        "kind": "CustomGeometry",
        "vertex_shader": "shaders/mesh.spv",
        "fragment_shader": "shaders/filter.spv"
      }
    ],
    "ui_panels": [
      {
        "id": "opsis.panel.material_inspector",
        "title": "Material Inspector",
        "category": "Item",
        "target_region": "Sidebar",
        "provider": "script"
      }
    ]
  }
}
```

### 2.2 Entry Points vs. Capabilities Architecture
* **`entry_points` (Physical Module Discovery):** Declares runnable binaries or scripts. Unqualified base paths (e.g. `"bin/gltf_suite"`) automatically resolve at runtime to platform targets (`bin/x86_64-pc-windows/gltf_suite.dll`, `bin/x86_64-unknown-linux/libgltf_suite.so`, `bin/aarch64-apple-darwin/libgltf_suite.dylib`).
* **`capabilities` (Functional Subsystem Hooking):** Maps specific decoders, render pipelines, or UI panels to their provider runtime (`"native"` or `"script"`).

---

## 3. Unified Plugin Lifecycle & FFI Interfaces

### 3.1 Canonical C-ABI Core Crate (`opsis-ffi-core`)

All shared C-ABI structures, error codes, and VTables are canonically defined in the shared `opsis-ffi-core` Rust crate and exported as `opsis_ffi_core.h` for C/C++ developers to prevent field misalignment or struct layout drift:

```rust
use std::ffi::c_void;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FfiSlice {
    pub ptr: *const u8,
    pub len: usize,
}

impl FfiSlice {
    pub fn from_str(s: &str) -> Self {
        Self {
            ptr: s.as_ptr(),
            len: s.len(),
        }
    }
}
```

### 3.2 Unified Plugin Manifest & Entry Point

Every extension shared library exports a single canonical symbol: `opsis_plugin_init`.

```rust
#[repr(C)]
pub struct FfiRenderPlugin {
    pub get_pipeline_count: extern "C" fn() -> usize,
    pub get_pipeline_descriptor: extern "C" fn(index: usize, out_desc: *mut FfiPipelineDescriptor) -> i32,
}

#[repr(C)]
pub struct FfiUiExtensionPlugin {
    pub get_operator_count: extern "C" fn() -> usize,
    pub get_operator_descriptor: extern "C" fn(index: usize, out_desc: *mut FfiOperatorDescriptor) -> i32,
    pub get_panel_count: extern "C" fn() -> usize,
    pub get_panel_descriptor: extern "C" fn(index: usize, out_desc: *mut FfiPanelDescriptor) -> i32,
}

#[repr(C)]
pub struct FfiPluginManifest {
    pub api_version: u32,               // Host ABI version (e.g. 1)
    pub plugin_id: FfiSlice,            // Unique identifier
    pub name: FfiSlice,                 // Human-readable name
    pub version: FfiSlice,              // Semantic version string
    
    // Subsystem VTables (Set to null pointer if capability not provided)
    pub media_decoder: *const FfiDecoderPlugin,       // Hooks into Media Pipeline
    pub render_pipelines: *const FfiRenderPlugin,     // Hooks into Render Pipeline
    pub ui_extensions: *const FfiUiExtensionPlugin,   // Hooks into UI & Window Management
}

/// The single exported dynamic library symbol
#[no_mangle]
pub extern "C" fn opsis_plugin_init(
    host_version: u32,
    out_manifest: *mut FfiPluginManifest,
) -> i32; // Returns 0 on success, negative error code on incompatibility
```

---

### 3.3 Host Event & Invalidation Bus (`FfiEventBus`)

To enable real-time synchronization between decoding tasks, viewport transforms, and extension panels without tight coupling, Opsis provides a zero-allocation, C-ABI event bus:

```rust
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FfiEventId {
    MediaOpened = 0,         // New file sniffed & opened (payload: path, dimensions)
    LayerChanged = 1,        // Active layer/frame index switched
    TransformChanged = 2,    // Viewport camera panned, zoomed, or rotated
    MetadataUpdated = 3,     // Dynamic metadata injected during decode
    PropertyModified = 4,    // Bound property scrubbed / updated
    ColorProfileChanged = 5, // Display or asset color space changed
}

#[repr(C)]
pub struct FfiEventPayload {
    pub event_id: FfiEventId,
    pub timestamp_ns: u64,
    pub sender_id: FfiSlice,
    pub data_json: FfiSlice,  // Lightweight contextual parameters (e.g. layer index, zoom EV)
}

pub type FfiEventCallback = extern "C" fn(
    user_data: *mut c_void,
    event: *const FfiEventPayload,
);

#[repr(C)]
pub struct FfiEventBusVTable {
    pub bus_handle: *mut c_void,
    pub subscribe: extern "C" fn(
        bus: *mut c_void,
        event_id: FfiEventId,
        callback: FfiEventCallback,
        user_data: *mut c_void,
    ) -> u64, // Returns subscription handle
    pub unsubscribe: extern "C" fn(bus: *mut c_void, subscription_handle: u64),
    pub publish: extern "C" fn(bus: *mut c_void, event: *const FfiEventPayload),
}
```

---

## 4. Subsystem Extension Points

An extension can connect into any or all of the three downstream subsystems:

```
                                [ .opx Extension ]
                                        │
                                        ▼ `opsis_plugin_init`
                       ┌─────────────────────────────────┐
                       │      Unified Plugin Manifest    │
                       └────────────────┬────────────────┘
                                        │
         ┌──────────────────────────────┼──────────────────────────────┐
         ▼                              ▼                              ▼
┌──────────────────┐           ┌──────────────────┐           ┌──────────────────┐
│  Media Pipeline  │           │ Render Pipeline  │           │   Window & UI    │
├──────────────────┤           ├──────────────────┤           ├──────────────────┤
│ • File Decoders  │           │ • SPIR-V Shaders │           │ • Themed Widgets │
│ • Header Sniffer │           │ • 3D Mesh Stages │           │ • Vector Canvas  │
│ • Raster/Geometry│           │ • Blend Operators│           │ • GPU Viewports  │
│   Stream Emits   │           │ • Tone/Post FX   │           │ • Action Hooks   │
└──────────────────┘           └──────────────────┘           └──────────────────┘
```

### 4.1 Media Pipeline Capability (`FfiDecoderPlugin`)
* **Discovery:** Registers magic byte patterns and file extension associations.
* **Decoding:** Implements `sniff_header` and `decode_layer`, emitting either `AssetPayloadKind::Raster` (2D bitmaps) or `AssetPayloadKind::Geometry` (3D vertex/index buffers).

### 4.2 Render Pipeline Capability (`FfiRenderPlugin`)
* **Pipelines:** Supplies precompiled SPIR-V bytecode (`FfiPipelineDescriptor`) for custom reconstruction filters, multi-layer blend operators, 3D geometry shaders, or fullscreen post-processing effects.

### 4.3 Window and UI Capability (`FfiUiExtensionPlugin`)
* **Declarative Panels:** Implements `draw` to construct declarative layout trees (`UILayout` rows, columns, boxes, and property data-binding) inside standard workspace regions (Header, T-Shelf toolbar, N-Panel sidebar, or Status Bar).
* **Operator Registration:** Injects custom `FfiOperatorDescriptor` instances (supporting `poll`, `invoke`, `modal`, `execute`, and `cancel`) and keymap bindings into the host engine.

### 4.4 Multi-Component Provider Execution (`native` vs. `script`)
The Extensions Pipeline host orchestrator evaluates the `provider` field declared in `manifest.json`:
* **`native` (Dynamic Shared Library):** Dynamically linked via `libloading` for zero-copy C-ABI execution. Used for high-throughput image decoders, raw point cloud parsers, and custom GPU render nodes.
* **`script` (Embedded Python):** Executed by the host's lazy-loaded Python runner (`pyo3`). 
  - **Multi-Module Package Mode:** If `entry_points.script` points to `scripts/__init__.py`, the host adds `scripts/` to `sys.path` and imports `__init__.py`, allowing clean submodule imports.
  - **Single-File Mode:** If `entry_points.script` points to a specific script (e.g. `scripts/material_inspector.py`), the host executes that script file directly.
  - Python panels call declarative layout functions (`layout.row()`, `layout.prop()`, `layout.operator()`) to construct UI and simple operators with zero compilation overhead.

---

## 5. Plugin Security, Verification & Safe Mode

To guarantee application stability, prevent malicious execution, and provide recovery from faulty community plugins:

### 5.1 CLI Recovery Flags (`--safe-mode` & `--no-plugins`)
* **`opsis --safe-mode`:** Starts Opsis with all third-party `.opx` packages temporarily disabled and hardware acceleration fallbacks enabled. Allows users to remove or reconfigure corrupted plugins.
* **`opsis --no-plugins`:** Launches a pure, minimal host instance bypassing all discovery directories (`< 10 ms` boot).

### 5.2 Manifest Integrity Verification (`sha256`)
Extensions can declare cryptographic checksums for packaged binaries in `manifest.json`:
```json
{
  "checksums": {
    "bin/x86_64-pc-windows/gltf_suite.dll": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    "scripts/material_inspector.py": "ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb"
  }
}
```
If enabled in `opsis.json`, the host verifies checksums prior to linking dynamic libraries, rejecting tampered packages.

### 5.3 Panic Isolation & Auto-Disabling
* **Host Panic Boundary:** Every dynamic library FFI call (`sniff_header`, `decode_layer`, `draw`, `execute`) is wrapped inside `std::panic::catch_unwind`.
* **Fault Recovery:** If an extension panics or returns an illegal state, Opsis immediately catches the error, unloads the plugin instance, flags the extension as `errored` in the session, and presents a non-blocking recovery toast in the UI without crashing the host process.

---

## 6. Plugin Discovery & Loading Lifecycle

```
[ Application Boot ]
       │
       ▼
┌─────────────────────────────────────────────────────────┐
│ 1. Check CLI Flags (--safe-mode / --no-plugins)         │
│  • If active: Bypass third-party directory scanning     │
└──────────────────────────┬──────────────────────────────┘
                           │ (Normal boot)
                           ▼
┌─────────────────────────────────────────────────────────┐
│ 2. Scan Extension Directories (< 1 ms)                  │
│  • AppData/plugins, Executable/plugins, ~/.config/opsis │
│  • Parse `manifest.json` headers only (No DLL load)     │
└──────────────────────────┬──────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│ 3. Register Metadata in Capability Catalogs             │
│  • Build Magic-Byte & File Extension Lookup Tables      │
│  • Register Action IDs in Command Palette Catalog       │
└──────────────────────────┬──────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│ 4. Lazy Dynamic Library Loading (On Demand)             │
│  • Verify SHA-256 integrity checksums (if configured)   │
│  • Load DLL via `libloading` under `catch_unwind`       │
│  • Invoke `opsis_plugin_init` & bind Event Bus          │
└─────────────────────────────────────────────────────────┘
```

---

## 7. Rust Host Manager API Contract

```rust
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ExtensionMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub manifest_path: PathBuf,
    pub library_path: PathBuf,
    pub is_enabled: bool,
    pub is_errored: bool,
}

pub trait ExtensionManager {
    /// Discovers all available extensions in plugin search paths without loading binaries
    fn discover_extensions(&mut self, search_paths: &[PathBuf]) -> Result<Vec<ExtensionMetadata>, ExtensionError>;

    /// Loads and activates an extension dynamic library on demand with panic guards
    fn load_extension(&mut self, extension_id: &str) -> Result<Arc<LoadedExtension>, ExtensionError>;

    /// Unloads an extension and frees associated host/GPU resources
    fn unload_extension(&mut self, extension_id: &str) -> Result<(), ExtensionError>;

    /// Accesses the host publish/subscribe event bus
    fn event_bus(&self) -> &FfiEventBusVTable;

    /// Returns all registered decoders across all active extensions
    fn get_active_decoders(&self) -> Vec<Arc<dyn MediaDecoder>>;
}
```
