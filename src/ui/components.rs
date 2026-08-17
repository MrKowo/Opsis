use super::helpers::*;
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
    button(text_content, ButtonVariant::Primary, on_press_handler)
}

/// Secondary button component.
pub fn button_secondary(
    text_content: impl Into<String>,
    on_press_handler: impl FnMut(Event<PressEventData>) + 'static,
) -> Element {
    button(text_content, ButtonVariant::Secondary, on_press_handler)
}

/// Versatile button component supporting multiple variants (Primary, Secondary, Danger) and click handling.
pub fn button(
    label_text: impl Into<String>,
    variant: ButtonVariant,
    on_press_handler: impl FnMut(Event<PressEventData>) + 'static,
) -> Element {
    let (bg_color, border_color, text_color) = match variant {
        ButtonVariant::Primary => (
            Theme::accent_primary(),
            Theme::accent_primary(),
            Color::from_rgb(10, 15, 25),
        ),
        ButtonVariant::Secondary => (
            Theme::surface_element(),
            Theme::border_normal(),
            Theme::text_primary(),
        ),
        ButtonVariant::Danger => (
            Theme::status_danger(),
            Theme::status_danger(),
            Color::from_rgb(255, 255, 255),
        ),
    };

    rect()
        .padding(Gaps::new(5.0, 12.0, 5.0, 12.0))
        .background(bg_color)
        .border(Border::new().width(1.0).fill(border_color))
        .corner_radius(Theme::RADIUS_MD)
        .on_press(on_press_handler)
        .child(
            label()
                .text(label_text.into())
                .font_size(Theme::FONT_BODY)
                .font_weight(FontWeight::BOLD)
                .color(text_color),
        )
        .into()
}

/// Key-Value interactive button row (e.g. `Reset Defaults` -> [Reset]).
pub fn button_row(
    label_text: impl Into<String>,
    button_label: impl Into<String>,
    on_press_handler: impl FnMut(Event<PressEventData>) + 'static,
) -> Element {
    rect()
        .width(Size::fill())
        .padding(Gaps::new(6.0, 12.0, 6.0, 12.0))
        .direction(Direction::horizontal())
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center)
        .child(
            label()
                .text(label_text.into())
                .font_size(Theme::FONT_BODY_SM)
                .color(Theme::text_primary()),
        )
        .child(button_secondary(button_label, on_press_handler))
        .into()
}

/// Interactive pill switch toggle (e.g. ON/OFF toggle switch).
pub fn switch_toggle(
    is_active: bool,
    on_press_handler: impl FnMut(Event<PressEventData>) + 'static,
) -> Element {
    let thumb = rect()
        .width(Size::px(14.0))
        .height(Size::px(14.0))
        .background(if is_active {
            Color::from_rgb(10, 15, 25)
        } else {
            Theme::text_secondary()
        })
        .corner_radius(Theme::RADIUS_PILL);

    rect()
        .width(Size::px(36.0))
        .height(Size::px(20.0))
        .padding(Gaps::new_all(3.0))
        .background(if is_active {
            Theme::accent_primary()
        } else {
            Theme::surface_element()
        })
        .border(Border::new().width(1.0).fill(if is_active {
            Theme::accent_primary()
        } else {
            Theme::border_normal()
        }))
        .corner_radius(Theme::RADIUS_PILL)
        .direction(Direction::horizontal())
        .main_align(if is_active {
            Alignment::End
        } else {
            Alignment::Start
        })
        .cross_align(Alignment::Center)
        .on_press(on_press_handler)
        .child(thumb)
        .into()
}

/// Key-Value interactive toggle switch row.
pub fn switch_row(
    label_text: impl Into<String>,
    is_active: bool,
    on_press_handler: impl FnMut(Event<PressEventData>) + 'static,
) -> Element {
    rect()
        .width(Size::fill())
        .padding(Gaps::new(6.0, 12.0, 6.0, 12.0))
        .direction(Direction::horizontal())
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center)
        .child(
            label()
                .text(label_text.into())
                .font_size(Theme::FONT_BODY_SM)
                .color(Theme::text_primary()),
        )
        .child(switch_toggle(is_active, on_press_handler))
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

