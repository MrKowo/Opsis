# Media Pipeline Subsystem Specification

The Media Pipeline is responsible for discovering, streaming, decoding, caching, and linearizing image assets across diverse bit depths and color representations. It operates as an asynchronous data provider that hands native-depth pixel buffers to the Viewport Engine—which are lazily promoted to linear 32-bit float representation via GPU texture sampling and shader stages—while exposing arbitrary, extensible metadata to the application session and UI extensions. It strictly deals in standard renderable display buffers (`RGBA` / `Gray`) and generic metadata key-value stores. Domain-specific decoding, multi-channel parsing, container-specific metadata interpretation, and layer-to-display mapping are delegated entirely to decoder implementations.

---

## 1. Architectural Principles

* **Decoupled Decode & Render:** Decoding operates entirely in host memory on dedicated worker thread pools, independent of GPU texture state or UI loops.
* **Thin-Core Abstraction:** The core defines stream I/O traits, dispatch logic, and memory caching. Format-specific decoders are implemented as self-contained plugins interacting across a strict C-ABI FFI boundary.
* **Plugin-Delegated Layer & Channel Mapping:** Multi-channel or multi-part assets expose viewable layers via dynamic metadata and pack the requested layer into standard viewable pixel buffers during decode.
* **Open-Ended Metadata:** All non-raster data (orientation hints, layer manifests, capture settings, color profiles) is stored in a dynamic `MetadataStore`, keeping core pipeline data structures free of format-specific fields.
* **Lazy 32-Bit Promotion & Hardware Swizzling:** Non-float formats remain in their native bit depth inside RAM caches and during GPU upload to conserve memory and transfer bandwidth. Unsigned integer formats are bound as normalized textures (`UNORM`), relying on GPU hardware texture samplers to yield normalized floats on read. Single-channel (`Gray`) formats are allocated natively as 1-channel textures in VRAM and mapped to standard 4-channel `RGBA` vectors at zero runtime cost via hardware-level texture view swizzling `(R, R, R, 1.0)`.
* **Cooperative Cancellation:** Decoding tasks execute asynchronously on background workers. Plugins are provided a thread-safe cancellation context and are strictly required to poll it cooperatively to abort early when requested.

---

## 2. Decoder Discovery & Fast-Path Dispatch

To optimize decoder discovery, the pipeline utilizes an $O(1)$ fast-path resolution model, backed by an $O(N)$ sequential fallback:

```
                    [ Input Stream ]
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│ Tier 1: O(1) Candidate Match                            │
│  • Primary: Match Magic Bytes against Registry Table    │
│  • Secondary: Match File Extension Hint                 │
└──────────────────────────┬──────────────────────────────┘
                           │ Matches 1–2 Candidate Decoders
                           ▼
┌─────────────────────────────────────────────────────────┐
│ Tier 2: Targeted Header Sniff (<1 ms)                   │
│  • Execute `sniff_header()` on candidate decoders only  │
└──────────────────────────┬──────────────────────────────┘
                           │ (If candidates reject stream)
                           │ (Stream rewinds to byte 0)
                           ▼
┌─────────────────────────────────────────────────────────┐
│ Fallback: Priority Scan                                 │
│  • Sequential evaluation of decoders by Priority Score  │
└─────────────────────────────────────────────────────────┘
```

* **Tier 1 (Fast Match):** Matches the stream's leading bytes or file extension against the registry populated by each plugin's `FfiDecoderDescriptor`.
* **Tier 2 (Targeted Verification):** Calls `sniff_header()` strictly on matched candidates to extract dimensions, populate metadata properties, and validate internal headers in $<1\text{ ms}$.
* **Fallback Scan:** Iterates over remaining decoders in descending `default_priority` order only if Tier 1 yields no valid candidates or candidate headers fail verification. The host unconditionally issues a `seek(0, SEEK_SET)` before invoking subsequent candidates.

---

## 3. Data Contracts & FFI Interfaces

