use bytes::Bytes;
use freya::elements::image::{image, ImageHandle};
use freya::engine::prelude::{AlphaType, Data, SkImage};
use freya::prelude::*;

use crate::file_io::{load_image, LoadedImage};
use crate::ui::Theme;

const BASE_LOGO: &[u8] = include_bytes!("../assets/logo.png");

use std::path::PathBuf;

/// Core 2D Canvas viewport state.
#[derive(Debug, Clone)]
pub struct CanvasState {
    pub image: Option<LoadedImage>,
    pub zoom: f32,
    pub pan_offset: (f32, f32),
    pub is_dragging: bool,
    pub drag_start: (f64, f64),
    pub error_message: Option<String>,
    pub last_file_path: Option<PathBuf>,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            image: None,
            zoom: 1.0,
            pan_offset: (0.0, 0.0),
            is_dragging: false,
            drag_start: (0.0, 0.0),
            error_message: None,
            last_file_path: None,
        }
    }
}

impl CanvasState {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieve the currently active or last attempted image path for folder navigation.
    pub fn active_path(&self) -> Option<PathBuf> {
        self.image
            .as_ref()
            .map(|img| img.metadata.path.clone())
            .or_else(|| self.last_file_path.clone())
    }

    /// Calculate default zoom: Auto-fit image to window dimensions.
    pub fn calculate_initial_zoom(img_w: u32, img_h: u32, win_w: f64, win_h: f64) -> f32 {
        if img_w == 0 || img_h == 0 || win_w <= 0.0 || win_h <= 0.0 {
            return 1.0;
        }

        let scale_x = win_w as f32 / img_w as f32;
        let scale_y = win_h as f32 / img_h as f32;
        scale_x.min(scale_y).clamp(0.001, 100.0)
    }

    /// Compute the target window dimensions to preserve image aspect ratio within 75% of screen size.
    #[allow(dead_code)]
    pub fn calculate_target_window_size(
        img_dims: (u32, u32),
        screen_dims: (f64, f64),
    ) -> (f64, f64) {
        calculate_target_window_size(img_dims, screen_dims)
    }

    /// Set a newly loaded image and initialize viewport zoom and pan.
    pub fn set_image(&mut self, image: LoadedImage, window_size: (f64, f64)) {
        let (w, h) = image.metadata.dimensions;
        let initial_zoom = Self::calculate_initial_zoom(w, h, window_size.0, window_size.1);
        self.last_file_path = Some(image.metadata.path.clone());
        self.image = Some(image);
        self.zoom = initial_zoom;
        self.pan_offset = (0.0, 0.0);
        self.is_dragging = false;
        self.error_message = None;
    }

    /// Clear the current image and return to the default empty base window.
    pub fn clear_image(&mut self) {
        crate::log_canvas!("Cleared active image");
        self.image = None;
        self.zoom = 1.0;
        self.pan_offset = (0.0, 0.0);
        self.is_dragging = false;
        self.error_message = None;
        self.last_file_path = None;
    }

    /// Set an error message when loading an image fails.
    pub fn set_error(&mut self, err: String) {
        crate::log_canvas!("Canvas error: {err}");
        self.image = None;
        self.error_message = Some(err);
    }

    /// Set an error message associated with a failed file path (preserving path for folder cycling).
    pub fn set_error_for_path(&mut self, err: String, path: PathBuf) {
        crate::log_canvas!("Canvas error for '{}': {err}", path.display());
        self.image = None;
        self.last_file_path = Some(path);
        self.error_message = Some(err);
    }

    /// Zoom in by 25%.
    pub fn zoom_in(&mut self) {
        self.zoom = (self.zoom * 1.25).clamp(0.02, 50.0);
        crate::log_canvas!("Zoom in -> {:.0}%", self.zoom * 100.0);
    }