/// Reusable section component with text on top of a full-width dividing line.
pub fn section(title: impl Into<String>) -> Element {
    section_header(title)
}

/// Reusable section header component with text on top of a full-width dividing line.
pub fn section_header(title: impl Into<String>) -> Element {
    rect()
        .width(Size::fill())
        .direction(Direction::vertical())
        .padding(Gaps::new(10.0, 0.0, 4.0, 0.0))
        .spacing(6.0)
        .child(
            label()
                .text(title.into())
                .font_size(Theme::FONT_BODY_SM)
                .font_weight(FontWeight::BOLD)
                .color(Theme::text_primary()),
        )
        .child(
            rect()
                .width(Size::fill())
                .height(Size::px(1.0))
                .background(Theme::border_subtle()),
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

/// Reusable 1px horizontal line divider spanning full width.
pub fn divider() -> Element {
    rect()
        .width(Size::fill())
        .height(Size::px(1.0))
        .background(Theme::border_subtle())
        .into()
}

/// Reusable 1px vertical line divider spanning full height.
pub fn vertical_divider() -> Element {
    rect()
        .width(Size::px(1.0))
        .height(Size::fill())
        .background(Theme::border_subtle())
        .into()
}

/// Standardized top title bar with title, optional subtitle description, and optional right-aligned action or badge.
pub fn pane_header(
    title: impl Into<String>,
    subtitle: Option<impl Into<String>>,
    right_action: Option<Element>,
) -> Element {
    let mut root = rect()
        .width(Size::fill())
        .direction(Direction::vertical())
        .spacing(4.0);

    let mut top_bar = rect()
        .width(Size::fill())
        .direction(Direction::horizontal())
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center)
        .child(
            label()
                .text(title.into())
                .font_size(Theme::FONT_TITLE)
                .font_weight(FontWeight::BOLD)
                .color(Theme::text_primary()),
        );

    if let Some(action) = right_action {
        top_bar = top_bar.child(action);
    }

    root = root.child(top_bar);

    if let Some(sub) = subtitle {
        root = root.child(
            label()
                .text(sub.into())
                .font_size(Theme::FONT_BODY_SM)
                .color(Theme::text_secondary()),
        );
    }

    root.into()
}

/// Elevated grouping card container with rounded corners and subtle border.
pub fn card(children: impl IntoIterator<Item = Element>) -> Element {
    rect()
        .width(Size::fill())
        .background(Theme::surface_panel())
        .border(Border::new().width(1.0).fill(Theme::border_subtle()))
        .corner_radius(Theme::RADIUS_MD)
        .padding(Gaps::new_all(8.0))
        .direction(Direction::vertical())
        .spacing(4.0)
        .children(children)
        .into()
}

/// Setting row inside a card with left and right items.
pub fn card_row(left: Element, right: Element) -> Element {
    rect()
        .width(Size::fill())
        .direction(Direction::horizontal())
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center)
        .padding(Gaps::new(4.0, 8.0, 4.0, 8.0))
        .child(left)
        .child(right)
        .into()
}

/// Collapsible accordion card for list items with chevron, title, badge, and expandable body.
pub fn expandable_card(
    title: impl Into<String>,
    badge_text: Option<impl Into<String>>,
    is_expanded: bool,
    on_toggle: impl FnMut(Event<PressEventData>) + 'static,
    details_children: impl IntoIterator<Item = Element>,
) -> Element {
    let title_str = title.into();
    let chevron_symbol = if is_expanded { "▼" } else { "▶" };

    let mut header = rect()
        .width(Size::fill())
        .padding(Gaps::new(6.0, 10.0, 6.0, 10.0))
        .direction(Direction::horizontal())
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center)
        .child(
            rect()
                .direction(Direction::horizontal())
                .spacing(8.0)
                .cross_align(Alignment::Center)
                .child(button_icon(chevron_symbol, on_toggle))
                .child(
                    label()
                        .text(title_str)
                        .font_size(Theme::FONT_BODY)
                        .font_weight(FontWeight::BOLD)
                        .color(Theme::text_primary()),
                ),
        );

    if let Some(badge) = badge_text {
        header = header.child(status_pill(badge.into(), false));
    }

    let mut container = rect()
        .width(Size::fill())
        .direction(Direction::vertical())
        .background(Theme::surface_panel())
        .border(Border::new().width(1.0).fill(Theme::border_subtle()))
        .corner_radius(Theme::RADIUS_MD)
        .child(header);

    if is_expanded {
        container = container.child(
            rect()
                .width(Size::fill())
                .padding(Gaps::new(4.0, 8.0, 8.0, 8.0))
                .background(Theme::surface_base())
                .direction(Direction::vertical())
                .spacing(2.0)
                .children(details_children),
        );
    }

    container.into()
}

