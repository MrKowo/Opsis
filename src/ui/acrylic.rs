use freya::engine::prelude::{
    blur, Paint, PaintStyle, RRect, SaveLayerRec, SkRect, TileMode,
};
use freya::prelude::*;

use super::theme::Theme;

/// Configuration options for acrylic backdrop blurring.
#[derive(Debug, Clone, PartialEq)]
pub struct AcrylicConfig {
    /// Gaussian blur radius sigma for the backdrop capture (pixels).
    pub blur_sigma: f32,
    /// Semi-transparent color tint overlay.
    pub tint_color: Color,
    /// Optional subtle border highlight color.
    pub border_color: Option<Color>,
    /// Optional border corner radius.
    pub corner_radius: f32,
}

impl Default for AcrylicConfig {
    fn default() -> Self {
        Self {
            blur_sigma: Theme::ACRYLIC_BLUR_SIGMA,
            tint_color: Theme::acrylic_tint(),
            border_color: Some(Theme::acrylic_border()),
            corner_radius: 0.0,
        }
    }
}

impl AcrylicConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_blur_sigma(mut self, sigma: f32) -> Self {
        self.blur_sigma = sigma;
        self
    }

    pub fn with_tint(mut self, tint: Color) -> Self {
        self.tint_color = tint;
        self
    }

    pub fn with_border(mut self, border: Option<Color>) -> Self {
        self.border_color = border;
        self
    }

    pub fn with_corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }
}

/// Renders a hardware-accelerated Skia acrylic backdrop with Gaussian blur, semi-transparent tint, and border.
pub fn render_acrylic_backdrop(ctx: &mut CanvasContext, config: &AcrylicConfig) {
    let width = ctx.size.width;
    let height = ctx.size.height;
    if width <= 0.0 || height <= 0.0 {
        return;
    }

    let skia_rect = SkRect::new(0.0, 0.0, width, height);

    // 1. Create Gaussian blur image filter
    let blur_filter = if config.blur_sigma > 0.0 {
        blur(
            (config.blur_sigma, config.blur_sigma),
            TileMode::Clamp,
            None,
            None,
        )
    } else {
        None
    };

    // 2. Configure SaveLayerRec with backdrop filter and local bounds
    let mut save_rec = SaveLayerRec::default();
    save_rec = save_rec.bounds(&skia_rect);
    if let Some(ref filter) = blur_filter {
        save_rec = save_rec.backdrop(filter);
    }

    // 3. Save Skia layer to capture and blur background pixels
    ctx.canvas.save_layer(&save_rec);

    // 4. Draw semi-transparent acrylic color tint
    let mut tint_paint = Paint::default();
    tint_paint.set_anti_alias(true);
    tint_paint.set_color(config.tint_color);
    tint_paint.set_style(PaintStyle::Fill);

    if config.corner_radius > 0.0 {
        let rrect = RRect::new_rect_xy(skia_rect, config.corner_radius, config.corner_radius);
        ctx.canvas.draw_rrect(rrect, &tint_paint);

        if let Some(border) = config.border_color {
            let mut border_paint = Paint::default();
            border_paint.set_anti_alias(true);
            border_paint.set_color(border);
            border_paint.set_style(PaintStyle::Stroke);
            border_paint.set_stroke_width(1.0);
            ctx.canvas.draw_rrect(rrect, &border_paint);
        }
    } else {
        ctx.canvas.draw_rect(skia_rect, &tint_paint);

        if let Some(border) = config.border_color {
            let mut border_paint = Paint::default();
            border_paint.set_anti_alias(true);
            border_paint.set_color(border);
            border_paint.set_style(PaintStyle::Stroke);
            border_paint.set_stroke_width(1.0);
            ctx.canvas.draw_rect(skia_rect, &border_paint);
        }
    }

    // 5. Restore Skia layer, compositing blurred backdrop back to target
    ctx.canvas.restore();
}

/// Renders an ambient blurred backdrop of an image scaled to fill the canvas bounds with acrylic tinting.
pub fn render_ambient_blurred_backdrop(
    ctx: &mut CanvasContext,
    image: &freya::engine::prelude::SkImage,
    config: &AcrylicConfig,
) {
    let width = ctx.size.width;
    let height = ctx.size.height;
    if width <= 0.0 || height <= 0.0 {
        return;
    }

    let dst_rect = SkRect::new(0.0, 0.0, width, height);

    // 1. Setup blur paint
    let mut blur_paint = Paint::default();
    blur_paint.set_anti_alias(true);
    if config.blur_sigma > 0.0 {
        if let Some(filter) = blur(
            (config.blur_sigma, config.blur_sigma),
            TileMode::Clamp,
            None,
            None,
        ) {
            blur_paint.set_image_filter(filter);
        }
    }

    // 3. Draw blurred cover image
    ctx.canvas.draw_image_rect(
        image,
        None,
        dst_rect,
        &blur_paint,
    );

    // 4. Draw semi-transparent acrylic color tint on top
    let mut tint_paint = Paint::default();
    tint_paint.set_anti_alias(true);
    tint_paint.set_color(config.tint_color);
    tint_paint.set_style(PaintStyle::Fill);
    ctx.canvas.draw_rect(dst_rect, &tint_paint);
}

