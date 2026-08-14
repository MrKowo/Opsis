use bytes::Bytes;
use freya::elements::image::{image, ImageHandle};
use freya::engine::prelude::{AlphaType, Data, SkImage};
use freya::prelude::*;

use crate::file_io::{load_image, LoadedImage};

const BASE_LOGO: &[u8] = include_bytes!("../assets/branding/logo.png");

/// Core 2D Canvas viewport state.
#[derive(Debug, Clone)]
pub struct CanvasState {
    pub image: Option<LoadedImage>,
    pub zoom: f32,
    pub pan_offset: (f32, f32),
    pub is_dragging: bool,
    pub drag_start: (f64, f64),
    pub error_message: Option<String>,
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
        }
    }
}

impl CanvasState {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate default zoom: Auto-fit if image exceeds window, 1.0 (100%) otherwise.
    pub fn calculate_initial_zoom(img_w: u32, img_h: u32, win_w: f64, win_h: f64) -> f32 {
        if img_w == 0 || img_h == 0 {
            return 1.0;
        }

        let available_w = (win_w * 0.95).max(100.0) as f32;
        let available_h = (win_h * 0.95).max(100.0) as f32;

        let img_w_f = img_w as f32;
        let img_h_f = img_h as f32;

        if img_w_f > available_w || img_h_f > available_h {
            let scale_x = available_w / img_w_f;
            let scale_y = available_h / img_h_f;
            scale_x.min(scale_y).clamp(0.05, 1.0)
        } else {
            1.0
        }
    }

    /// Set a newly loaded image and initialize viewport zoom and pan.
    pub fn set_image(&mut self, image: LoadedImage, window_size: (f64, f64)) {
        let (w, h) = image.metadata.dimensions;
        let initial_zoom = Self::calculate_initial_zoom(w, h, window_size.0, window_size.1);
        self.image = Some(image);
        self.zoom = initial_zoom;
        self.pan_offset = (0.0, 0.0);
        self.is_dragging = false;
        self.error_message = None;
    }

    /// Clear the current image and return to the default empty base window.
    pub fn clear_image(&mut self) {
        self.image = None;
        self.zoom = 1.0;
        self.pan_offset = (0.0, 0.0);
        self.is_dragging = false;
        self.error_message = None;
    }

    /// Set an error message when loading an image fails.
    pub fn set_error(&mut self, err: String) {
        self.error_message = Some(err);
    }

    /// Zoom in by 25%.
    pub fn zoom_in(&mut self) {
        self.zoom = (self.zoom * 1.25).clamp(0.02, 50.0);
    }

    /// Zoom out by 25%.
    pub fn zoom_out(&mut self) {
        self.zoom = (self.zoom / 1.25).clamp(0.02, 50.0);
    }

    /// Reset zoom to 100% and center pan offset.
    pub fn reset_zoom(&mut self) {
        self.zoom = 1.0;
        self.pan_offset = (0.0, 0.0);
    }

    /// Fit the current image to the given window dimensions.
    pub fn fit_to_window(&mut self, window_size: (f64, f64)) {
        if let Some(ref img) = self.image {
            let (w, h) = img.metadata.dimensions;
            if w > 0 && h > 0 {
                let available_w = (window_size.0 * 0.95).max(100.0) as f32;
                let available_h = (window_size.1 * 0.95).max(100.0) as f32;
                let scale_x = available_w / w as f32;
                let scale_y = available_h / h as f32;
                self.zoom = scale_x.min(scale_y).clamp(0.02, 50.0);
                self.pan_offset = (0.0, 0.0);
            }
        }
    }

    /// Apply incremental zoom factor.
    pub fn apply_zoom_delta(&mut self, delta_factor: f32) {
        self.zoom = (self.zoom * delta_factor).clamp(0.02, 50.0);
    }

    /// Pan by delta (dx, dy).
    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.pan_offset.0 += dx;
        self.pan_offset.1 += dy;
    }
}