/// Interactive drag-and-drop installer zone with path caption and drop handler.
pub fn file_dropzone(
    prompt_text: impl Into<String>,
    caption_text: impl Into<String>,
    on_drop_handler: impl FnMut(Event<FileEventData>) + 'static,
) -> Element {
    rect()
        .width(Size::fill())
        .padding(Gaps::new(8.0, 12.0, 8.0, 12.0))
        .background(Theme::surface_panel())
        .border(Border::new().width(1.0).fill(Theme::border_subtle()))
        .corner_radius(Theme::RADIUS_MD)
        .direction(Direction::horizontal())
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center)
        .on_file_drop(on_drop_handler)
        .child(
            label()
                .text(prompt_text.into())
                .font_size(Theme::FONT_BODY_SM)
                .color(Theme::text_secondary()),
        )
        .child(
            label()
                .text(caption_text.into())
                .font_size(Theme::FONT_CAPTION)
                .color(Theme::text_muted()),
        )
        .into()
}

/// Centered placeholder widget when a list or search result is empty.
pub fn empty_state(
    symbol: impl Into<String>,
    title_text: impl Into<String>,
    description_text: Option<impl Into<String>>,
) -> Element {
    let mut root = rect()
        .width(Size::fill())
        .padding(Gaps::new_all(24.0))
        .direction(Direction::vertical())
        .main_align(Alignment::Center)
        .cross_align(Alignment::Center)
        .spacing(6.0)
        .child(
            label()
                .text(symbol.into())
                .font_size(Theme::FONT_TITLE)
                .color(Theme::text_muted()),
        )
        .child(
            label()
                .text(title_text.into())
                .font_size(Theme::FONT_BODY)
                .font_weight(FontWeight::BOLD)
                .color(Theme::text_secondary()),
        );

    if let Some(desc) = description_text {
        root = root.child(
            label()
                .text(desc.into())
                .font_size(Theme::FONT_CAPTION)
                .color(Theme::text_muted()),
        );
    }

    root.into()
}

/// Generalized text field component for string inputs (search, naming, file paths, etc.).
pub fn text_field(
    value: impl Into<String>,
    placeholder_text: impl Into<String>,
    width: Size,
) -> Element {
    let val_str = value.into();
    let placeholder = placeholder_text.into();
    let is_empty = val_str.is_empty();

    rect()
        .width(width)
        .padding(Gaps::new(5.0, 10.0, 5.0, 10.0))
        .background(Theme::surface_element())
        .border(Border::new().width(1.0).fill(Theme::border_normal()))
        .corner_radius(Theme::RADIUS_MD)
        .direction(Direction::horizontal())
        .cross_align(Alignment::Center)
        .spacing(6.0)
        .child(
            label()
                .text(if is_empty { placeholder } else { val_str })
                .font_size(Theme::FONT_BODY_SM)
                .color(if is_empty {
                    Theme::text_muted()
                } else {
                    Theme::text_primary()
                }),
        )
        .into()
}

/// Text field with an accompanying label to the left.
pub fn text_field_row(
    label_text: impl Into<String>,
    value: impl Into<String>,
    placeholder_text: impl Into<String>,
) -> Element {
    rect()
        .width(Size::fill())
        .direction(Direction::horizontal())
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center)
        .padding(Gaps::new(6.0, 12.0, 6.0, 12.0))
        .child(
            label()
                .text(label_text.into())
                .font_size(Theme::FONT_BODY_SM)
                .color(Theme::text_primary()),
        )
        .child(text_field(value, placeholder_text, Size::px(220.0)))
        .into()
}

/// Generalized table component props.
pub struct TableProps {
    pub column_widths: Vec<Size>,
    pub header: Option<Element>,
    pub rows: Vec<Element>,
    pub show_borders: bool,
}

