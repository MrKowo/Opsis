# Render Pipeline Subsystem Specification (Vulkan Engine)

The Render Pipeline is responsible for ingesting native-depth pixel buffers and custom asset geometry from the Media Pipeline, uploading resources to GPU memory, evaluating geometric transformations, executing resampling reconstruction filters, compositing visual layers (2D images, vector scenes, 3D meshes), and mapping high-dynamic-range (HDR) or wide-gamut source colors to the presentation swapchain.

It is built directly on **Vulkan** (via `ash` in Rust) utilizing **offline precompiled SPIR-V bytecode**, zero runtime shader compilation, pipeline caching, and explicit memory allocation for instantaneous cold startup ($< 15\text{ ms}$ to first frame) across Windows, Linux, and macOS (via MoltenVK).

---

## 1. Architectural Principles & Performance Budgets

* **Instant Cold Start ($< 15\text{ ms}$ First Frame):** All core shaders (vertex transform, sampling kernels, color transforms) are compiled to SPIR-V bytecode at build time (`include_bytes!`) and loaded into `VkShaderModule` with zero runtime translation or validation overhead.
* **Polymorphic Scene Graph (2D Images, Vector, & 3D Geometry):** The viewport operates on an ordered scene of generic **Render Nodes**. While 2D raster images are the high-performance default, nodes can represent 2D textures, vector curves, or 3D geometry (meshes, point clouds) with extension-provided graphics pipelines.
* **Unified Offscreen FP32 Target:** All scene nodes (whether 2D quads, procedural shaders, or 3D meshes) render into a shared 32-bit linear floating-point workspace (`VK_FORMAT_R32G32B32A32_SFLOAT`), ensuring consistent multi-layer blending, depth testing, HDR tone mapping, and color gamut management.
* **Native Texture Allocation & Zero-Cost Swizzling:** Decoded raster buffers retain their source bit depth in VRAM. Grayscale formats (`R8`, `R16`, `R32F`) are bound natively as 1-component textures and swizzled to `(R, R, R, 1.0)` at zero runtime cost via `VkImageViewCreateInfo.components`.
* **Demand-Driven Reactive Scheduling ($0\%$ Idle Load):** The render loop is strictly event-driven. Render passes are recorded and submitted only when user input occurs or the `FfiEventBus` emits invalidation events. Static idle state uses $0.0\%$ CPU and $0.0\%$ GPU.
* **Strict Performance & Memory Guardrails:**
  - **Cold Startup Budget:** $\le 15\text{ ms}$ from process invocation to swapchain presentation.
  - **Idle Memory Footprint:** $\le 50\text{ MB}$ RSS on cold boot with no media loaded.
  - **Resampling Filter Pass Budget:** $\le 1.5\text{ ms}$ for 4K raster presentation pass on integrated GPUs.
  - **Transform Event Throttling:** Viewport camera panning/zooming publishes `FfiEventId::TransformChanged` throttled to 120Hz display refresh rates.

---

## 2. Pipeline Execution Graph

```
                                [ Media Pipeline ]
                                        │
                         ┌──────────────┴──────────────┐
                         │                             │
               (Decoded Raster Buffer)       (Decoded 3D / Custom Data)
                         ▼                             ▼
       ┌───────────────────────────────┐ ┌───────────────────────────────┐
       │ 1A. Staging & Texture Upload  │ │ 1B. Vertex / Index Buffer Alloc│
       │  • Host-visible ring staging  │ │  • GPU mesh / geometry upload │
       └───────────────┬───────────────┘ └─────────────┬─────────────────┘
                       │                               │
                       └───────────────┬───────────────┘
                                       │
                                       ▼
                     ┌─────────────────────────────────────┐
                     │ 2. Scene Graph Render Subpass       │
                     │  • Target: Linear FP32 Workspace    │
                     │  • Depth/Stencil testing (optional) │
                     │  • 2D Quads: Resampling + Transforms│
                     │  • 3D Nodes: Extension Mesh Shaders │
                     │  • Node Compositing / Blend Stage   │
                     └──────────────────┬──────────────────┘
                                        │
                                        ▼
                     ┌─────────────────────────────────────┐
                     │ 3. Color Transform & Post-Process   │
                     │  • Source EOTF $\to$ Linear Space   │
                     │  • Color matrix / 3D LUT transform  │
                     │  • HDR Tone Mapping (ACES, Reinhard)│
                     │  • Output Color Gamut $\to$ OETF    │
                     └──────────────────┬──────────────────┘
                                        │
                                        ▼
                     ┌─────────────────────────────────────┐
                     │ 4. Overlay & Presentation           │
                     │  • Custom UI / Tool gizmo passes    │
                     │  • Swapchain presentation           │
                     │  • vkQueuePresentKHR                │
                     └─────────────────────────────────────┘
```

---

## 3. Polymorphic Scene Graph & Render Node Model