/// Global reusable acrylic surface component that renders a hardware-accelerated blurred backdrop
/// behind child elements.
pub fn acrylic_surface(
    config: AcrylicConfig,
    content_width: Size,
    content_height: Size,
    children: impl IntoIterator<Item = Element>,
) -> Element {
    let cfg = config.clone();
    canvas(RenderCallback::new(move |ctx| {
        render_acrylic_backdrop(ctx, &cfg);
    }))
    .width(content_width)
    .height(content_height)
    .direction(Direction::vertical())
    .children(children)
    .into()
}

/// Convenience acrylic panel container with default theme tokens.
pub fn acrylic_panel(
    content_width: Size,
    content_height: Size,
    children: impl IntoIterator<Item = Element>,
) -> Element {
    acrylic_surface(
        AcrylicConfig::default(),
        content_width,
        content_height,
        children,
    )
}

/// Applies native OS acrylic backdrop blur to a Win32 window handle and syncs the title bar caption color.
#[cfg(target_os = "windows")]
pub fn apply_windows_acrylic(hwnd: isize, enabled: bool) {
    use std::ffi::c_void;

    #[repr(C)]
    struct Margins {
        cx_left_width: i32,
        cx_right_width: i32,
        cy_top_height: i32,
        cy_bottom_height: i32,
    }

    type DwmExtendFrameIntoClientAreaFn = unsafe extern "system" fn(
        hwnd: isize,
        p_mar_inset: *const Margins,
    ) -> i32;

    type DwmSetWindowAttributeFn = unsafe extern "system" fn(
        hwnd: isize,
        dw_attribute: u32,
        pv_attribute: *const c_void,
        cb_attribute: u32,
    ) -> i32;

    unsafe {
        if let Ok(lib) = libloading::Library::new("dwmapi.dll") {
            if let Ok(dwm_set_window_attribute) =
                lib.get::<DwmSetWindowAttributeFn>(b"DwmSetWindowAttribute\0")
            {
                // DWMWA_USE_IMMERSIVE_DARK_MODE (20 on Win 11, 19 on older Win 10)
                let dark_mode: u32 = 1;
                let _ = dwm_set_window_attribute(hwnd, 20, &dark_mode as *const _ as *const c_void, 4);

                // Extend DWM frame into client area to flow acrylic backdrop seamlessly across title bar
                if let Ok(dwm_extend_frame) =
                    lib.get::<DwmExtendFrameIntoClientAreaFn>(b"DwmExtendFrameIntoClientArea\0")
                {
                    let margins = if enabled {
                        Margins {
                            cx_left_width: -1,
                            cx_right_width: -1,
                            cy_top_height: -1,
                            cy_bottom_height: -1,
                        }
                    } else {
                        Margins {
                            cx_left_width: 0,
                            cx_right_width: 0,
                            cy_top_height: 0,
                            cy_bottom_height: 0,
                        }
                    };
                    let _ = dwm_extend_frame(hwnd, &margins as *const _);
                }

                // DWMWA_CAPTION_COLOR = 35 (0xFFFFFFFE = DWMWA_COLOR_DEFAULT / inherited backdrop material)
                let caption_color: u32 = if enabled {
                    0xFFFFFFFE
                } else {
                    let c = Theme::surface_base();
                    (c.r() as u32) | ((c.g() as u32) << 8) | ((c.b() as u32) << 16)
                };
                let _ = dwm_set_window_attribute(
                    hwnd,
                    35,
                    &caption_color as *const _ as *const c_void,
                    4,
                );

                // DWMWA_TEXT_COLOR = 36 (COLORREF format: 0x00BBGGRR)
                let text_color: u32 = 0x00FFFFFF;
                let _ = dwm_set_window_attribute(
                    hwnd,
                    36,
                    &text_color as *const _ as *const c_void,
                    4,
                );

                // DWMWA_SYSTEMBACKDROP_TYPE = 38: 3 = Acrylic, 1 = None (solid)
                let backdrop_acrylic: u32 = if enabled { 3 } else { 1 };
                let hr = dwm_set_window_attribute(
                    hwnd,
                    38,
                    &backdrop_acrylic as *const _ as *const c_void,
                    4,
                );

                if hr == 0 {
                    return;
                }
            }
        }
    }

    // 2. Try Windows 10 SetWindowCompositionAttribute (user32.dll)
    if enabled {
        #[repr(C)]
        struct AccentPolicy {
            accent_state: u32,
            accent_flags: u32,
            gradient_color: u32,
            animation_id: u32,
        }

        #[repr(C)]
        struct WindowCompositionAttributeData {
            attribute: u32,
            data: *mut c_void,
            size_of_data: usize,
        }

        type SetWindowCompositionAttributeFn = unsafe extern "system" fn(
            hwnd: isize,
            data: *mut WindowCompositionAttributeData,
        ) -> i32;

        unsafe {
            if let Ok(lib) = libloading::Library::new("user32.dll") {
                if let Ok(set_comp_attr) =
                    lib.get::<SetWindowCompositionAttributeFn>(b"SetWindowCompositionAttribute\0")
                {
                    let (r, g, b) = Theme::ACRYLIC_TINT_RGB;
                    let a = Theme::ACRYLIC_ALPHA;
                    let gradient_color =
                        ((a as u32) << 24) | ((b as u32) << 16) | ((g as u32) << 8) | (r as u32);

                    let mut policy = AccentPolicy {
                        accent_state: 4, // ACCENT_ENABLE_ACRYLICBLURBEHIND
                        accent_flags: 2,
                        gradient_color,
                        animation_id: 0,
                    };
                    let mut data = WindowCompositionAttributeData {
                        attribute: 19, // WCA_ACCENT_POLICY
                        data: &mut policy as *mut _ as *mut c_void,
                        size_of_data: std::mem::size_of::<AccentPolicy>(),
                    };
                    let _ = set_comp_attr(hwnd, &mut data);
                }
            }
        }
    }
}