/// Generalized tabular layout aligning elements into rows and columns with optional borders and header.
pub fn table(props: TableProps) -> Element {
    let mut container = rect()
        .width(Size::fill())
        .height(Size::fill())
        .direction(Direction::vertical());

    if props.show_borders {
        container = container
            .border(Border::new().width(1.0).fill(Theme::border_subtle()))
            .corner_radius(Theme::RADIUS_MD);
    }

    if let Some(header_elem) = props.header {
        container = container.child(header_elem);
    }

    container = container.child(
        ScrollView::new()
            .width(Size::fill())
            .height(Size::fill())
            .child(
                rect()
                    .width(Size::fill())
                    .direction(Direction::vertical())
                    .children(props.rows),
            ),
    );

    container.into()
}

/// Table column header bar with customizable column titles and widths.
pub fn table_header(columns: Vec<impl Into<String>>, column_widths: &[Size]) -> Element {
    let cells: Vec<Element> = columns
        .into_iter()
        .enumerate()
        .map(|(idx, col_title)| {
            let width = column_widths
                .get(idx)
                .cloned()
                .unwrap_or(Size::flex(1.0));
            let is_last = idx == column_widths.len().saturating_sub(1);

            let mut lbl = label()
                .text(col_title.into())
                .font_size(Theme::FONT_BODY_SM)
                .font_weight(FontWeight::BOLD)
                .color(Theme::text_secondary());

            if is_last {
                lbl = lbl.width(Size::fill()).text_align(TextAlign::Right);
            }

            rect()
                .width(width)
                .child(lbl)
                .into()
        })
        .collect();

    rect()
        .width(Size::fill())
        .background(Theme::surface_panel())
        .padding(Gaps::new(8.0, 16.0, 8.0, 16.0))
        .direction(Direction::horizontal())
        .border(Border::new().width(1.0).fill(Theme::border_subtle()))
        .children(cells)
        .into()
}

/// Single table row element with configurable column widths, optional alternating background color, and on_press.
pub fn table_row(
    cells: Vec<Element>,
    column_widths: &[Size],
    is_even: bool,
    on_press_handler: Option<impl FnMut(Event<PressEventData>) + 'static>,
) -> Element {
    let row_cells: Vec<Element> = cells
        .into_iter()
        .enumerate()
        .map(|(idx, cell)| {
            let width = column_widths
                .get(idx)
                .cloned()
                .unwrap_or(Size::flex(1.0));
            rect()
                .width(width)
                .child(cell)
                .into()
        })
        .collect();

    let bg = if is_even {
        Theme::surface_base()
    } else {
        Theme::surface_panel()
    };

    let mut r = rect()
        .width(Size::fill())
        .background(bg)
        .padding(Gaps::new(6.0, 16.0, 6.0, 16.0))
        .direction(Direction::horizontal())
        .cross_align(Alignment::Center)
        .children(row_cells);

    if let Some(handler) = on_press_handler {
        r = r.on_press(handler);
    }

    r.into()
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

/// Single clickable item row inside a dropdown menu matching Fluent UI styling with left accent indicator and hover highlight.
pub fn dropdown_item(
    label_text: impl Into<String>,
    is_selected: bool,
    is_hovered: bool,
    on_press_handler: impl FnMut(Event<PressEventData>) + 'static,
    on_pointer_enter_handler: impl FnMut(Event<PointerEventData>) + 'static,
    on_pointer_leave_handler: impl FnMut(Event<PointerEventData>) + 'static,
) -> Element {
    let on_press_rc = std::rc::Rc::new(std::cell::RefCell::new(on_press_handler));

    let left_indicator = rect()
        .width(Size::px(3.0))
        .height(Size::px(12.0))
        .background(if is_selected {
            Theme::accent_primary()
        } else {
            Color::TRANSPARENT
        })
        .corner_radius(Theme::RADIUS_PILL);

    let left_section = rect()
        .direction(Direction::horizontal())
        .spacing(6.0)
        .cross_align(Alignment::Center)
        .child(left_indicator)
        .child(
            label()
                .text(label_text.into())
                .font_size(Theme::FONT_BODY_SM)
                .font_weight(if is_selected {
                    FontWeight::BOLD
                } else {
                    FontWeight::NORMAL
                })
                .color(if is_selected {
                    Theme::accent_primary()
                } else {
                    Theme::text_primary()
                }),
        );

    let bg = if is_selected {
        Theme::accent_muted()
    } else if is_hovered {
        Theme::surface_element()
    } else {
        Color::TRANSPARENT
    };

    rect()
        .width(Size::fill())
        .height(Size::px(DROPDOWN_ITEM_ROW_HEIGHT))
        .padding(Gaps::new(2.0, 8.0, 2.0, 8.0))
        .background(bg)
        .corner_radius(Theme::RADIUS_SM)
        .on_press(move |e: Event<PressEventData>| {
            e.stop_propagation();
            (on_press_rc.borrow_mut())(e);
        })
        .on_pointer_enter(on_pointer_enter_handler)
        .on_pointer_leave(on_pointer_leave_handler)
        .direction(Direction::horizontal())
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center)
        .child(left_section)
        .maybe_child(if is_selected {
            Some(
                label()
                    .text("✓")
                    .font_size(Theme::FONT_BODY_SM)
                    .font_weight(FontWeight::BOLD)
                    .color(Theme::accent_primary()),
            )
        } else {
            None
        })
        .into()
}

