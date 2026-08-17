# Window and UI Management Subsystem Specification

The Window and UI Management subsystem is responsible for managing OS window surfaces, driving the event loop with power-efficient demand-driven scheduling, handling multi-modal user input, and hosting an extensible, **Blender-inspired Declarative UI & Operator Architecture** (Layout Trees, Property Data-Binding, Modal Operators, and Structured Workspace Regions).

It combines Blender's proven ergonomic layout and tool paradigm with Rust's native speed, $< 15\text{ ms}$ cold start, $0\%$ idle CPU/GPU utilization, and a strict zero-cost C-ABI for extension plugins.

---

## 1. Architectural Principles

* **Blender-Inspired Declarative Layout Tree (`UILayout`):** UI is not drawn with imperative ad-hoc coordinates. Instead, panels and headers construct structured declarative layout trees (`row`, `column`, `grid_flow`, `box`, `split`) with automatic property data-binding, auto-alignment, and label-to-widget alignment.
* **Unified Operator System (`poll`, `invoke`, `modal`, `execute`):** All user actions, shortcuts, toolbar buttons, and viewport tools are implemented as **Operators**. Operators define strict lifecycle contracts for execution, modal mouse interaction (e.g. pan/zoom, interactive crop, brush/measurement drag), and automatic UI availability checking (`poll`).
* **Property Reflection & Two-Way Binding (`Props`):** Controls bind directly to typed property descriptors (Boolean, Int, Float with numeric drag scrubbing, Enum dropdowns/radio rows, Color, and String) with built-in validation ranges, default values, and undo/redo change notifications.
* **Structured Workspace Region Hierarchy:** Window surfaces are partitioned into standardized, collapsible regions:
  - **Header Region:** Compact top/bottom strip for menus, mode selectors, and quick toggles.
  - **Toolbar Region (T-Shelf):** Collapsible tool palette for active modal operators and viewing modes.
  - **Sidebar Region (N-Panel):** Tabbed, collapsible inspector categories housing modular `Panel` groups.
  - **Status Bar / Footer Region:** Context-sensitive shortcut hints, cursor readouts, and background task progress.
  - **Main Viewport Canvas:** High-framerate 120Hz+ subpixel canvas with overlaid HUD gizmos.
* **Demand-Driven Event Loop ($0\%$ Idle Load):** OS redraws and GPU frame submissions are triggered exclusively when user inputs occur, background decoder tasks finish, animations are active, or modal operators are running.
* **Native C-ABI Extension Safety:** Extension UI definitions use C-compatible structs (`#[repr(C)]`) and layout VTables, requiring zero runtime interpreter overhead while enabling hot-reloading and multi-language extension development (C, C++, Rust, Zig).

---

## 2. Subsystem Architecture & Frame Flow

```
                               [ OS Event Stream ]
                           (Mouse, Keys, Resize, Drops)
                                        │
                                        ▼
                      ┌─────────────────────────────────────┐
                      │ 1. Window & Event Dispatch (winit)  │
                      │  • Window state, DPI scale factor   │
                      │  • Hotkey & Keymap routing          │
                      │  • File drag-and-drop ingestion     │
                      └──────────────────┬──────────────────┘
                                         │
                                         ▼
                      ┌─────────────────────────────────────┐
                      │ 2. Active Modal Operator Evaluation │
                      │  • Intercepts input if modal tool   │
                      │    is active (e.g. Pan, Crop, Drag) │
                      │  • Returns RUNNING_MODAL / FINISHED │
                      └──────────────────┬──────────────────┘
                                         │ (If no modal operator captured input)
                                         ▼
                      ┌─────────────────────────────────────┐
                      │ 3. Region Hit-Testing & UI Layout   │
                      │  • Evaluates Header, T-Shelf,       │
                      │    N-Panel, & Status Bar layouts    │
                      │  • Dispatches pointer to UI widgets │
                      │  • Auto-triggers bound Operators    │
                      │  • Displays Safe Mode recovery badge│
                      └──────────────────┬──────────────────┘
                                         │ (If input NOT over UI widgets)
                                         ▼
                      ┌─────────────────────────────────────┐
                      │ 4. Viewport Navigation Fallback     │
                      │  • Default 2D Canvas Pan / Zoom     │
                      │  • 3D Orbit / Camera Navigation     │
                      └──────────────────┬──────────────────┘
                                         │
                                         ▼
                      ┌─────────────────────────────────────┐
                      │ 5. Frame Scheduling & Render Pass   │
                      │  • Records Vulkan Scene + UI passes │
                      │  • Applies Acrylic / DWM Backdrop   │
                      │  • Presents to swapchain            │
                      └─────────────────────────────────────┘
```