/// Render the core 2D canvas viewport.
pub fn canvas_view(mut state: State<CanvasState>, window_size: (f64, f64)) -> Element {
    let current_state = state.read().clone();

    if let Some(ref image_data) = current_state.image {
        let (img_w, img_h) = image_data.metadata.dimensions;
        let zoom = current_state.zoom;
        let rendered_w = (img_w as f32 * zoom).max(1.0);
        let rendered_h = (img_h as f32 * zoom).max(1.0);
        let (pan_x, pan_y) = current_state.pan_offset;

        let bytes = Bytes::copy_from_slice(&image_data.bytes);
        let data = unsafe { Data::new_bytes(&bytes) };

        let rendered_image: Element = if let Some(sk_img) = SkImage::from_encoded(data) {
            image(ImageHandle::new(sk_img, bytes))
                .width(Size::fill())
                .height(Size::fill())
                .into()
        } else if let Ok(dyn_img) = image::load_from_memory(&image_data.bytes) {
            let rgba = dyn_img.to_rgba8();
            let (w, h) = rgba.dimensions();
            let rgba_bytes = Bytes::from(rgba.into_raw());
            if let Some(handle) = ImageHandle::from_rgba(w, h, rgba_bytes, AlphaType::Unpremul) {
                image(handle)
                    .width(Size::fill())
                    .height(Size::fill())
                    .into()
            } else {
                rect().into()
            }
        } else {
            rect().into()
        };

        rect()
            .width(Size::fill())
            .height(Size::fill())
            .overflow(Overflow::Clip)
            .main_align(Alignment::Center)
            .cross_align(Alignment::Center)
            .on_wheel(move |e: Event<WheelEventData>| {
                let delta = e.delta_y;
                let factor = if delta < 0.0 { 1.15 } else { 1.0 / 1.15 };
                state.with_mut(|mut st| st.apply_zoom_delta(factor));
            })
            .on_mouse_down(move |e: Event<MouseEventData>| {
                state.with_mut(|mut st| {
                    st.is_dragging = true;
                    st.drag_start = (e.global_location.x, e.global_location.y);
                });
            })
            .on_mouse_move(move |e: Event<MouseEventData>| {
                state.with_mut(|mut st| {
                    if st.is_dragging {
                        let dx = (e.global_location.x - st.drag_start.0) as f32;
                        let dy = (e.global_location.y - st.drag_start.1) as f32;
                        st.pan(dx, dy);
                        st.drag_start = (e.global_location.x, e.global_location.y);
                    }
                });
            })
            .on_mouse_up(move |_| {
                state.with_mut(|mut st| st.is_dragging = false);
            })
            .on_file_drop(move |e: Event<FileEventData>| {
                if let Some(ref path) = e.file_path {
                    match load_image(path) {
                        Ok(img) => state.with_mut(|mut st| st.set_image(img, window_size)),
                        Err(err) => state.with_mut(|mut st| st.set_error(err)),
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
    } else if let Some(ref err) = current_state.error_message {
        // Error presentation card
        rect()
            .width(Size::fill())
            .height(Size::fill())
            .main_align(Alignment::Center)
            .cross_align(Alignment::Center)
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
                    ]),
            )
            .into()
    } else {
        // Base watermark view when no image is loaded
        let logo_bytes = Bytes::from_static(BASE_LOGO);
        let logo_data = unsafe { Data::new_bytes(&logo_bytes) };
        let logo_element: Element = if let Some(sk_img) = SkImage::from_encoded(logo_data) {
            image(ImageHandle::new(sk_img, logo_bytes))
                .width(Size::px(180.0))
                .height(Size::px(180.0))
                .opacity(0.15)
                .into()
        } else {
            rect().into()
        };

        rect()
            .width(Size::fill())
            .height(Size::fill())
            .main_align(Alignment::Center)
            .cross_align(Alignment::Center)
            .on_file_drop(move |e: Event<FileEventData>| {
                if let Some(ref path) = e.file_path {
                    match load_image(path) {
                        Ok(img) => state.with_mut(|mut st| st.set_image(img, window_size)),
                        Err(err) => state.with_mut(|mut st| st.set_error(err)),
                    }
                }
            })
            .child(
                rect()
                    .direction(Direction::vertical())
                    .cross_align(Alignment::Center)
                    .spacing(14.0)
                    .children([
                        logo_element,
                        label()
                            .text("Press S to open settings")
                            .font_size(12.0)
                            .color(Color::from_argb(45, 255, 255, 255))
                            .into(),
                    ]),
            )
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_zoom_small_image() {
        let zoom = CanvasState::calculate_initial_zoom(400, 300, 800.0, 600.0);
        assert_eq!(zoom, 1.0);
    }

    #[test]
    fn test_initial_zoom_large_image() {
        let zoom = CanvasState::calculate_initial_zoom(3840, 2160, 800.0, 600.0);
        assert!(zoom < 1.0);
        assert!(zoom > 0.0);
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
}