/// Interactive scrollable container with a clean, static-thickness, brighter-on-hover draggable scrollbar.
fn dropdown_scrollable_list(
    content_height: f32,
    total_content_height: f32,
    initial_scroll_y: i32,
    scroll_controller: Option<ScrollController>,
    scrollbar_state: Option<ScrollbarState>,
    list_content: impl IntoElement,
) -> Element {
    let scroll_view = if let Some(ctrl) = scroll_controller {
        ScrollView::new_controlled(ctrl)
    } else {
        ScrollView::new()
    };

    let cur_scroll_y = if let Some(ctrl) = scroll_controller {
        let (_, y): (i32, i32) = ctrl.into();
        y
    } else {
        initial_scroll_y
    };

    let thumb_height = ((content_height / total_content_height) * content_height).max(20.0);
    let max_scroll = (total_content_height - content_height).max(1.0);
    let max_thumb_travel = (content_height - thumb_height).max(1.0);
    let thumb_top =
        ((-cur_scroll_y as f32) / max_scroll * max_thumb_travel).clamp(0.0, max_thumb_travel);

    let is_dragging = scrollbar_state
        .and_then(|s| s.drag)
        .map(|d| d.peek().is_some())
        .unwrap_or(false);
    let is_hovered = scrollbar_state
        .and_then(|s| s.hover)
        .map(|h| *h.peek())
        .unwrap_or(false);

    let thumb_bg = if is_dragging {
        Theme::scrollbar_thumb_active()
    } else if is_hovered {
        Theme::scrollbar_thumb_hover()
    } else {
        Theme::scrollbar_thumb()
    };

    let on_pointer_enter = move |_| {
        if let Some(mut h) = scrollbar_state.and_then(|s| s.hover) {
            h.set(true);
        }
    };

    let on_pointer_leave = move |_| {
        if let Some(mut h) = scrollbar_state.and_then(|s| s.hover) {
            h.set(false);
        }
    };

    let on_track_down = move |e: Event<PointerEventData>| {
        if e.data().is_primary() {
            e.stop_propagation();
            let click_element_y = e.element_location().y as f32;
            let click_global_y = e.global_location().y as f32;

            let thumb_bottom = thumb_top + thumb_height;
            let target_thumb_top =
                if click_element_y >= thumb_top && click_element_y <= thumb_bottom {
                    thumb_top
                } else {
                    (click_element_y - thumb_height / 2.0).clamp(0.0, max_thumb_travel)
                };

            let next_scroll = -((target_thumb_top / max_thumb_travel) * max_scroll) as i32;
            if let Some(mut d) = scrollbar_state.and_then(|s| s.drag) {
                d.set(Some((click_global_y, next_scroll)));
            }
            if let Some(mut ctrl) = scroll_controller {
                ctrl.scroll_to_y(next_scroll);
            }
        }
    };

    let on_global_drag = move |e: Event<PointerEventData>| {
        if let Some(d) = scrollbar_state.and_then(|s| s.drag) {
            let active_drag = { *d.peek() };
            if let Some((start_global_y, start_scroll)) = active_drag {
                let current_global_y = e.global_location().y as f32;
                let delta_global_y = current_global_y - start_global_y;
                let scroll_delta = (delta_global_y / max_thumb_travel) * max_scroll;
                let next_scroll =
                    (start_scroll as f32 - scroll_delta).clamp(-max_scroll, 0.0) as i32;

                if let Some(mut ctrl) = scroll_controller {
                    ctrl.scroll_to_y(next_scroll);
                }
            }
        }
    };

    let on_release = move |_: Event<PointerEventData>| {
        if let Some(mut d) = scrollbar_state.and_then(|s| s.drag) {
            let is_dragging = { d.peek().is_some() };
            if is_dragging {
                d.set(None);
            }
        }
        if let Some(mut h) = scrollbar_state.and_then(|s| s.hover) {
            let is_hovered = { *h.peek() };
            if is_hovered {
                h.set(false);
            }
        }
    };

    let on_wheel = move |e: Event<WheelEventData>| {
        let delta = e.delta_y as f32;
        let cur = if let Some(ctrl) = scroll_controller {
            let (_, y): (i32, i32) = ctrl.into();
            y
        } else {
            initial_scroll_y
        };
        let next = (cur as f32 + delta).clamp(-max_scroll, 0.0) as i32;
        if let Some(mut ctrl) = scroll_controller {
            ctrl.scroll_to_y(next);
        }
    };

    let thumb = rect()
        .position(Position::new_absolute().top(thumb_top).right(2.0))
        .width(Size::px(Theme::SCROLLBAR_SIZE))
        .height(Size::px(thumb_height))
        .background(thumb_bg)
        .corner_radius(Theme::RADIUS_PILL);

    let scrollbar_track = rect()
        .position(Position::new_absolute().top(0.0).right(0.0))
        .layer(999)
        .width(Size::px(12.0))
        .height(Size::px(content_height))
        .background(Color::TRANSPARENT)
        .on_pointer_enter(on_pointer_enter)
        .on_pointer_leave(on_pointer_leave)
        .on_pointer_down(on_track_down)
        .on_pointer_press(on_release)
        .child(thumb);

    rect()
        .width(Size::fill())
        .height(Size::px(content_height))
        .on_wheel(on_wheel)
        .on_capture_global_pointer_move(on_global_drag)
        .on_capture_global_pointer_press(on_release)
        .on_pointer_press(on_release)
        .child(
            scroll_view
                .width(Size::fill())
                .height(Size::px(content_height))
                .show_scrollbar(false)
                .child(list_content),
        )
        .child(scrollbar_track)
        .into()
}

