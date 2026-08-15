use freya::prelude::Color;

/// Centralized Theme Token Engine for Opsis.
pub struct Theme;

impl Theme {
    // --- Surface Elevations ---
    /// Deepest base window/canvas background (RGB: 18, 18, 20 / #121214)
    pub const fn surface_base() -> Color {
        Color::from_rgb(18, 18, 20)
    }

    /// Header, footer, and docked sidebar panels (RGB: 24, 24, 28 / #18181c)
    pub const fn surface_panel() -> Color {
        Color::from_rgb(24, 24, 28)
    }

    /// Elevated cards, popovers, and table headers (RGB: 32, 32, 38 / #202026)
    pub const fn surface_card() -> Color {
        Color::from_rgb(32, 32, 38)
    }

    /// Interactive element idle state (buttons, inputs) (RGB: 40, 40, 48 / #282830)
    pub const fn surface_element() -> Color {
        Color::from_rgb(40, 40, 48)
    }

    /// Interactive element hover state (RGB: 52, 52, 64 / #343440)
    pub const fn surface_element_hover() -> Color {
        Color::from_rgb(52, 52, 64)
    }

    /// Interactive element active/pressed state (RGB: 64, 64, 80 / #404050)
    pub const fn surface_element_active() -> Color {
        Color::from_rgb(64, 64, 80)
    }

    // --- Accent Colors ---
    /// Vibrant primary brand/selection accent (Sky / Cyan Blue: RGB: 56, 189, 248 / #38bdf8)
    pub const fn accent_primary() -> Color {
        Color::from_rgb(56, 189, 248)
    }

    /// Primary accent hover (RGB: 96, 165, 250 / #60a5fa)
    pub const fn accent_primary_hover() -> Color {
        Color::from_rgb(96, 165, 250)
    }

    /// Muted accent container background (RGB: 30, 41, 59 / #1e293b)
    pub const fn accent_muted() -> Color {
        Color::from_rgb(30, 41, 59)
    }

    /// Warm accent for custom bindings and warnings (Amber: RGB: 245, 158, 11 / #f59e0b)
    pub const fn accent_warm() -> Color {
        Color::from_rgb(245, 158, 11)
    }

    /// Warm accent container background (RGB: 45, 35, 18 / #2d2312)
    pub const fn accent_warm_bg() -> Color {
        Color::from_rgb(45, 35, 18)
    }

    // --- Text Hierarchy ---
    /// Primary high-contrast text (RGB: 248, 250, 252 / #f8fafc)
    pub const fn text_primary() -> Color {
        Color::from_rgb(248, 250, 252)
    }

    /// Secondary descriptions, table values (RGB: 148, 163, 184 / #94a3b8)
    pub const fn text_secondary() -> Color {
        Color::from_rgb(148, 163, 184)
    }

    /// Muted hints, shortcuts, watermarks (RGB: 100, 116, 139 / #64748b)
    pub const fn text_muted() -> Color {
        Color::from_rgb(100, 116, 139)
    }

    // --- Borders & Dividers ---
    /// Subtle 1px dividers and panel separators (RGB: 39, 39, 44 / #27272c)
    pub const fn border_subtle() -> Color {
        Color::from_rgb(39, 39, 44)
    }

    /// Widget boundaries and cards (RGB: 55, 55, 65 / #373741)
    pub const fn border_normal() -> Color {
        Color::from_rgb(55, 55, 65)
    }

    /// Focus rings and active recording border (RGB: 56, 189, 248 / #38bdf8)
    pub const fn border_focus() -> Color {
        Color::from_rgb(56, 189, 248)
    }

    // --- Status Colors ---
    pub const fn status_success() -> Color {
        Color::from_rgb(34, 197, 94)
    }

    pub const fn status_danger() -> Color {
        Color::from_rgb(239, 68, 68)
    }

    // --- Layout & Typography Metrics ---
    pub const RADIUS_SM: f32 = 3.0;
    pub const RADIUS_MD: f32 = 4.0;
    pub const RADIUS_LG: f32 = 6.0;
    pub const RADIUS_PILL: f32 = 999.0;

    pub const FONT_CAPTION: f32 = 10.0;
    pub const FONT_BODY_SM: f32 = 11.0;
    pub const FONT_BODY: f32 = 12.0;
    pub const FONT_SUBTITLE: f32 = 14.0;
    pub const FONT_TITLE: f32 = 16.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_colors_and_metrics() {
        assert_eq!(Theme::surface_base(), Color::from_rgb(18, 18, 20));
        assert_eq!(Theme::surface_panel(), Color::from_rgb(24, 24, 28));
        assert_eq!(Theme::accent_primary(), Color::from_rgb(56, 189, 248));
        assert!(Theme::RADIUS_SM < Theme::RADIUS_MD);
        assert!(Theme::RADIUS_MD < Theme::RADIUS_LG);
        assert!(Theme::FONT_CAPTION < Theme::FONT_BODY);
        assert!(Theme::FONT_BODY < Theme::FONT_TITLE);
    }
}