/// Fallback no-op for non-Windows platforms.
#[cfg(not(target_os = "windows"))]
pub fn apply_windows_acrylic(_hwnd: isize, _enabled: bool) {}

/// Standalone test window view displaying only the acrylic frosted effect with live interactive controls.
pub fn acrylic_test_window_view() -> Element {
    crate::ui::use_init_opsis_theme();

    let mut blur_sigma = use_state(|| 24.0f32);
    let mut tint_alpha = use_state(|| 120u8);

    let cur_sigma = *blur_sigma.read();
    let cur_alpha = *tint_alpha.read();

    let config = AcrylicConfig::default()
        .with_blur_sigma(cur_sigma)
        .with_tint(Color::from_argb(cur_alpha, 20, 24, 32))
        .with_border(Some(Theme::acrylic_border()))
        .with_corner_radius(12.0);

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .background(Color::TRANSPARENT)
        .padding(Gaps::new_all(20.0))
        .main_align(Alignment::Center)
        .cross_align(Alignment::Center)
        .child(
            acrylic_surface(
                config,
                Size::fill(),
                Size::fill(),
                vec![
                    rect()
                        .width(Size::fill())
                        .height(Size::fill())
                        .padding(Gaps::new_all(24.0))
                        .direction(Direction::vertical())
                        .spacing(14.0)
                        .child(
                            label()
                                .text("✨ Acrylic Blur Effect Test")
                                .font_size(Theme::FONT_TITLE)
                                .font_weight(FontWeight::BOLD)
                                .color(Theme::text_primary()),
                        )
                        .child(
                            label()
                                .text("This floating test window renders a frosted acrylic surface over what is behind it.")
                                .font_size(Theme::FONT_BODY_SM)
                                .color(Theme::text_secondary()),
                        )
                        .child(crate::ui::section_header("Live Controls"))
                        .child(
                            rect()
                                .direction(Direction::horizontal())
                                .spacing(8.0)
                                .cross_align(Alignment::Center)
                                .child(crate::ui::status_pill(format!("Blur: {cur_sigma:.0}px"), true))
                                .child(crate::ui::status_pill(format!("Tint Alpha: {cur_alpha}"), true)),
                        )
                        .child(
                            rect()
                                .direction(Direction::horizontal())
                                .spacing(8.0)
                                .child(crate::ui::button_primary("Increase Blur", move |_| {
                                    blur_sigma.with_mut(|mut s| *s = (*s + 6.0).min(60.0));
                                }))
                                .child(crate::ui::button_secondary("Decrease Blur", move |_| {
                                    blur_sigma.with_mut(|mut s| *s = (*s - 6.0).max(0.0));
                                }))
                                .child(crate::ui::button_secondary("Toggle Opacity", move |_| {
                                    tint_alpha.with_mut(|mut a| *a = if *a > 100 { 60 } else { 160 });
                                })),
                        )
                        .into(),
                ],
            ),
        )
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acrylic_config_and_render() {
        let config = AcrylicConfig::default()
            .with_blur_sigma(20.0)
            .with_tint(Color::from_argb(180, 20, 20, 25))
            .with_border(Some(Color::from_argb(50, 255, 255, 255)))
            .with_corner_radius(8.0);

        assert_eq!(config.blur_sigma, 20.0);
        assert_eq!(config.tint_color, Color::from_argb(180, 20, 20, 25));
        assert_eq!(config.border_color, Some(Color::from_argb(50, 255, 255, 255)));
        assert_eq!(config.corner_radius, 8.0);

        let elem = acrylic_panel(
            Size::px(140.0),
            Size::fill(),
            vec![label().text("Test Item").into()],
        );
        let _ = elem;
    }
}
