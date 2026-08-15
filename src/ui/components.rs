use super::theme::Theme;
use freya::prelude::*;

/// Clean, bordered keyboard shortcut badge (e.g. `Right`, `Space`, `O`).
pub fn key_badge(label_text: impl Into<String>) -> Element {
    rect()
        .padding(Gaps::new(2.0, 7.0, 2.0, 7.0))
        .background(Theme::surface_element())
        .border(Border::new().width(1.0).fill(Theme::border_normal()))
        .corner_radius(Theme::RADIUS_SM)
        .child(
            label()
                .text(label_text.into())
                .font_size(Theme::FONT_CAPTION)
                .font_weight(FontWeight::BOLD)
                .color(Theme::text_primary()),
        )
        .into()
}

/// Compact status or metadata pill badge (e.g. `1920x1080`, `PNG`, `3.2 MB`, `100%`).
pub fn status_pill(label_text: impl Into<String>, is_accent: bool) -> Element {
    rect()
        .padding(Gaps::new(2.0, 8.0, 2.0, 8.0))
        .background(if is_accent {
            Theme::accent_muted()
        } else {
            Theme::surface_element()
        })
        .border(Border::new().width(1.0).fill(if is_accent {
            Theme::accent_primary()
        } else {
            Theme::border_subtle()
        }))
        .corner_radius(Theme::RADIUS_PILL)
        .child(
            label()
                .text(label_text.into())
                .font_size(Theme::FONT_CAPTION)
                .font_weight(FontWeight::BOLD)
                .color(if is_accent {
                    Theme::accent_primary()
                } else {
                    Theme::text_secondary()
                }),
        )
        .into()
}

/// Primary button component.
pub fn button_primary(
    text_content: impl Into<String>,
    on_press_handler: impl FnMut(Event<PressEventData>) + 'static,
) -> Element {
    rect()
        .padding(Gaps::new(5.0, 12.0, 5.0, 12.0))
        .background(Theme::accent_primary())
        .corner_radius(Theme::RADIUS_MD)
        .on_press(on_press_handler)
        .child(
            label()
                .text(text_content.into())
                .font_size(Theme::FONT_BODY)
                .font_weight(FontWeight::BOLD)
                .color(Color::from_rgb(10, 15, 25)),
        )
        .into()
}

/// Secondary button component.
pub fn button_secondary(
    text_content: impl Into<String>,
    on_press_handler: impl FnMut(Event<PressEventData>) + 'static,
) -> Element {
    rect()
        .padding(Gaps::new(5.0, 12.0, 5.0, 12.0))
        .background(Theme::surface_element())
        .border(Border::new().width(1.0).fill(Theme::border_normal()))
        .corner_radius(Theme::RADIUS_MD)
        .on_press(on_press_handler)
        .child(
            label()
                .text(text_content.into())
                .font_size(Theme::FONT_BODY)
                .font_weight(FontWeight::BOLD)
                .color(Theme::text_primary()),
        )
        .into()
}

/// Compact toggle button with active indicator state.
pub fn button_toggle(
    text_content: impl Into<String>,
    is_active: bool,
    on_press_handler: impl FnMut(Event<PressEventData>) + 'static,
) -> Element {
    rect()
        .padding(Gaps::new(4.0, 9.0, 4.0, 9.0))
        .background(if is_active {
            Theme::accent_muted()
        } else {
            Theme::surface_element()
        })
        .border(Border::new().width(1.0).fill(if is_active {
            Theme::accent_primary()
        } else {
            Theme::border_subtle()
        }))
        .corner_radius(Theme::RADIUS_MD)
        .on_press(on_press_handler)
        .child(
            label()
                .text(text_content.into())
                .font_size(Theme::FONT_BODY_SM)
                .font_weight(if is_active {
                    FontWeight::BOLD
                } else {
                    FontWeight::NORMAL
                })
                .color(if is_active {
                    Theme::accent_primary()
                } else {
                    Theme::text_secondary()
                }),
        )
        .into()
}

/// Compact icon / symbol button (e.g. `+`, `-`, `✕`, `☰`).
pub fn button_icon(
    symbol: impl Into<String>,
    on_press_handler: impl FnMut(Event<PressEventData>) + 'static,
) -> Element {
    rect()
        .width(Size::px(26.0))
        .height(Size::px(26.0))
        .main_align(Alignment::Center)
        .cross_align(Alignment::Center)
        .background(Theme::surface_element())
        .border(Border::new().width(1.0).fill(Theme::border_subtle()))
        .corner_radius(Theme::RADIUS_MD)
        .on_press(on_press_handler)
        .child(
            label()
                .text(symbol.into())
                .font_size(Theme::FONT_BODY)
                .font_weight(FontWeight::BOLD)
                .color(Theme::text_primary()),
        )
        .into()
}

/// Section header for sidebars and panels.
pub fn section_header(title: impl Into<String>) -> Element {
    rect()
        .width(Size::fill())
        .padding(Gaps::new(8.0, 12.0, 8.0, 12.0))
        .background(Theme::surface_panel())
        .border(Border::new().width(1.0).fill(Theme::border_subtle()))
        .child(
            label()
                .text(title.into().to_uppercase())
                .font_size(Theme::FONT_CAPTION)
                .font_weight(FontWeight::BOLD)
                .color(Theme::text_muted()),
        )
        .into()
}

/// Key-Value Information row for the N-panel (e.g. `Resolution` -> `3840 x 2160 px`).
pub fn info_row(label_text: impl Into<String>, value_text: impl Into<String>) -> Element {
    rect()
        .width(Size::fill())
        .padding(Gaps::new(5.0, 12.0, 5.0, 12.0))
        .direction(Direction::horizontal())
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center)
        .child(
            label()
                .text(label_text.into())
                .font_size(Theme::FONT_BODY_SM)
                .color(Theme::text_secondary()),
        )
        .child(
            label()
                .text(value_text.into())
                .font_size(Theme::FONT_BODY_SM)
                .font_weight(FontWeight::BOLD)
                .color(Theme::text_primary()),
        )
        .into()
}

/// Standard Header / Toolbar bar container component for extensions and overlays.
pub fn toolbar_container(
    left_items: impl IntoIterator<Item = Element>,
    center_items: impl IntoIterator<Item = Element>,
    right_items: impl IntoIterator<Item = Element>,
) -> Element {
    let left = rect()
        .direction(Direction::horizontal())
        .cross_align(Alignment::Center)
        .spacing(8.0)
        .children(left_items);

    let center = rect()
        .direction(Direction::horizontal())
        .cross_align(Alignment::Center)
        .spacing(6.0)
        .children(center_items);

    let right = rect()
        .direction(Direction::horizontal())
        .cross_align(Alignment::Center)
        .spacing(6.0)
        .children(right_items);

    rect()
        .width(Size::fill())
        .height(Size::px(38.0))
        .background(Theme::surface_panel())
        .border(Border::new().width(1.0).fill(Theme::border_subtle()))
        .padding(Gaps::new(4.0, 12.0, 4.0, 12.0))
        .direction(Direction::horizontal())
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center)
        .child(left)
        .child(center)
        .child(right)
        .into()
}