> [!NOTE]
> All shared C-ABI types (`FfiSlice`, `FfiPipelineDescriptor`) are canonically defined in the [`ffi`](Extensions%20Pipeline.md#31-canonical-c-abi-core-crate-ffi--cratesffi) shared crate (`crates/ffi` / `opsis_ffi_core.h`).

The rendering engine represents visual entities inside the viewport through a polymorphic `RenderNode`:

```rust
use ash::vk;
use std::sync::Arc;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResamplingFilter {
    Nearest = 0,
    Bilinear = 1,
    Bicubic = 2,
    Lanczos3 = 3,
    CatmullRom = 4,
    Custom = 5, // Extension pipeline registered via FfiPipelineDescriptor
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct NodeTransform {
    pub model_matrix: [[f32; 4]; 4], // 4x4 Affine / 3D transformation matrix
    pub uv_bounds: [f32; 4],        // [u_min, v_min, u_max, v_max] for 2D sampling
    pub opacity: f32,               // Layer opacity (0.0 to 1.0)
    pub _pad: [f32; 3],
}

#[derive(Debug, Clone)]
pub enum NodePayload {
    /// Standard 2D raster image with sampling filter
    Texture {
        image: Arc<VulkanImageHandle>,
        filter: ResamplingFilter,
        custom_filter_pipeline_id: Option<u32>,
    },
    /// 3D Mesh or custom geometry rendered with an extension pipeline
    Geometry {
        vertex_buffer: vk::Buffer,
        index_buffer: Option<vk::Buffer>,
        index_count: u32,
        index_type: vk::IndexType,
        pipeline_id: u32, // ID registered via FfiShaderStageDescriptor
    },
    /// Custom procedural / generative shader node
    Procedural {
        pipeline_id: u32,
    },
}

#[derive(Debug, Clone)]
pub struct RenderNode {
    pub id: u64,
    pub payload: NodePayload,
    pub transform: NodeTransform,
    pub compositor_id: u32,       // ID of blend operator (Core or Extension)
    pub uniform_data: Vec<u8>,    // Arbitrary push-constant / uniform bytes
    pub is_visible: bool,
}

#[derive(Debug, Clone)]
pub struct RenderScene {
    pub view_matrix: [[f32; 4]; 4],       // Camera View Matrix (supports 2D Pan/Zoom and 3D Orbit)
    pub projection_matrix: [[f32; 4]; 4], // Orthographic or Perspective Projection
    pub nodes: Vec<RenderNode>,            // Ordered rendering list
    pub color_config: ColorConfig,         // Color management & tone mapping configuration
    pub clear_color: [f32; 4],
}
```

---

## 4. Extension Points & Custom Pipeline Registration

Extensions can hook into the render pipeline across four specific interfaces:

```
                  ┌────────────────────────────────────────┐
                  │ 1. Resampling Filter Stage             │
                  │    • Custom 2D reconstruction kernels  │
                  └──────────────────┬─────────────────────┘
                                     │
                                     ▼
                  ┌────────────────────────────────────────┐
                  │ 2. Custom Node / 3D Geometry Pipeline  │
                  │    • Vertex + Fragment SPIR-V shaders  │
                  │    • Custom mesh, point cloud, vector  │
                  └──────────────────┬─────────────────────┘
                                     │
                                     ▼
                  ┌────────────────────────────────────────┐
                  │ 3. Node Compositor / Blend Stage       │
                  │    • Custom multi-surface blend math   │
                  └──────────────────┬─────────────────────┘
                                     │
                                     ▼
                  ┌────────────────────────────────────────┐
                  │ 4. Post-Processing & HUD Overlay Stage │
                  │    • Fullscreen filters & 3D gizmos    │
                  └────────────────────────────────────────┘
```

### 4.1 FFI Pipeline Descriptor (`FfiPipelineDescriptor`)

To enable extensions to render 3D geometry or procedural shaders without touching core code, extensions register full graphics pipeline descriptors:

```rust
use std::ffi::c_char;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineKind {
    ResamplingFilter = 0,     // 2D image sampling kernel
    CustomGeometry = 1,       // 3D mesh / vertex shader + fragment shader
    NodeCompositor = 2,       // Multi-layer blending operator
    PostProcessEffect = 3,    // Fullscreen tone/image processing filter
}

#[repr(C)]
pub struct FfiPipelineDescriptor {
    pub id: FfiSlice,                      // e.g. "opsis.render.mesh_pbr"
    pub display_name: FfiSlice,
    pub kind: PipelineKind,
    pub vertex_spirv_ptr: *const u8,       // SPIR-V Vertex shader (for CustomGeometry)
    pub vertex_spirv_len: usize,
    pub fragment_spirv_ptr: *const u8,     // SPIR-V Fragment shader
    pub fragment_spirv_len: usize,
    pub enable_depth_test: bool,           // Enable VkPipelineDepthStencilStateCreateInfo
    pub cull_mode: u32,                    // VkCullModeFlags
    pub uniform_buffer_size: usize,        // Push constant / uniform payload size
}
```

---

## 5. Viewport Camera Model (2D Canvas + 3D Orbit/Perspective)

To seamlessly accommodate both 2D image navigation and 3D object inspection, the viewport camera provides dual projection modes:

```rust
#[derive(Debug, Clone, Copy)]
pub enum CameraProjection {
    /// 2D Infinite Canvas (subpixel pan, exponential zoom, 2D rotation)
    Orthographic {
        center: [f32; 2],
        zoom: f32,
        rotation_rad: f32,
        pixel_snapping: bool,
    },
    /// 3D Inspection (Orbit, Dolly, Pan, Field of View)
    Perspective {
        target: [f32; 3],
        distance: f32,
        yaw_rad: f32,
        pitch_rad: f32,
        fov_y_rad: f32,
        near_clip: f32,
        far_clip: f32,
    },
}
```

---

## 6. Color Management & HDR Pipeline

Regardless of whether a layer is a 2D photograph or a rendered 3D model, it is composited into the linear FP32 buffer and processed identically:

```
┌──────────────────┐
│ Scene Node Draw  │ (2D Image Quads, 3D Meshes, Procedural Shaders)
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Connection Space │ Linear FP32 Compositing Buffer (`VK_FORMAT_R32G32B32A32_SFLOAT`)
└────────┬─────────┘ (Compositing, depth testing, and blending occur in linear light)
         │
         ▼
┌──────────────────┐
│ Color Transform  │ 3D LUT Texture (`VK_IMAGE_TYPE_3D`) or 3x3 Matrix Transform
└────────┬─────────┘ (ICC profile color space matching, display gamut mapping)
         │
         ▼
┌──────────────────┐
│ Tone Mapping     │ HDR $\to$ SDR tone mapping (ACES Fitted, Reinhard, AgX)
└────────┬─────────┘ (Pass-through if Swapchain supports HDR10 / scRGB)
         │
         ▼
```



### 6.1 HDR Swapchain Negotiation & Wide-Gamut Output

* **Dynamic Swapchain Color Space Selection:**
  - **SDR Standard Displays:** Initializes `VK_FORMAT_B8G8R8A8_UNORM` / `VK_FORMAT_R8G8B8A8_SRGB` with `VK_COLOR_SPACE_SRGB_NONLINEAR_KHR`.
  - **HDR10 Displays (Windows HDR / macOS EDR):** Selects `VK_FORMAT_R16G16B16A16_SFLOAT` with `VK_COLOR_SPACE_EXTENDED_SRGB_LINEAR_EXT` (scRGB FP16) or `VK_FORMAT_A2B10G10R10_UNORM_PACK32` with `VK_COLOR_SPACE_HDR10_ST2084_EXT`.
* **Direct Extension Pass-Through:** Extension post-processing shaders can bypass tone mapping when an HDR display is detected, outputting linear floating-point values ($> 1.0$) directly to the screen.

---

## 7. Rust Vulkan Backend API Contract

```rust
use ash::vk;
use std::sync::Arc;

pub struct VulkanImageHandle {
    pub id: u64,
    pub image: vk::Image,
    pub allocation: gpu_allocator::vulkan::Allocation,
    pub view: vk::ImageView,
    pub format: vk::Format,
    pub width: u32,
    pub height: u32,
    pub descriptor_set: vk::DescriptorSet,
}

pub struct RenderContext {
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub device: ash::Device,
    pub physical_device: vk::PhysicalDevice,
    pub graphics_queue: vk::Queue,
    pub transfer_queue: vk::Queue,
    pub pipeline_cache: vk::PipelineCache,
}

pub trait VulkanRenderer {
    /// Ingests a native decoded buffer into GPU VRAM
    fn upload_texture(
        &mut self,
        ctx: &RenderContext,
        buffer: &DecodedPixelBuffer,
    ) -> Result<Arc<VulkanImageHandle>, RenderError>;

    /// Ingests vertex/index geometry buffers for 3D/vector nodes
    fn upload_geometry(
        &mut self,
        ctx: &RenderContext,
        vertices: &[u8],
        indices: Option<&[u8]>,
    ) -> Result<(vk::Buffer, Option<vk::Buffer>), RenderError>;

    /// Registers a custom plugin graphics pipeline (3D mesh shader, filter, or compositor)
    fn register_pipeline(
        &mut self,
        ctx: &RenderContext,
        descriptor: &FfiPipelineDescriptor,
    ) -> Result<u32, RenderError>;

    /// Updates the complete scene graph to be rendered
    fn set_scene(&mut self, scene: RenderScene);

    /// Records and presents the frame to the swapchain
    fn draw_frame(&mut self, ctx: &RenderContext) -> Result<(), RenderError>;
}
```