---

## 3. Workspace Region Layout & Minimalist Defaults

By default, **Opsis presents a pure, uncluttered viewport canvas**. Regions can be revealed on mouse hover, toggled via hotkeys (`T` for Toolbar, `N` for Sidebar), or configured to persist in workspace layouts:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ [Header Region]  File  View  Image  Filter  [Zoom: 100%] [Fit] [1:1]        │
├─────────┬───────────────────────────────────────────────────────┬───────────┤
│ [T-Bar] │                                                       │ [N-Panel] │
│ • Pan   │                                                       │ ┌────────┐│
│ • Zoom  │                                                       │ │Metadata││
│ • Crop  │                    VIEWPORT CANVAS                    │ ├────────┤│
│ • Sample│                 (Linear Vulkan Surface)               │ │Channels││
│ • 3D    │                                                       │ ├────────┤│
│         │                                                       │ │Pipeline││
│         │                                                       │ └────────┘│
├─────────┴───────────────────────────────────────────────────────┴───────────┤
│ [Status Bar]  RGBA: (0.12, 0.45, 0.88, 1.0) | 3840x2160 (Linear Float32)   │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3.1 Region Descriptions
* **Header Region:** Placed at the top or bottom of the window. Hosts context menus (`Menu`), view mode selectors, and quick-action operator buttons. Supports auto-hide and acrylic blur.
* **Toolbar Shelf (T-Shelf):** Left-aligned collapsible column housing single-column or double-column tool icons for selecting the active tool/operator.
* **Sidebar Inspector (N-Panel):** Right-aligned collapsible region featuring vertical category tabs (e.g. `Item`, `Metadata`, `Color Management`, `Extension Tools`). Each tab renders a vertical stack of collapsible `Panel` blocks.
* **Status Bar:** Bottom strip providing contextual operator shortcut hints (e.g. `[LMB] Confirm | [RMB] Cancel | [Shift] Precision`), cursor coordinates, color probe values, and background task progress bars.

---

## 4. Blender-Style Operator Lifecycle & Architecture