/// Generic dropdown menu container component with a trigger header and an elevated, floating collapsible item list.
pub fn dropdown_menu(
    trigger_label: impl Into<String>,
    is_open: bool,
    selected_idx: Option<usize>,
    scroll_controller: Option<ScrollController>,
    scrollbar_state: Option<ScrollbarState>,
    on_toggle: impl FnMut(Event<PressEventData>) + 'static,
    menu_items: impl IntoIterator<Item = Element>,
) -> Element {
    let on_toggle_rc = std::rc::Rc::new(std::cell::RefCell::new(on_toggle));

    let header = rect()
        .width(Size::fill())
        .height(Size::px(32.0))
        .padding(Gaps::new(4.0, 10.0, 4.0, 10.0))
        .background(if is_open {
            Theme::surface_card()
        } else {
            Theme::surface_element()
        })
        .border(Border::new().width(1.0).fill(Theme::border_normal()))
        .corner_radius(Theme::RADIUS_MD)
        .direction(Direction::horizontal())
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center)
        .on_press(move |e: Event<PressEventData>| {
            e.stop_propagation();
            (on_toggle_rc.borrow_mut())(e);
        })
        .child(
            label()
                .text(trigger_label.into())
                .font_size(Theme::FONT_BODY_SM)
                .font_weight(FontWeight::BOLD)
                .color(Theme::text_primary()),
        )
        .child(
            label()
                .text(if is_open { "▴" } else { "▾" })
                .font_size(Theme::FONT_CAPTION)
                .color(if is_open {
                    Theme::accent_primary()
                } else {
                    Theme::text_secondary()
                }),
        );

    let items_vec: Vec<Element> = menu_items.into_iter().collect();
    let count = items_vec.len();

    let popup = if is_open {
        let visible_count = count.min(MAX_DROPDOWN_VISIBLE_ITEMS);
        let content_height = (visible_count as f32 * DROPDOWN_ITEM_ROW_HEIGHT)
            + (visible_count.saturating_sub(1) as f32 * 2.0);

        let metrics = DropdownLayoutMetrics::compute(count, selected_idx);

        let list_content = rect()
            .width(Size::fill())
            .direction(Direction::vertical())
            .spacing(2.0)
            .padding(if count > MAX_DROPDOWN_VISIBLE_ITEMS {
                Gaps::new(0.0, 8.0, 0.0, 0.0)
            } else {
                Gaps::new_all(0.0)
            })
            .children(items_vec);

        let list_body: Element = if count > MAX_DROPDOWN_VISIBLE_ITEMS {
            let total_content_height = (count as f32 * DROPDOWN_ITEM_ROW_HEIGHT)
                + (count.saturating_sub(1) as f32 * 2.0);

            dropdown_scrollable_list(
                content_height,
                total_content_height,
                metrics.initial_scroll_y,
                scroll_controller,
                scrollbar_state,
                list_content,
            )
        } else {
            list_content.into()
        };

        Some(
            rect()
                .position(Position::new_absolute().top(metrics.top_offset).left(0.0))
                .layer(50)
                .width(Size::fill())
                .padding(Gaps::new_all(4.0))
                .background(Theme::surface_card())
                .border(Border::new().width(1.0).fill(Theme::border_normal()))
                .shadow(Shadow::new().blur(16.0).color(Color::from_argb(220, 0, 0, 0)))
                .corner_radius(Theme::RADIUS_MD)
                .on_press(|e: Event<PressEventData>| {
                    e.stop_propagation();
                })
                .child(list_body),
        )
    } else {
        None
    };

    rect()
        .width(Size::fill())
        .direction(Direction::vertical())
        .child(header)
        .maybe_child(popup)
        .into()
}

