# Extension Specification & Authoring Guide

This document is the definitive developer guide for authoring, packaging, and distributing modular **`.opx` extensions** for Opsis.

An Opsis extension can provide:

1. **Python UI Panels & Tool Operators:** High-velocity, Blender-style declarative UI and interactive tools.
2. **Native C-ABI Decoders & Algorithms:** Multi-threaded, hardware-accelerated image, video, or 3D geometry decoders.
3. **Precompiled SPIR-V GPU Shaders:** Custom image filters, 3D render pipelines, and post-processing effects.

---

## 1. Package Structure (`.opx`)

An `.opx` package is a standard ZIP archive (or uncompressed folder during development) containing metadata and assets:

```
my-extension.opx/
├── manifest.json            # Required: Extension metadata, capabilities, entry points
├── bin/ (optional)          # Native dynamic libraries (C / C++ / Rust / Zig)
│   ├── x86_64-pc-windows/   # Windows: my_ext.dll
│   ├── x86_64-unknown-linux/# Linux: my_ext.so
│   └── aarch64-apple-darwin/# macOS: my_ext.dylib
├── scripts/ (optional)      # Python UI panels, modal tools & operators
│   ├── __init__.py
│   └── inspector.py
└── shaders/ (optional)      # Precompiled SPIR-V GPU bytecode
    ├── filter.spv           # Fragment shader (reconstruction / post-FX)
    └── mesh.spv             # Vertex shader (3D geometry)
```

---

## 2. Complete Manifest Reference (`manifest.json`)

The `manifest.json` file is parsed on application boot to index capabilities without loading heavy code:

```json
{
  "schema_version": 1,
  "id": "com.opsis.hdr-suite",
  "name": "HDR & 3D Viewer Suite",
  "version": "1.2.0",
  "author": "Creative Tools Team",
  "description": "Adds OpenEXR multi-layer decoding, real-time waveform analysis, and tone curves.",
  "min_opsis_version": "1.0.0",

  "entry_points": {
    "native_library": "bin/hdr_suite",
    "script": "scripts/__init__.py"
  },

  "capabilities": {
    "media_decoders": [
      {
        "id": "opsis.decoder.exr_multichannel",
        "extensions": [".exr", ".sxr"],
        "priority": 100,
        "provider": "native"
      }
    ],
    "render_pipelines": [
      {
        "id": "opsis.render.tone_curve",
        "kind": "PostProcessEffect",
        "fragment_shader": "shaders/filter.spv"
      }
    ],
    "ui_panels": [
      {
        "id": "opsis.panel.waveform",
        "title": "Color Waveform & Curves",
        "category": "Color",
        "target_region": "Sidebar",
        "default_open": true,
        "provider": "script"
      }
    ]
  },

  "keymaps": [
    {
      "operator": "opsis.hdr.toggle_waveform",
      "key": "KeyW",
      "shift": true,
      "context": "Viewport"
    }
  ],

  "checksums": {
    "bin/x86_64-pc-windows/hdr_suite.dll": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    "scripts/__init__.py": "ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb"
  },

  "settings": {
    "waveform_resolution": {
      "type": "int",
      "default": 256,
      "min": 64,
      "max": 1024,
      "description": "Internal sampling resolution for the color waveform."
    }
  }
}
```

### 2.1 Entry Points vs. Capabilities Mapping

* **`entry_points` (Physical Modules):** Declares where the runnable code lives in the package.
  - **Native Base Path Resolution:** Setting `"native_library": "bin/hdr_suite"` instructs Opsis to automatically append the host platform triplet and dynamic library suffix:
    - **Windows:** `bin/x86_64-pc-windows/hdr_suite.dll`
    - **Linux:** `bin/x86_64-unknown-linux/libhdr_suite.so`
    - **macOS:** `bin/aarch64-apple-darwin/libhdr_suite.dylib`
  - **Script Entry Resolution:** Setting `"script": "scripts/__init__.py"` sets the root Python package entry point.
* **`capabilities` (Subsystem Hook Routing):** Registers individual functional features into host registries and routes them to their provider (`"native"` or `"script"`). A single extension can register decoders (Media Pipeline), custom shaders (Render Pipeline), and UI panels (Window Management).

---

## 3. Python Extension Development (`scripts/`)

Python extensions use a **declarative UI and Operator lifecycle**. Python code is lazy-loaded only when an associated tool or panel is opened.

### 3.1 Creating a Declarative UI Panel (`opsis.Panel`)