> [!NOTE]
> All shared C-ABI types (`FfiSlice`, `FfiOperatorDescriptor`, `FfiOperatorContext`, `FfiPanelDescriptor`) are canonically defined in the [`ffi`](Extensions%20Pipeline.md#31-canonical-c-abi-core-crate-ffi--cratesffi) shared crate (`crates/ffi` / `opsis_ffi_core.h`).

All interactive tools, commands, and shortcuts in Opsis implement the **Operator Contract**:

```rust
use std::ffi::c_void;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorReturn {
    /// Action completed successfully; state committed to undo stack
    Finished = 0,
    /// Action cancelled; temporary changes discarded
    Cancelled = 1,
    /// Operator seized the modal event loop (continues receiving input events)
    RunningModal = 2,
    /// Event was not handled by this operator; pass through to other listeners
    PassThrough = 3,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FfiEventType {
    None = 0,
    MouseMove = 1,
    LeftMouseDown = 2,
    LeftMouseUp = 3,
    RightMouseDown = 4,
    RightMouseUp = 5,
    MiddleMouseDown = 6,
    MiddleMouseUp = 7,
    MouseWheel = 8,
    KeyDown = 9,
    KeyUp = 10,
}

#[repr(C)]
pub struct FfiOperatorContext {
    pub host_ctx: *mut c_void,
    pub event_type: FfiEventType,
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub delta_x: f32,
    pub delta_y: f32,
    pub shift_held: bool,
    pub ctrl_held: bool,
    pub alt_held: bool,
}

#[repr(C)]
pub struct FfiOperatorDescriptor {
    pub id: FfiSlice,                  // e.g. "opsis.image.rotate_90"
    pub label: FfiSlice,               // e.g. "Rotate 90° Clockwise"
    pub description: FfiSlice,         // e.g. "Rotates active image buffer clockwise"
    pub category: FfiSlice,            // e.g. "Image", "Navigation", "Filter"
    pub flags: u32,                    // Undoable, Registered, Modal
    
    /// Determines whether the operator can run in the current context
    pub poll: extern "C" fn(ctx: *const FfiOperatorContext) -> bool,
    
    /// Executed for instantaneous operations (e.g. menu items, button clicks)
    pub execute: extern "C" fn(instance: *mut c_void, ctx: *const FfiOperatorContext) -> OperatorReturn,
    
    /// Initializes modal interaction (e.g. starting a brush drag or interactive pan)
    pub invoke: extern "C" fn(instance: *mut c_void, ctx: *const FfiOperatorContext) -> OperatorReturn,
    
    /// Handles continuous mouse/keyboard events while running in modal state
    pub modal: extern "C" fn(instance: *mut c_void, ctx: *const FfiOperatorContext) -> OperatorReturn,
    
    /// Cleans up state when the operator is cancelled
    pub cancel: extern "C" fn(instance: *mut c_void, ctx: *const FfiOperatorContext),
}
```

---

## 5. Declarative Layout Tree (`UILayout`) & Property Data-Binding

Instead of immediate coordinate drawing, UI panels define their interface declaratively. The host engine automatically calculates grid alignments, label-control margins, numeric scrubbing physics, and theme styling.

```
                  ┌────────────────────────────────────────┐
                  │              Panel / Header            │
                  └──────────────────┬─────────────────────┘
                                     │ .draw(layout)
                                     ▼
                  ┌────────────────────────────────────────┐
                  │             Root UILayout              │
                  └───────┬──────────────┬───────────┬─────┘
                          │              │           │
                          ▼              ▼           ▼
                   ┌─────────────┐ ┌───────────┐ ┌─────────┐
                   │  row(align) │ │  column() │ │  box()  │
                   └──────┬──────┘ └─────┬─────┘ └────┬────┘
                          │              │            │
                          ▼              ▼            ▼
                   ┌─────────────┐ ┌───────────┐ ┌─────────┐
                   │  prop(zoom) │ │ operator()│ │  label  │
                   └─────────────┘ └───────────┘ └─────────┘
```

### 5.1 Property Data-Binding Types (`FfiPropertyType`)

Properties support two-way data-binding with automatic widget generation and undo integration:

```rust
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FfiPropertyType {
    Boolean = 0,
    Int32 = 1,
    Float32 = 2,
    Enum = 3,
    String = 4,
    ColorRgba = 5,
}

#[repr(C)]
pub struct FfiPropertyDescriptor {
    pub id: FfiSlice,
    pub name: FfiSlice,
    pub description: FfiSlice,
    pub prop_type: FfiPropertyType,
    pub min_val: f32,
    pub max_val: f32,
    pub step: f32,
    pub default_float: f32,
    pub default_int: i32,
    pub default_bool: bool,
}
```

### 5.2 Modular Declarative Layout VTable Contract (`FfiLayoutContext`)

To guarantee strict ABI stability, prevent binary breakage, and simplify foreign language binding generation (C, C++, Rust, Zig), layout operations are grouped into three focused sub-VTables:

```rust
use std::ffi::c_void;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutAlignment {
    Expand = 0,
    Left = 1,
    Center = 2,
    Right = 3,
}

/// Structural container operations (Rows, Columns, Group Boxes, Splits, Grids)
#[repr(C)]
pub struct FfiLayoutContainersVTable {
    /// Creates a horizontal row container. If align=true, child widgets visually attach without gaps.
    pub row: extern "C" fn(layout: *mut c_void, align: bool) -> *mut c_void,
    
    /// Creates a vertical column container.
    pub column: extern "C" fn(layout: *mut c_void, align: bool) -> *mut c_void,
    
    /// Creates a distinct visual grouped box container.
    pub r#box: extern "C" fn(layout: *mut c_void) -> *mut c_void,
    
    /// Splits layout horizontally according to a percentage factor (0.0 to 1.0)
    pub split: extern "C" fn(layout: *mut c_void, factor: f32, align: bool) -> *mut c_void,
    
    /// Creates a multi-column property grid
    pub grid_flow: extern "C" fn(layout: *mut c_void, columns: u32, align: bool) -> *mut c_void,
}

/// Interactive UI controls and property data-binding operations
#[repr(C)]
pub struct FfiLayoutWidgetsVTable {
    /// Adds a descriptive static text label
    pub label: extern "C" fn(layout: *mut c_void, text: FfiSlice, icon_id: u32),
    
    /// Binds an interactive UI widget directly to a registered property ID
    pub prop: extern "C" fn(layout: *mut c_void, prop_owner: *mut c_void, prop_id: FfiSlice),
    
    /// Binds a property as a compact numeric drag/slider control
    pub prop_slider: extern "C" fn(layout: *mut c_void, prop_owner: *mut c_void, prop_id: FfiSlice),
    
    /// Binds an enum property as an expanded horizontal radio button row
    pub prop_enum_row: extern "C" fn(layout: *mut c_void, prop_owner: *mut c_void, prop_id: FfiSlice),
    
    /// Inserts a button that directly invokes or executes an Operator ID
    pub operator: extern "C" fn(layout: *mut c_void, operator_id: FfiSlice, label: FfiSlice, icon_id: u32),
    
    /// Inserts an interactive menu popup anchor
    pub menu: extern "C" fn(layout: *mut c_void, menu_id: FfiSlice, label: FfiSlice),
    
    /// Adds horizontal or vertical whitespace separation
    pub separator: extern "C" fn(layout: *mut c_void),
}

/// Custom 2D interactive vector canvas & texture drawing operations
#[repr(C)]
pub struct FfiLayoutCanvasVTable {
    /// Allocates an interactive custom drawing rectangle
    pub allocate_custom_canvas: extern "C" fn(layout: *mut c_void, width: f32, height: f32, out_rect: *mut [f32; 4]) -> bool,
    pub draw_polyline: extern "C" fn(layout: *mut c_void, points_ptr: *const [f32; 2], points_len: usize, color: FfiColor32, stroke_width: f32),
    pub draw_gpu_texture_view: extern "C" fn(layout: *mut c_void, texture_id: u64, width: f32, height: f32),
}

/// Aggregated layout context provided to panel draw callbacks
#[repr(C)]
pub struct FfiLayoutContext {
    pub layout_handle: *mut c_void,
    pub containers: *const FfiLayoutContainersVTable,
    pub widgets: *const FfiLayoutWidgetsVTable,
    pub canvas: *const FfiLayoutCanvasVTable,
}
```

---

## 6. Panel & Header Extension Descriptors

Extensions contribute UI sections by registering declarative `Panel` and `Header` descriptors:

```rust
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiRegionTarget {
    Header = 0,            // Top / bottom window header
    Toolbar = 1,           // Left-aligned T-Shelf
    Sidebar = 2,           // Right-aligned N-Panel inspector
    StatusBar = 3,         // Bottom status bar
    FloatingPopup = 4,     // Modal or popup dialog
}

#[repr(C)]
pub struct FfiPanelDescriptor {
    pub id: FfiSlice,                  // e.g. "opsis.panel.waveform"
    pub label: FfiSlice,               // e.g. "Color Waveform & Curves"
    pub category: FfiSlice,            // e.g. "Color", "Item", "Metadata" (tab name in N-Panel)
    pub target_region: UiRegionTarget, // Target UI region
    pub default_open: bool,            // Whether panel starts expanded
    
    /// Determines whether the panel should be visible in current context
    pub poll: extern "C" fn(ctx: *const FfiOperatorContext) -> bool,
    
    /// Declaratively constructs the panel layout tree (receives layout and operator context)
    pub draw: extern "C" fn(
        instance_ptr: *mut c_void,
        layout_ctx: *const FfiLayoutContext,
        layout: *mut c_void,
        operator_ctx: *const FfiOperatorContext,
    ),
}
```

---

## 7. Universal Acrylic & Surface Styling

Opsis retains its signature frosted-glass aesthetic while supporting Blender-style regions:

```rust
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackdropMaterial {
    None = 0,         // Fully transparent
    Solid = 1,        // Solid opaque/semi-transparent background color
    Acrylic = 2,      // Dual-Kawase / Frosted-glass backdrop blur with noise/tint
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SurfaceStyle {
    pub material: BackdropMaterial,
    pub tint_color: FfiColor32,
    pub blur_radius: f32,
    pub corner_radius: f32,
    pub border_width: f32,
    pub border_color: FfiColor32,
}
```

* **Region Translucency:** Headers, sidebars, and HUD boxes can render with configurable `BackdropMaterial::Acrylic`, floating elegantly over the canvas and the OS desktop via Windows DWM (`DWMSBT_TRANSIENTWINDOW`) and macOS `NSVisualEffectView`.

---

## 8. High-DPI Scaling & Coordinate Spaces

* **Logical Layout Coordinates:** All `UILayout` elements (row padding, column widths, font metrics) are calculated in logical points.
* **Swapchain Framebuffer Scaling:** Framebuffers and custom GPU viewport textures scale crisply to physical device pixels:
  $$\text{Physical Pixels} = \text{Logical Points} \times \text{DPI Scale Factor}$$
* **Dynamic Multi-Monitor Transitions:** `ScaleFactorChanged` events trigger instant swapchain and UI atlas resizing without dropping active modal operator state.

---

## 9. Keymap Registry & Modal Event Routing

Hotkeys, mouse buttons, and gestures are bound directly to Operator IDs via an editable **Keymap Table**:

```rust
#[repr(C)]
pub struct FfiKeyBinding {
    pub key: FfiSlice,             // e.g. "KeyG", "Space", "F11"
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub operator_id: FfiSlice,     // Target operator (e.g. "opsis.image.rotate_90")
    pub properties_json: FfiSlice, // Initial parameters passed to operator
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub key: String,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub operator_id: String,
    pub properties_json: String,
}

pub trait KeymapManager {
    fn bind_key(&mut self, binding: KeyBinding);
    fn resolve_operator(&self, event: &winit::event::KeyEvent) -> Option<String>;
}
```

---

## 10. Portable Single-Source-of-Truth Configuration & State Persistence

Opsis is designed as a **zero-installation, fully portable application**. Configuration and user preferences follow a **Single Source of Truth** model with a **Sparse Overlay Deserialization** strategy:

```
                            [ Process Invocation ]
                                      │
                                      ▼
             ┌─────────────────────────────────────────────────┐
             │ 1. Check Executable Directory for `opsis.json`  │
             └───────────────┬─────────────────┬───────────────┘
                 (Found)     │                 │ (Not found)
                             ▼                 ▼
             ┌───────────────────────┐ ┌───────────────────────┐
             │ Portable Mode Active  │ │ Standard OS Mode      │
             │ • Read `./opsis.json` │ │ • Read AppData/config │
             └───────────────┬───────┘ └───────┬───────────────┘
                             │                 │
                             └────────┬────────┘
                                      │
                                      ▼
             ┌─────────────────────────────────────────────────┐
             │ 2. Sparse Merge over Compiled Default Struct    │
             │  • Hardcoded Rust defaults provide baseline     │
             │  • Missing / unconfigured keys use defaults     │
             │  • Parse latency < 0.3 ms (serde_json)          │
             └─────────────────────────────────────────────────┘
```

### 10.1 Discovery Precedence (Portable vs. Installed)
1. **Portable Mode (Highest Priority):** On startup, Opsis checks for `opsis.json` or `config.json` in the directory containing the running executable. If present, **Portable Mode** is activated: all settings, plugin folders (`./plugins/`), and transient cache directories remain strictly self-contained within the application folder (ideal for USB drives and zero-registry setups).
2. **User Profile Fallback:** If no local file is found, Opsis checks standard platform config paths (`%APPDATA%/opsis/config.json` on Windows, `~/.config/opsis/config.json` on Linux, `~/Library/Application Support/opsis/config.json` on macOS).
3. **Compiled In-Memory Baseline:** If no configuration file exists in either location, Opsis boots instantly ($< 0.1\text{ ms}$) using hardcoded default constants in memory, creating no disk files until the user explicitly saves a preference.

### 10.2 Single Source of Truth & Sparse JSON Schema (`opsis.json`)

The config struct is modeled in Rust with `#[derive(Serialize, Deserialize)]` and `#[serde(default)]`. The JSON file only needs to store properties the user has explicitly changed:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OpsisConfig {
    pub schema_version: u32,
    pub window: WindowConfig,
    pub viewport: ViewportConfig,
    pub media: MediaConfig,
    pub extensions: ExtensionsConfig,
    pub keymap: HashMap<String, String>, // ActionId -> Shortcut
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowConfig {
    pub remember_geometry: bool,
    pub acrylic_backdrop: bool,
    pub ui_scale: f32,
    pub show_top_bar: bool,
    pub show_status_bar: bool,
    pub default_theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ViewportConfig {
    pub default_filter: String, // "Bicubic", "Lanczos3", "Nearest"
    pub zoom_speed: f32,
    pub smooth_inertia: bool,
    pub hdr_output_mode: String, // "Auto", "SDR", "scRGB_FP16", "HDR10"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MediaConfig {
    pub lru_cache_mb: usize,
    pub natural_sort_alphanumeric: bool,
    pub background_prefetch_adjacent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ExtensionsConfig {
    pub disabled_extensions: Vec<String>,
    pub custom_plugin_paths: Vec<String>,
}
```

### 10.3 Default Fallback Guarantee
* **Zero Migration Overhead:** If an entry is missing, corrupted, or an older `opsis.json` version lacks newly added keys, `serde` automatically fills the missing fields from the hardcoded `Default::default()` implementation without throwing errors or requiring complex migration scripts.
* **Atomic Disk Writing:** Setting changes are flushed to disk using debounced, atomic temporary file swaps (`tempfile` $\to$ rename) to eliminate any risk of config corruption during power loss.

---

## 11. Multi-Window & Session Management

The subsystem supports flexible multi-window workflows:

* **Primary Session Window:** Houses the main viewport, active session state, and global event dispatch.
* **Detachable Inspector Windows:** Sidecar panels (e.g. detailed EXIF viewer, color waveform analyzer, 3D material settings) can be detached into standalone OS windows.
* **Shared GPU Context:** All secondary windows share the primary Vulkan device, pipeline cache, and texture memory, eliminating redundant VRAM allocations.

---

## 12. Rust Public API Contract

```rust
use std::sync::Arc;
use winit::window::WindowId;

pub trait WindowManager {
    /// Processes pending OS events and returns whether the application should exit
    fn pump_events(&mut self) -> bool;

    /// Requests an immediate redraw of the viewport and UI regions
    fn request_redraw(&self, window_id: WindowId);

    /// Registers an Operator descriptor from host or extension
    fn register_operator(&mut self, descriptor: FfiOperatorDescriptor);

    /// Registers a UI Panel descriptor into a target region (Header, Sidebar N-panel, T-shelf)
    fn register_panel(&mut self, descriptor: FfiPanelDescriptor);

    /// Dispatches an operator by ID with execution context
    fn dispatch_operator(&mut self, operator_id: &str, invoke_modal: bool) -> OperatorReturn;

    /// Accesses the global publish/subscribe event bus
    fn event_bus(&self) -> &FfiEventBusVTable;

    /// Displays a non-blocking toast notification for plugin crashes or Safe Mode recovery
    fn show_recovery_toast(&mut self, message: &str, is_warning: bool);
}
```