    /// Zoom out by 25%.
    pub fn zoom_out(&mut self) {
        self.zoom = (self.zoom / 1.25).clamp(0.02, 50.0);
        crate::log_canvas!("Zoom out -> {:.0}%", self.zoom * 100.0);
    }

    /// Reset zoom to 100% and center pan offset.
    pub fn reset_zoom(&mut self) {
        self.zoom = 1.0;
        self.pan_offset = (0.0, 0.0);
        crate::log_canvas!("Zoom reset to 100% (1:1 scale)");
    }

    /// Fit the current image to the given window dimensions.
    #[allow(dead_code)]
    pub fn fit_to_window(&mut self, window_size: (f64, f64)) {
        if let Some(ref img) = self.image {
            let (w, h) = img.metadata.dimensions;
            if w > 0 && h > 0 {
                self.zoom = Self::calculate_initial_zoom(w, h, window_size.0, window_size.1);
                self.pan_offset = (0.0, 0.0);
            }
        }
    }

    /// Fit the image horizontally to fill the viewport width (touching left/right borders).
    #[allow(dead_code)]
    pub fn fit_horizontal(&mut self, window_size: (f64, f64)) {
        if let Some(ref img) = self.image {
            let (w, _h) = img.metadata.dimensions;
            if w > 0 {
                self.zoom = (window_size.0 as f32 / w as f32).clamp(0.001, 100.0);
                self.pan_offset = (0.0, 0.0);
            }
        }
    }

    /// Fit the image vertically to fill the viewport height (touching top/bottom borders).
    #[allow(dead_code)]
    pub fn fit_vertical(&mut self, window_size: (f64, f64)) {
        if let Some(ref img) = self.image {
            let (_w, h) = img.metadata.dimensions;
            if h > 0 {
                self.zoom = (window_size.1 as f32 / h as f32).clamp(0.001, 100.0);
                self.pan_offset = (0.0, 0.0);
            }
        }
    }

    /// Toggle between horizontal fit (touching left/right edges) and vertical fit (touching top/bottom edges).
    pub fn toggle_fit_axis(&mut self, window_size: (f64, f64)) {
        if let Some(ref img) = self.image {
            let (w, h) = img.metadata.dimensions;
            if w > 0 && h > 0 {
                let scale_h = (window_size.0 as f32 / w as f32).clamp(0.001, 100.0);
                let scale_v = (window_size.1 as f32 / h as f32).clamp(0.001, 100.0);

                // If currently closer to horizontal fit, switch to vertical; otherwise switch to horizontal
                if (self.zoom - scale_h).abs() < (self.zoom - scale_v).abs() {
                    self.zoom = scale_v;
                } else {
                    self.zoom = scale_h;
                }
                self.pan_offset = (0.0, 0.0);
            }
        }
    }

    /// Apply incremental zoom factor.
    pub fn apply_zoom_delta(&mut self, delta_factor: f32) {
        self.zoom = (self.zoom * delta_factor).clamp(0.02, 50.0);
    }

    /// Apply incremental zoom factor centered at a specific cursor coordinate relative to the viewport size.
    pub fn zoom_at(&mut self, factor: f32, cursor: (f32, f32), viewport: (f32, f32)) {
        if viewport.0 <= 0.0 || viewport.1 <= 0.0 {
            self.apply_zoom_delta(factor);
            return;
        }

        let old_zoom = self.zoom;
        let new_zoom = (old_zoom * factor).clamp(0.02, 50.0);
        if (new_zoom - old_zoom).abs() < f32::EPSILON {
            return;
        }

        let k = new_zoom / old_zoom;
        let dx = cursor.0 - viewport.0 / 2.0;
        let dy = cursor.1 - viewport.1 / 2.0;

        self.pan_offset.0 = dx - k * (dx - self.pan_offset.0);
        self.pan_offset.1 = dy - k * (dy - self.pan_offset.1);
        self.zoom = new_zoom;
    }