```python
import opsis

class WaveformInspectorPanel(opsis.Panel):
    id = "opsis.panel.waveform"
    label = "Color Waveform & Curves"
    category = "Color"               # Tab name in the Sidebar (N-Panel)
    target_region = "Sidebar"        # Header, Toolbar, Sidebar, or StatusBar

    @classmethod
    def poll(cls, context):
        """Only show panel if an active image is loaded."""
        return context.active_image is not None

    def draw(self, layout, context):
        """Constructs the declarative UI layout tree."""
        # Visual grouped container
        box = layout.box()
        box.label("Exposure & Gain", icon="brightness")

        # Two-way property data binding
        box.prop("image.exposure", label="Exposure (EV)")
        box.prop("image.gamma", label="Gamma")

        # Horizontal button row with auto-alignment
        row = layout.row(align=True)
        row.operator("opsis.color.auto_balance", label="Auto Balance")
        row.operator("opsis.color.reset", label="Reset")

        # Custom 2D interactive vector canvas (Tier 2)
        canvas = layout.allocate_custom_canvas(width=200, height=100)
        if canvas:
            canvas.draw_line((0, 50), (200, 50), color=(1.0, 1.0, 1.0, 0.3), width=1.0)
            canvas.draw_polyline(context.get_histogram_points(), color=(0.2, 0.8, 1.0, 1.0), width=2.0)
```

### 3.2 Creating an Operator (`opsis.Operator`)

Operators represent all interactive tools, hotkeys, and menu items:

```python
import opsis

class AutoExposeOperator(opsis.Operator):
    id = "opsis.color.auto_balance"
    label = "Auto Balance Exposure"
    description = "Calculates optimal exposure from image luminance histogram"

    def execute(self, context):
        """Instantaneous execution for menu clicks and hotkeys."""
        active_img = context.active_image
        if not active_img:
            return opsis.OperatorReturn.CANCELLED

        optimal_ev = active_img.calculate_optimal_ev()
        context.set_property("image.exposure", optimal_ev)
        return opsis.OperatorReturn.FINISHED
```

### 3.3 Creating a Modal Operator (Interactive Viewport Dragging)

Modal operators seize the event loop for continuous mouse interaction (e.g. crop drag, precision zoom, sampling brush):

```python
import opsis

class ColorSamplerTool(opsis.Operator):
    id = "opsis.tool.color_sampler"
    label = "Interactive Color Probe"

    def invoke(self, context):
        """Initializes modal interaction state."""
        context.set_status_bar_hint("[LMB Drag] Sample Pixel | [Esc / RMB] Cancel")
        return opsis.OperatorReturn.RUNNING_MODAL

    def modal(self, context):
        """Receives continuous mouse events while running."""
        if context.event_type == opsis.EventType.MOUSE_MOVE:
            rgba = context.sample_pixel(context.mouse_x, context.mouse_y)
            context.set_status_bar_text(f"RGB: ({rgba.r:.3f}, {rgba.g:.3f}, {rgba.b:.3f})")
            return opsis.OperatorReturn.RUNNING_MODAL

        elif context.event_type in (opsis.EventType.LEFT_MOUSE_UP, opsis.EventType.KEY_DOWN):
            return opsis.OperatorReturn.FINISHED

        elif context.event_type in (opsis.EventType.RIGHT_MOUSE_DOWN, opsis.EventType.KEY_UP):
            return opsis.OperatorReturn.CANCELLED

        return opsis.OperatorReturn.PASS_THROUGH
```

### 3.4 Subscribing to Host Events (`@opsis.on_event`)

Extensions can register reactive event listeners that fire on specific host state changes:

```python
import opsis

@opsis.on_event("MEDIA_OPENED")
def handle_new_media(event):
    """Fires when a new image is loaded."""
    print(f"Loaded: {event.data['path']} ({event.data['width']}x{event.data['height']})")

@opsis.on_event("TRANSFORM_CHANGED")
def handle_viewport_transform(event):
    """Fires when camera zooms or pans."""
    zoom_pct = event.data.get("zoom_level", 1.0) * 100
    # Update UI properties or tool states
```

---

## 4. Native C-ABI Development (`bin/`)

Native components deliver maximum performance for format decoders, heavy multi-threading, and hardware SIMD.

> [!NOTE]
> All canonical C-ABI struct and VTable definitions (`FfiSlice`, `FfiPluginManifest`, `FfiDecoderPlugin`, `opsis_plugin_init`) are defined in the canonical shared header `opsis_ffi_core.h` and Rust crate `opsis-ffi-core`.

### 4.1 Canonical Entry Point (`opsis_plugin_init`)

Every native shared library exports a single standard C symbol:

```rust
use std::ffi::c_void;

#[no_mangle]
pub extern "C" fn opsis_plugin_init(
    host_version: u32,
    out_manifest: *mut FfiPluginManifest,
) -> i32 {
    if host_version < 1 {
        return -1; // Incompatible host version
    }

    unsafe {
        (*out_manifest).api_version = 1;
        (*out_manifest).plugin_id = FfiSlice::from_str("com.opsis.hdr-suite");
        (*out_manifest).name = FfiSlice::from_str("OpenEXR Multichannel Suite");
        (*out_manifest).media_decoder = &EXR_DECODER_PLUGIN;
        (*out_manifest).render_pipelines = std::ptr::null();
        (*out_manifest).ui_extensions = std::ptr::null();
    }
    0 // Success
}
```

### 4.2 Implementing a High-Speed Media Decoder (`FfiDecoderPlugin`)

```rust
static EXR_DECODER_PLUGIN: FfiDecoderPlugin = FfiDecoderPlugin {
    get_descriptor: exr_get_descriptor,
    create_instance: exr_create_instance,
};

extern "C" fn exr_get_descriptor(out_desc: *mut FfiDecoderDescriptor) -> i32 {
    unsafe {
        (*out_desc).id = FfiSlice::from_str("opsis.decoder.exr_multichannel");
        (*out_desc).default_priority = 100;
        (*out_desc).supported_extensions = FfiSlice::from_str(".exr,.sxr");
        (*out_desc).magic_patterns = EXR_MAGIC.as_ptr();
        (*out_desc).magic_patterns_count = 1;
    }
    0
}

extern "C" fn exr_decode_layer(
    instance: *mut c_void,
    stream: *const FfiStreamVTable,
    layer_index: u32,
    ctx: *const FfiContext,
    out_layer: *mut FfiImageLayer,
) -> i32 {
    // 1. Cooperative cancellation check
    unsafe {
        if ((*ctx).is_cancelled)((*ctx).host_context_ptr) {
            return -3; // Cancelled
        }
    }

    // 2. Decode native pixel buffer (e.g. 16-bit float RGBA)
    let buffer = decode_raw_exr_stream(stream, layer_index);

    // 3. Populate FfiImageLayer with explicit free callback
    unsafe {
        (*out_layer).width = buffer.width;
        (*out_layer).height = buffer.height;
        (*out_layer).stride_bytes = buffer.stride;
        (*out_layer).format = NativePixelFormat::Rgba16Unorm;
        (*out_layer).data_ptr = buffer.data.as_mut_ptr();
        (*out_layer).data_len = buffer.data.len();
        (*out_layer).free_callback = exr_free_buffer;
    }
    0
}

extern "C" fn exr_free_buffer(ptr: *mut u8, len: usize, _user_data: *mut c_void) {
    // Safely deallocates memory using the plugin's own allocator
    unsafe {
        let _ = Vec::from_raw_parts(ptr, len, len);
    }
}
```

---

## 5. SPIR-V GPU Shader Integration (`shaders/`)

Extensions can supply precompiled SPIR-V fragment and vertex shaders directly to Vulkan.

### 5.1 Compiling Shaders to SPIR-V

Compile standard GLSL or Slang shaders offline to `.spv` using `glslc`:

```bash
# Compile fragment shader for a tone-curve post-processing effect
glslc -fshader-stage=fragment shaders/tone_curve.frag -o shaders/filter.spv
```

### 5.2 Uniform Buffer Contract

Fragment shaders receive standard viewport parameters via uniform binding 0:

```glsl
#version 450

layout(binding = 0) uniform ViewportUniforms {
    mat4 u_view_matrix;
    vec4 u_resolution;    // (width, height, dpi_scale, time)
    vec4 u_tone_params;   // Custom parameters from UI slider props
};

layout(binding = 1) uniform sampler2D u_source_texture;
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 out_color;

void main() {
    vec4 src = texture(u_source_texture, v_uv);
    // Apply custom tone curve
    out_color = pow(src, vec4(u_tone_params.x));
}
```

---

## 6. Testing & Distribution

### 6.1 Testing Locally During Development

1. Create a subfolder inside the Opsis plugins path:
   - **Portable Mode:** `<opsis_exe_folder>/plugins/my-extension/`
   - **Installed Mode (Windows):** `%APPDATA%/opsis/plugins/my-extension/`
   - **Installed Mode (Linux):** `~/.config/opsis/plugins/my-extension/`
2. Place `manifest.json`, `scripts/`, `bin/`, and `shaders/` directly inside.
3. Launch Opsis. The extension will be indexed immediately on boot.

### 6.2 Packaging into `.opx`

To package for distribution, simply ZIP the contents of the extension directory and rename the extension to `.opx`:

```bash
cd my-extension
zip -r ../com.opsis.hdr-suite.opx manifest.json bin/ scripts/ shaders/
```

Users can drag and drop `my-extension.opx` directly into the Opsis window to install.