/// Standalone interactive dropdown selector component for a list of string options.
pub fn dropdown_select<O: IntoIterator<Item = impl Into<String>>>(
    props: DropdownSelectProps<O>,
    on_toggle: impl FnMut(Event<PressEventData>) + 'static,
    on_select: impl FnMut(usize) + 'static,
    on_hover: impl FnMut(Option<usize>) + 'static,
) -> Element {
    let sel_label = props.selected_label;
    let on_select = std::rc::Rc::new(std::cell::RefCell::new(on_select));
    let on_hover = std::rc::Rc::new(std::cell::RefCell::new(on_hover));

    let mut selected_idx = None;
    let items = props
        .options
        .into_iter()
        .enumerate()
        .map(|(idx, opt)| {
            let opt_str: String = opt.into();
            let is_sel = opt_str == sel_label;
            if is_sel {
                selected_idx = Some(idx);
            }
            let is_hov = props.hovered_idx == Some(idx);
            let on_select_item = std::rc::Rc::clone(&on_select);
            let on_hover_enter = std::rc::Rc::clone(&on_hover);
            let on_hover_leave = std::rc::Rc::clone(&on_hover);

            dropdown_item(
                opt_str,
                is_sel,
                is_hov,
                move |_| {
                    (on_select_item.borrow_mut())(idx);
                },
                move |_| {
                    (on_hover_enter.borrow_mut())(Some(idx));
                },
                move |_| {
                    (on_hover_leave.borrow_mut())(None);
                },
            )
        })
        .collect::<Vec<_>>();

    dropdown_menu(
        sel_label,
        props.is_open,
        selected_idx,
        props.scroll_controller,
        props.scrollbar_state,
        on_toggle,
        items,
    )
}