    /// Pan by delta (dx, dy).
    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.pan_offset.0 += dx;
        self.pan_offset.1 += dy;
    }
}

/// Compute the target window dimensions to preserve image aspect ratio within 75% of screen size.
pub fn calculate_target_window_size(
    img_dims: (u32, u32),
    screen_dims: (f64, f64),
) -> (f64, f64) {
    let (w, h) = (img_dims.0 as f64, img_dims.1 as f64);
    if w <= 0.0 || h <= 0.0 {
        return (800.0, 600.0);
    }

    let max_w = (screen_dims.0 * 0.75).max(400.0);
    let max_h = (screen_dims.1 * 0.75).max(300.0);

    let (target_w, target_h) = if w <= max_w && h <= max_h {
        (w, h)
    } else {
        let scale = (max_w / w).min(max_h / h);
        (w * scale, h * scale)
    };

    (target_w.max(400.0), target_h.max(300.0))
}

use crate::manager::ExtensionManager;
use std::sync::{Arc, Mutex};

/// Render the core 2D canvas viewport with post-processing extension filters applied.
pub fn canvas_view(
    mut state: State<CanvasState>,
    window_size: (f64, f64),
    ext_mgr: Option<&Arc<Mutex<ExtensionManager>>>,
) -> Element {
    let mut canvas_size = use_state(|| (window_size.0 as f32, window_size.1 as f32));
    let current_state = state.read().clone();

    let (acrylic_enabled, show_watermark) = if let Some(mgr_arc) = ext_mgr {
        if let Ok(manager) = mgr_arc.lock() {
            (
                manager.settings.acrylic_background,
                manager.settings.show_watermark,
            )
        } else {
            (false, true)
        }
    } else {
        (false, true)
    };

    if let Some(ref image_data) = current_state.image {
        let (img_w, img_h) = image_data.metadata.dimensions;
        let zoom = current_state.zoom;
        let rendered_w = (img_w as f32 * zoom).max(1.0);
        let rendered_h = (img_h as f32 * zoom).max(1.0);
        let (pan_x, pan_y) = current_state.pan_offset;

        // Apply active post-processing filters from registered extensions
        let filtered_bytes = if let Some(mgr_arc) = ext_mgr {
            if let Ok(manager) = mgr_arc.lock() {
                if manager.registry.has_image_filters() {
                    if let Some(rgba) = image_data.get_rgba_or_decode() {
                        manager.apply_image_filters(&rgba, img_w, img_h)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let rendered_image_opt: Option<Element> = if let Some(filtered) = filtered_bytes {
            ImageHandle::from_rgba(img_w, img_h, filtered, AlphaType::Unpremul)
                .map(|handle| image(handle).width(Size::fill()).height(Size::fill()).into())
        } else {
            let data = Data::new_copy(&image_data.bytes);

            if let Some(sk_img) = SkImage::from_encoded(data) {
                let bytes = image_data.bytes.clone();
                Some(image(ImageHandle::new(sk_img, bytes)).width(Size::fill()).height(Size::fill()).into())
            } else if let Some(rgba) = image_data.get_rgba_or_decode() {
                ImageHandle::from_rgba(img_w, img_h, rgba, AlphaType::Unpremul)
                    .map(|handle| image(handle).width(Size::fill()).height(Size::fill()).into())
            } else {
                None
            }
        };

        if let Some(rendered_image) = rendered_image_opt {
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .overflow(Overflow::Clip)
                .main_align(Alignment::Center)
                .cross_align(Alignment::Center)
                .background(Theme::canvas_background(acrylic_enabled))
                .on_sized(move |e: Event<SizedEventData>| {
                    let w = e.area.size.width;
                    let h = e.area.size.height;
                    if w > 0.0 && h > 0.0 {
                        canvas_size.set((w, h));
                    }
                })
                .on_wheel(move |e: Event<WheelEventData>| {
                    let delta = e.delta_y;
                    if delta == 0.0 {
                        return;
                    }
                    let factor = if delta > 0.0 { 1.15 } else { 1.0 / 1.15 };
                    let cursor = (e.element_location.x as f32, e.element_location.y as f32);
                    let (canvas_w, canvas_h) = *canvas_size.read();
                    let viewport = if canvas_w > 0.0 && canvas_h > 0.0 {
                        (canvas_w, canvas_h)
                    } else {
                        (window_size.0 as f32, window_size.1 as f32)
                    };
                    state.with_mut(|mut st| {
                        st.zoom_at(factor, cursor, viewport);
                        crate::log_input!(
                            "Mouse wheel scroll: delta={delta:.1} cursor=({:.0}, {:.0}) -> Zoom: {:.0}%",
                            cursor.0,
                            cursor.1,
                            st.zoom * 100.0
                        );
                    });
                })
                .on_mouse_down(move |e: Event<MouseEventData>| {
                    state.with_mut(|mut st| {
                        st.is_dragging = true;
                        st.drag_start = (e.global_location.x, e.global_location.y);
                    });
                })
                .on_global_pointer_move(move |e: Event<PointerEventData>| {
                    state.with_mut(|mut st| {
                        if st.is_dragging {
                            let loc = e.global_location();
                            let dx = (loc.x - st.drag_start.0) as f32;
                            let dy = (loc.y - st.drag_start.1) as f32;
                            st.pan(dx, dy);
                            st.drag_start = (loc.x, loc.y);
                        }
                    });
                })
                .on_mouse_up(move |_| {
                    state.with_mut(|mut st| st.is_dragging = false);
                })
                .on_global_pointer_press(move |_| {
                    state.with_mut(|mut st| st.is_dragging = false);
                })
                .on_capture_global_pointer_press(move |_| {
                    state.with_mut(|mut st| st.is_dragging = false);
                })
                .on_file_drop(move |e: Event<FileEventData>| {
                    if let Some(ref path) = e.file_path {
                        crate::log_input!("File dropped onto window canvas: '{}'", path.display());
                        match load_image(path) {
                            Ok(img) => {
                                crate::window::resize_window_to_image_aspect(img.metadata.dimensions);
                                state.with_mut(|mut st| st.set_image(img, window_size));
                            }
                            Err(err) => state.with_mut(|mut st| st.set_error_for_path(err, path.clone())),
                        }
                    }
                })
                .child(
                    rect()
                        .width(Size::px(rendered_w))
                        .height(Size::px(rendered_h))
                        .offset_x(pan_x)
                        .offset_y(pan_y)
                        .child(rendered_image),
                )
                .into()
        } else {
            // Corrupted payload presentation card
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .main_align(Alignment::Center)
                .cross_align(Alignment::Center)
                .on_file_drop(move |e: Event<FileEventData>| {
                    if let Some(ref path) = e.file_path {
                        crate::log_input!("File dropped onto window canvas: '{}'", path.display());
                        match load_image(path) {
                            Ok(img) => {
                                crate::window::resize_window_to_image_aspect(img.metadata.dimensions);
                                state.with_mut(|mut st| st.set_image(img, window_size));
                            }
                            Err(err) => state.with_mut(|mut st| st.set_error_for_path(err, path.clone())),
                        }
                    }
                })
                .child(
                    rect()
                        .background(Color::from_rgb(32, 20, 22))
                        .border(Border::new().width(1.0).fill(Color::from_rgb(90, 35, 40)))
                        .corner_radius(10.0)
                        .padding(Gaps::new_all(20.0))
                        .direction(Direction::vertical())
                        .spacing(12.0)
                        .cross_align(Alignment::Center)
                        .children([
                            label()
                                .text("Corrupted Image File")
                                .font_size(15.0)
                                .font_weight(FontWeight::BOLD)
                                .color(Color::from_rgb(255, 120, 130))
                                .into(),
                            label()
                                .text(format!("Unable to decode raster image data for '{}'", image_data.metadata.filename))
                                .font_size(12.0)
                                .color(Color::from_rgb(200, 180, 180))
                                .into(),
                            label()
                                .text("Use Left / Right arrow keys to cycle, or press O to open another image")
                                .font_size(11.0)
                                .color(Color::from_rgb(150, 130, 130))
                                .into(),
                        ]),
                )
                .into()
        }
    } else if let Some(ref err) = current_state.error_message {
        // Error presentation card
        rect()
            .width(Size::fill())
            .height(Size::fill())
            .main_align(Alignment::Center)
            .cross_align(Alignment::Center)
            .on_file_drop(move |e: Event<FileEventData>| {
                if let Some(ref path) = e.file_path {
                    crate::log_input!("File dropped onto window canvas: '{}'", path.display());
                    match load_image(path) {
                        Ok(img) => {
                            crate::window::resize_window_to_image_aspect(img.metadata.dimensions);
                            state.with_mut(|mut st| st.set_image(img, window_size));
                        }
                        Err(err) => state.with_mut(|mut st| st.set_error_for_path(err, path.clone())),
                    }
                }
            })
            .child(
                rect()
                    .background(Color::from_rgb(32, 20, 22))
                    .border(Border::new().width(1.0).fill(Color::from_rgb(90, 35, 40)))
                    .corner_radius(10.0)
                    .padding(Gaps::new_all(20.0))
                    .direction(Direction::vertical())
                    .spacing(12.0)
                    .cross_align(Alignment::Center)
                    .children([
                        label()
                            .text("Failed to load image")
                            .font_size(15.0)
                            .font_weight(FontWeight::BOLD)
                            .color(Color::from_rgb(255, 120, 130))
                            .into(),
                        label()
                            .text(err.clone())
                            .font_size(12.0)
                            .color(Color::from_rgb(200, 180, 180))
                            .into(),
                        label()
                            .text("Use Left / Right arrow keys to cycle, or press O to open another image")
                            .font_size(11.0)
                            .color(Color::from_rgb(150, 130, 130))
                            .into(),
                    ]),
            )
            .into()
    } else {
        // Base watermark view when no image is loaded
        let logo_bytes = Bytes::from_static(BASE_LOGO);
        let logo_data = Data::new_copy(BASE_LOGO);
        let logo_element: Element = if let Some(sk_img) = SkImage::from_encoded(logo_data) {
            image(ImageHandle::new(sk_img, logo_bytes))
                .width(Size::px(180.0))
                .height(Size::px(180.0))
                .opacity(0.15)
                .into()
        } else {
            rect().into()
        };

        let watermark_child: Element = if show_watermark {
            logo_element
        } else {
            rect().into()
        };

        rect()
            .width(Size::fill())
            .height(Size::fill())
            .main_align(Alignment::Center)
            .cross_align(Alignment::Center)
            .background(Theme::canvas_background(acrylic_enabled))
            .on_file_drop(move |e: Event<FileEventData>| {
                if let Some(ref path) = e.file_path {
                    match load_image(path) {
                        Ok(img) => {
                            crate::window::resize_window_to_image_aspect(img.metadata.dimensions);
                            state.with_mut(|mut st| st.set_image(img, window_size));
                        }
                        Err(err) => state.with_mut(|mut st| st.set_error_for_path(err, path.clone())),
                    }
                }
            })
            .child(
                rect()
                    .direction(Direction::vertical())
                    .cross_align(Alignment::Center)
                    .spacing(14.0)
                    .child(watermark_child)
                    .child(
                        rect()
                            .direction(Direction::vertical())
                            .cross_align(Alignment::Center)
                            .spacing(6.0)
                            .child(
                                label()
                                    .text("Press O to open an image")
                                    .font_size(12.0)
                                    .color(Color::from_argb(50, 255, 255, 255)),
                            )
                            .child(
                                label()
                                    .text("Press S to open settings")
                                    .font_size(12.0)
                                    .color(Color::from_argb(35, 255, 255, 255)),
                            ),
                    ),
            )
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_io::ImageMetadata;

    #[test]
    fn test_initial_zoom_small_image() {
        let zoom = CanvasState::calculate_initial_zoom(400, 300, 800.0, 600.0);
        assert_eq!(zoom, 2.0);
    }

    #[test]
    fn test_initial_zoom_large_image() {
        let zoom = CanvasState::calculate_initial_zoom(3840, 2160, 800.0, 600.0);
        assert!((zoom - 800.0 / 3840.0).abs() < 0.001);
    }

    #[test]
    fn test_zoom_bounds() {
        let mut state = CanvasState::new();
        state.zoom = 1.0;
        state.zoom_in();
        assert_eq!(state.zoom, 1.25);
        state.zoom_out();
        assert_eq!(state.zoom, 1.0);
        state.reset_zoom();
        assert_eq!(state.zoom, 1.0);
    }

    #[test]
    fn test_fit_and_toggle_axis() {
        use crate::file_io::{ImageMetadata, LoadedImage};
        use bytes::Bytes;
        use std::path::PathBuf;

        let mut state = CanvasState::new();
        state.image = Some(LoadedImage {
            bytes: Bytes::new(),
            rgba_cache: std::sync::Arc::new(std::sync::OnceLock::new()),
            metadata: ImageMetadata {
                path: PathBuf::from("test.png"),
                filename: "test.png".to_string(),
                format_name: "PNG".to_string(),
                dimensions: (1000, 500),
                file_size_bytes: 1024,
            },
        });

        // Window 1000x1000: horizontal fit zoom = 1.0, vertical fit zoom = 2.0
        state.fit_horizontal((1000.0, 1000.0));
        assert!((state.zoom - 1.0).abs() < 0.01);

        state.fit_vertical((1000.0, 1000.0));
        assert!((state.zoom - 2.0).abs() < 0.01);

        // Toggle from vertical should switch to horizontal
        state.toggle_fit_axis((1000.0, 1000.0));
        assert!((state.zoom - 1.0).abs() < 0.01);

        // Toggle from horizontal should switch to vertical
        state.toggle_fit_axis((1000.0, 1000.0));
        assert!((state.zoom - 2.0).abs() < 0.01);

        state.fit_to_window((1000.0, 1000.0));
        assert!(state.zoom > 0.0);
    }

    #[test]
    fn test_generate_and_verify_app_icon() {
        let logo_bytes = include_bytes!("../assets/logo.png");
        let img = image::load_from_memory(logo_bytes).expect("Failed to load logo.png");
        let (orig_w, orig_h) = (img.width(), img.height());
        println!("Original logo dimensions: {}x{}", orig_w, orig_h);

        // Resize cleanly with Lanczos3 to standard 256x256 app icon
        let icon_256 = img.resize_exact(256, 256, image::imageops::FilterType::Lanczos3);
        let icon_path = std::path::Path::new("assets/icon.png");
        icon_256.save(icon_path).expect("Failed to save icon.png");

        let ico_path = std::path::Path::new("assets/icon.ico");
        icon_256.save(ico_path).expect("Failed to save icon.ico");

        assert!(icon_path.exists());
        assert!(ico_path.exists());
    }

    #[test]
    fn test_active_path_and_error_navigation() {
        let mut state = CanvasState::new();
        assert_eq!(state.active_path(), None);

        let path = PathBuf::from("my_image.png");
        state.set_image(
            LoadedImage {
                bytes: Bytes::new(),
                rgba_cache: std::sync::Arc::new(std::sync::OnceLock::new()),
                metadata: ImageMetadata {
                    path: path.clone(),
                    filename: "my_image.png".to_string(),
                    format_name: "PNG".to_string(),
                    dimensions: (100, 100),
                    file_size_bytes: 100,
                },
            },
            (800.0, 600.0),
        );
        assert_eq!(state.active_path(), Some(path.clone()));

        // When navigating to a corrupted image, path is preserved in last_file_path
        let corrupt_path = PathBuf::from("corrupted.jpg");
        state.set_error_for_path("Bad jpeg".to_string(), corrupt_path.clone());
        assert_eq!(state.image.is_some(), false);
        assert_eq!(state.error_message, Some("Bad jpeg".to_string()));
        assert_eq!(state.active_path(), Some(corrupt_path));

        // When explicitly clearing, path is cleared
        state.clear_image();
        assert_eq!(state.active_path(), None);
    }

    #[test]
    fn test_cursor_centered_zoom() {
        let mut state = CanvasState::new();
        state.zoom = 1.0;
        state.pan_offset = (0.0, 0.0);

        // Zooming at viewport center (400, 300) on 800x600 viewport keeps pan_offset at (0, 0)
        state.zoom_at(2.0, (400.0, 300.0), (800.0, 600.0));
        assert_eq!(state.zoom, 2.0);
        assert_eq!(state.pan_offset, (0.0, 0.0));

        // Reset
        state.zoom = 1.0;
        state.pan_offset = (0.0, 0.0);

        // Zooming at cursor offset (600, 300) [dx = +200] with 2x zoom:
        // P_x1 = 200 - 2.0 * (200 - 0) = -200
        state.zoom_at(2.0, (600.0, 300.0), (800.0, 600.0));
        assert_eq!(state.zoom, 2.0);
        assert!((state.pan_offset.0 - (-200.0)).abs() < 0.001);
        assert!((state.pan_offset.1 - 0.0).abs() < 0.001);

        // Zooming out with 0.5x at same cursor position (600, 300) returns pan_offset to 0
        state.zoom_at(0.5, (600.0, 300.0), (800.0, 600.0));
        assert_eq!(state.zoom, 1.0);
        assert!((state.pan_offset.0 - 0.0).abs() < 0.001);
        assert!((state.pan_offset.1 - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_pan_delta() {
        let mut state = CanvasState::new();
        state.pan(15.0, -25.0);
        assert_eq!(state.pan_offset, (15.0, -25.0));
        state.pan(-10.0, 5.0);
        assert_eq!(state.pan_offset, (5.0, -20.0));
    }

    #[test]
    fn test_calculate_target_window_size() {
        let screen = (1920.0, 1080.0); // 75% bounds = (1440.0, 810.0)

        // 1. Small image within 75% bounds (800x600) stays 800x600
        let (w, h) = calculate_target_window_size((800, 600), screen);
        assert_eq!((w, h), (800.0, 600.0));

        // 2. Large landscape 4K image (3840x2160) scales to 1440x810 preserving 16:9 aspect ratio
        let (w, h) = calculate_target_window_size((3840, 2160), screen);
        assert!((w - 1440.0).abs() < 0.001);
        assert!((h - 810.0).abs() < 0.001);
        assert!((w / h - 16.0 / 9.0).abs() < 0.001);

        // 3. Tall portrait image (1000x3000) scales to fit max_h (810) -> (270, 810), clamped to min_w 400 -> (400, 810)
        let (w, h) = calculate_target_window_size((1000, 3000), screen);
        assert!((w - 400.0).abs() < 0.001);
        assert!((h - 810.0).abs() < 0.001);

        // 4. Tiny image (50x50) clamps to minimum bounds (400x300)
        let (w, h) = calculate_target_window_size((50, 50), screen);
        assert_eq!((w, h), (400.0, 300.0));

        // 5. Zero/empty dimensions fallback to default 800x600
        let (w, h) = calculate_target_window_size((0, 0), screen);
        assert_eq!((w, h), (800.0, 600.0));
    }
}