> [!NOTE]
> All shared C-ABI types (`FfiSlice`, `FfiErrorCode`, `FfiStreamVTable`, `FfiDecoderPlugin`) are canonically defined in the [`ffi`](Extensions%20Pipeline.md#31-canonical-c-abi-core-crate-ffi--cratesffi) shared crate (`crates/ffi` / `opsis_ffi_core.h`).

To support dynamic hot-loading of `.opx` extension packages across language boundaries, the pipeline enforces a strict C-ABI interface.

```rust
use std::ffi::c_void;

#[repr(i32)]
pub enum FfiErrorCode {
    Success = 0,
    UnsupportedFormat = -1,
    CorruptHeader = -2,
    Cancelled = -3,
    IoError = -4,
    InvalidLayer = -5,
    BufferTooSmall = -6,
    InvalidParameter = -7,
}

#[repr(u32)]
pub enum NativePixelFormat {
    Rgba8Unorm = 0,
    Rgba16Unorm = 1,
    RgbaF32 = 2,
    Gray8Unorm = 3,
    Gray16Unorm = 4,
    GrayF32 = 5,
}

#[repr(u32)]
pub enum MetadataValueType {
    Utf8String = 0, // UTF-8 encoded text in value_data
    Int64 = 1,      // 8-byte signed integer, Little-Endian
    Float64 = 2,    // 8-byte IEEE 754 float, Little-Endian
    Bytes = 3,      // Raw binary blob (e.g., ICC Profile)
    Json = 4,       // UTF-8 encoded JSON document
}

#[repr(C)]
pub struct FfiSlice {
    pub ptr: *const u8,
    pub len: usize,
}

#[repr(C)]
pub struct FfiMagicPattern {
    pub offset: usize,
    pub pattern: FfiSlice,
}

#[repr(C)]
pub struct FfiStreamVTable {
    pub host_stream_ptr: *mut c_void,
    /// Returns bytes read (>= 0) or negative FfiErrorCode
    pub read: extern "C" fn(stream: *mut c_void, buf: *mut u8, buf_len: usize) -> i64,
    /// whence: 0 = SEEK_SET, 1 = SEEK_CUR, 2 = SEEK_END. Returns new offset (>= 0) or negative FfiErrorCode
    pub seek: extern "C" fn(stream: *mut c_void, offset: i64, whence: u32) -> i64,
}

#[repr(C)]
pub struct FfiContext {
    pub host_context_ptr: *mut c_void,
    pub is_cancelled: extern "C" fn(ctx: *mut c_void) -> bool,
    pub insert_metadata: extern "C" fn(
        ctx: *mut c_void,
        key: FfiSlice,
        value_type: MetadataValueType,
        value_data: FfiSlice,
    ),
}

#[repr(C)]
pub struct FfiImageMetadata {
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub default_format: NativePixelFormat,
    pub total_layers: u32,
}

#[repr(u32)]
pub enum AssetPayloadKind {
    Raster = 0,    // 2D Pixel bitmap buffer
    Geometry = 1,  // 3D Mesh / Vector Vertex+Index data
    Custom = 2,    // Extension-managed scene descriptor / procedural stream
}

#[repr(C)]
pub struct FfiGeometryBuffer {
    pub vertex_data_ptr: *const u8,
    pub vertex_data_len: usize,
    pub index_data_ptr: *const u8,
    pub index_data_len: usize,
    pub index_type: u32, // 0 = None, 1 = U16, 2 = U32
    pub pipeline_hint_id: FfiSlice, // Optional hint for default shader pipeline (e.g. "opsis.render.mesh_pbr")
}

#[repr(C)]
pub struct FfiImageLayer {
    pub layer_index: u32,
    pub payload_kind: AssetPayloadKind,
    pub origin_x: i32,
    pub origin_y: i32,
    pub width: u32,
    pub height: u32,
    /// Must be aligned to 256 bytes for Vulkan linear upload compatibility
    pub stride_bytes: usize,
    pub format: NativePixelFormat,

    pub data_ptr: *mut u8,
    pub data_len: usize,

    /// Set if payload_kind == AssetPayloadKind::Geometry
    pub geometry_payload: FfiGeometryBuffer,

    /// Context pointer passed back to free_callback (allocator state)
    pub user_data: *mut c_void,
    pub free_callback: extern "C" fn(ptr: *mut u8, len: usize, user_data: *mut c_void),
}

#[repr(C)]
pub struct FfiDecoderDescriptor {
    pub id: FfiSlice,
    pub default_priority: u32,
    pub supported_extensions: FfiSlice,
    pub magic_patterns: *const FfiMagicPattern,
    pub magic_patterns_count: usize,
}

/// Dynamic Decoder Instance Table (Stateful per decode session)
#[repr(C)]
pub struct FfiDecoderInstance {
    pub plugin_state: *mut c_void,
    pub sniff_header: extern "C" fn(
        instance: *mut c_void,
        stream: *const FfiStreamVTable,
        ctx: *const FfiContext,
        out_meta: *mut FfiImageMetadata,
    ) -> i32,
    pub decode_layer: extern "C" fn(
        instance: *mut c_void,
        stream: *const FfiStreamVTable,
        layer_index: u32,
        ctx: *const FfiContext,
        out_layer: *mut FfiImageLayer,
    ) -> i32,
    pub destroy: extern "C" fn(instance: *mut c_void),
}

#[repr(C)]
pub struct FfiDecoderPlugin {
    pub get_descriptor: extern "C" fn(out_desc: *mut FfiDecoderDescriptor) -> i32,
    pub create_instance: extern "C" fn(out_instance: *mut FfiDecoderInstance) -> i32,
}
```

---

## 4. Execution Pipeline & Data Handoff

```
[ Input Stream ] ──► [ O(1) Fast-Path Match ] ──► [ Sniff Header (<1ms) ]
                                                           │
                                                           ▼
                                                [ Loaded Image Payload ]
                                                           │
                                           ┌───────────────┴───────────────┐
                                           ▼                               ▼
                                 ┌───────────────────┐           ┌───────────────────┐
                                 │  Render Pipeline  │           │   Session State   │
                                 │   (GPU Canvas)    │           │    (UI & Exts)    │
                                 └───────────────────┘           └───────────────────┘
```

1. **Stream Instantiation:** Input files, memory slices, or virtual archive entries are wrapped into thread-safe host streams. When concurrent layer decodes are scheduled, the host provisions dedicated stream handles with isolated cursor state per worker thread.

2. **Directory Indexing:** Sibling files are indexed using natural alphanumeric ordering to establish directory navigation indices.

3. **Header Sniffing:** A matched candidate decoder inspects stream headers to determine container dimensions, layer count, and injects custom metadata via `FfiContext::insert_metadata` within $<1\text{ ms}$.

4. **Cooperative Async Decoding:** The requested layer decodes in a worker threadpool. The `FfiContext::insert_metadata` callback is thread-safe; plugins may safely inject supplementary metadata discovered concurrently during decompression.

5. **LRU Caching:** Uncompressed native buffers are retained in a thread-safe LRU cache (`moka`), keyed by `(source_uri, layer_index, timestamp)` and evicted based on byte footprint.

6. **Data Handoff Split:**
* **GPU Render Pipeline:** Consumes visual-critical fields (`data_ptr`, `stride_bytes`, `width`, `height`, `format`).

* **Native Texture Allocation:** Buffers are uploaded directly into matching GPU formats (`R8_UNORM`, `R16_UNORM`, `R32_FLOAT`, `RGBA8_UNORM`, `RGBA16_UNORM`, `RGBA32_FLOAT`).

* **Hardware Sampling Promotion:** Promotion to 32-bit linear float is executed automatically by GPU sampler units and shader passes, avoiding redundant CPU-side conversion overhead.

* **Texture View Swizzling:** Single-channel textures map via Shader Resource View swizzling to `(R, R, R, 1.0)`.

* **Color Transforms:** Color space transformations and orientation matrices are evaluated in the fragment stage using parameters read from the `MetadataStore`.

* **Event Bus Notifications:** Upon successful header sniff or layer decode, the pipeline publishes `FfiEventId::MediaOpened`, `FfiEventId::LayerChanged`, and `FfiEventId::MetadataUpdated` to the `FfiEventBus`, notifying active UI panels and render nodes immediately.

* **Application Session State:** Retains the `MetadataStore` in CPU memory, emitting change events for UI extensions, inspectors, and HUD overlays.

---

## 5. Decoder Provider & Registration Architecture

Decoders are supplied through two channels: **Core Built-in Providers** and **Extension Packages (`.opx`)**, registering into a shared `DecoderRegistry`.

### Decoder Sources

**1. Core Built-in Providers (Static)**

* Statically compiled into the host binary using standard, lightweight decoding crates.
* Explicitly implement the FFI traits internally to mirror external plugin architecture.
* Guaranteed fallback baseline with zero dynamic linking overhead.

**2. Extension Packages (`.opx` Bundles)**

* Distributed as `.opx` packages (zip archives containing a manifest, metadata, and platform-specific shared libraries: `.dll`, `.so`, `.dylib`).
* Loaded dynamically via the operating system's native dynamic linker (`LoadLibrary` / `dlopen`).
* Plugins export the canonical dynamic entry symbol: `extern "C" fn opsis_plugin_init(host_version: u32, out_manifest: *mut FfiPluginManifest) -> i32`, populating the `media_decoder` capability VTable.

### Registration Lifecycle

1. **Discovery & Dynamic Loading:** The Extension Manager registers static built-in decoders and scans extension paths to link `.opx` libraries.
2. **Descriptor Ingestion:** The host calls `get_descriptor()` to obtain static metadata without invoking header sniffs.
3. **Registry Indexing:** The `DecoderRegistry` indexes `magic_bytes` and `supported_extensions` into fast-path lookup hash tables and sorts the fallback list by `default_priority`.
4. **Memory Lifecycle Management:** Upon eviction from the host LRU cache, the host invokes `FfiImageLayer::free_callback`, releasing buffer memory back to the allocating plugin.

---

## 6. Error Handling, Fuzz Testing & Crash Resiliency

* **C-ABI Error Codes:** Plugins return concrete negative integer codes defined in `FfiErrorCode` (`0` for success, negative values for failures).
* **Decoder Fuzz-Testing Standards:** Decoders must satisfy strict fuzzing criteria. When presented with truncated, corrupted, or adversarial random bitstreams, decoders **must never segfault, panic, or execute out-of-bounds reads**. They are required to return `FfiErrorCode::CorruptHeader` or `FfiErrorCode::IoError` deterministically in $< 5\text{ ms}$.
* **Panic Boundary Isolation:** The host invokes all third-party decoder entry points (`sniff_header`, `decode_layer`) inside `std::panic::catch_unwind`. If an unhandled panic occurs within a decoder, the host intercepts it, isolates the worker thread, and presents a fallback error card.
* **Unsupported Formats:** If all fast-path candidates and fallback priority scans return `FfiErrorCode::UnsupportedFormat` (or fail validation), the pipeline bubbles up an unsupported error event.
* **Graceful Fallback Card:** The host UI displays a fallback inspection card detailing file path, container size, and detected byte signatures without interrupting application stability.
* **Cursor Retention:** Directory navigation state is preserved across unsupported files, allowing navigation forward or backward to adjacent valid siblings.