/// Fluent-style horizontal Key-Value row with a label on the left and right-aligned dropdown.
pub fn dropdown_row<O: IntoIterator<Item = impl Into<String>>>(
    props: DropdownRowProps<O>,
    on_toggle: impl FnMut(Event<PressEventData>) + 'static,
    on_select: impl FnMut(usize) + 'static,
    on_hover: impl FnMut(Option<usize>) + 'static,
) -> Element {
    rect()
        .width(Size::fill())
        .padding(Gaps::new(6.0, 12.0, 6.0, 12.0))
        .direction(Direction::horizontal())
        .main_align(Alignment::SpaceBetween)
        .cross_align(Alignment::Center)
        .child(
            label()
                .text(props.label_text)
                .font_size(Theme::FONT_BODY_SM)
                .color(Theme::text_primary()),
        )
        .child(
            rect()
                .width(Size::px(180.0))
                .direction(Direction::vertical())
                .child(dropdown_select(
                    DropdownSelectProps {
                        selected_label: props.selected_label,
                        options: props.options,
                        is_open: props.is_open,
                        hovered_idx: props.hovered_idx,
                        scroll_controller: props.scroll_controller,
                        scrollbar_state: props.scrollbar_state,
                    },
                    on_toggle,
                    on_select,
                    on_hover,
                )),
        )
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dropdown_components_render() {
        let item = dropdown_item("Item 1", true, false, |_| {}, |_| {}, |_| {});
        let menu = dropdown_menu("Select Option", true, Some(0), None, None, |_| {}, vec![item]);
        let select_many = dropdown_select(
            DropdownSelectProps {
                selected_label: "Option A".to_string(),
                options: vec![
                    "Option A", "Option B", "Option C", "Option D", "Option E", "Option F",
                    "Option G",
                ],
                is_open: true,
                hovered_idx: Some(1),
                scroll_controller: None,
                scrollbar_state: None,
            },
            |_| {},
            |_| {},
            |_| {},
        );
        let row = dropdown_row(
            DropdownRowProps {
                label_text: "Theme".to_string(),
                selected_label: "Neutral Dark".to_string(),
                options: vec!["Neutral Dark", "OLED Black"],
                is_open: false,
                hovered_idx: None,
                scroll_controller: None,
                scrollbar_state: None,
            },
            |_| {},
            |_| {},
            |_| {},
        );
        let _ = (menu, select_many, row);
    }

    #[test]
    fn test_buttons_and_controls_render() {
        let btn_primary = button_primary("Apply", |_| {});
        let btn_secondary = button_secondary("Cancel", |_| {});
        let btn_custom = button("Delete", ButtonVariant::Danger, |_| {});
        let btn_row = button_row("Reset Settings", "Reset", |_| {});
        let btn_icon = button_icon("✕", |_| {});
        let badge = key_badge("Ctrl+O");
        let pill = status_pill("Active", true);
        let switch = switch_row("Dark Mode", true, |_| {});
        let sec = section("Section Title");
        let sec_hdr = section_header("Section Header");
        let div = divider();
        let vdiv = vertical_divider();
        let p_hdr = pane_header("General Settings", Some("Configure app"), None);
        let crd = card(vec![card_row(label().text("Left").into(), label().text("Right").into())]);
        let exp_card = expandable_card("Extension Name", Some("v1.0.0"), false, |_| {}, vec![]);
        let dropzone = file_dropzone("Drop bundle here", "extensions/", |_| {});
        let empty = empty_state("🔍", "No items found", Some("Try searching again"));
        let tbl_hdr = table_header(vec!["Col 1", "Col 2"], &[Size::percent(50.0), Size::percent(50.0)]);
        let tbl_r = table_row(vec![label().text("A").into(), label().text("B").into()], &[Size::percent(50.0), Size::percent(50.0)], false, None::<fn(Event<PressEventData>)>);
        let tbl = table(TableProps {
            column_widths: vec![Size::percent(50.0), Size::percent(50.0)],
            header: Some(tbl_hdr),
            rows: vec![tbl_r],
            show_borders: true,
        });

        let txt = text_field("query", "placeholder", Size::px(200.0));
        let txt_row = text_field_row("Label", "query", "placeholder");

        let _ = (
            btn_primary,
            btn_secondary,
            btn_custom,
            btn_row,
            btn_icon,
            badge,
            pill,
            switch,
            sec,
            sec_hdr,
            div,
            vdiv,
            p_hdr,
            crd,
            exp_card,
            dropzone,
            empty,
            tbl,
            txt,
            txt_row,
        );
    }
}
