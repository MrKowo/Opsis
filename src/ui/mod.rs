pub mod acrylic;
pub mod components;
pub mod helpers;
pub mod theme;

pub use acrylic::*;
pub use components::*;
pub use helpers::*;
pub use theme::Theme;

use freya::prelude::*;

/// Initializes the global Opsis design system theme with unified, non-highlighting scrollbars
/// in the current Freya window context.
pub fn use_init_opsis_theme() {
    use_init_theme(Theme::create_freya_theme);
}

