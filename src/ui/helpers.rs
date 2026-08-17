use freya::prelude::*;

/// Maximum number of list elements visible in a dropdown before scrolling activates.
pub const MAX_DROPDOWN_VISIBLE_ITEMS: usize = 5;

/// Exact height per dropdown item row in pixels.
pub const DROPDOWN_ITEM_ROW_HEIGHT: f32 = 28.0;

/// Layout metrics for positioning a dropdown popover and centering its scroll view over the selected option.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DropdownLayoutMetrics {
    pub scroll_slot: usize,
    pub slot_in_window: usize,
    pub initial_scroll_y: i32,
    pub top_offset: f32,
}

impl DropdownLayoutMetrics {
    /// Computes popover `top_offset` and initial `scroll_y` offset for the given total item count and selected index.
    pub fn compute(total_items: usize, selected_idx: Option<usize>) -> Self {
        let sel_idx = selected_idx.unwrap_or(0);
        let visible_count = total_items.min(MAX_DROPDOWN_VISIBLE_ITEMS);
        let slot_step = DROPDOWN_ITEM_ROW_HEIGHT + 2.0;

        let (scroll_slot, slot_in_window) = if total_items <= MAX_DROPDOWN_VISIBLE_ITEMS {
            (0, sel_idx.min(visible_count.saturating_sub(1)))
        } else {
            let max_scroll_slot = total_items.saturating_sub(MAX_DROPDOWN_VISIBLE_ITEMS);
            let scroll_slot = sel_idx.saturating_sub(2).min(max_scroll_slot);
            let slot_in_window =
                sel_idx.saturating_sub(scroll_slot).min(visible_count.saturating_sub(1));
            (scroll_slot, slot_in_window)
        };

        let initial_scroll_y = -((scroll_slot as f32) * slot_step) as i32;
        let top_offset = -4.0 - (slot_in_window as f32 * slot_step);

        Self {
            scroll_slot,
            slot_in_window,
            initial_scroll_y,
            top_offset,
        }
    }
}

/// Style variants for standard button components.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Danger,
}

/// Persistent interactive state for a custom draggable scrollbar.
#[derive(Clone, Copy, Default)]
pub struct ScrollbarState {
    pub drag: Option<State<Option<(f32, i32)>>>,
    pub hover: Option<State<bool>>,
}

/// Configuration properties for a `dropdown_select` component.
pub struct DropdownSelectProps<O> {
    pub selected_label: String,
    pub options: O,
    pub is_open: bool,
    pub hovered_idx: Option<usize>,
    pub scroll_controller: Option<ScrollController>,
    pub scrollbar_state: Option<ScrollbarState>,
}

/// Configuration properties for a `dropdown_row` component.
pub struct DropdownRowProps<O> {
    pub label_text: String,
    pub selected_label: String,
    pub options: O,
    pub is_open: bool,
    pub hovered_idx: Option<usize>,
    pub scroll_controller: Option<ScrollController>,
    pub scrollbar_state: Option<ScrollbarState>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dropdown_layout_metrics() {
        // Short list (<= 5 items)
        let m_short_first = DropdownLayoutMetrics::compute(3, Some(0));
        assert_eq!(m_short_first.scroll_slot, 0);
        assert_eq!(m_short_first.slot_in_window, 0);
        assert_eq!(m_short_first.initial_scroll_y, 0);
        assert_eq!(m_short_first.top_offset, -4.0);

        let m_short_last = DropdownLayoutMetrics::compute(3, Some(2));
        assert_eq!(m_short_last.scroll_slot, 0);
        assert_eq!(m_short_last.slot_in_window, 2);
        assert_eq!(m_short_last.initial_scroll_y, 0);
        assert_eq!(m_short_last.top_offset, -4.0 - 60.0);

        // Long list (13 items, e.g. UI scale options)
        let m_long_first = DropdownLayoutMetrics::compute(13, Some(0));
        assert_eq!(m_long_first.scroll_slot, 0);
        assert_eq!(m_long_first.slot_in_window, 0);
        assert_eq!(m_long_first.initial_scroll_y, 0);
        assert_eq!(m_long_first.top_offset, -4.0);

        let m_long_second = DropdownLayoutMetrics::compute(13, Some(1));
        assert_eq!(m_long_second.scroll_slot, 0);
        assert_eq!(m_long_second.slot_in_window, 1);
        assert_eq!(m_long_second.initial_scroll_y, 0);
        assert_eq!(m_long_second.top_offset, -34.0);

        // Middle item (index 6: "2.50x")
        let m_long_middle = DropdownLayoutMetrics::compute(13, Some(6));
        assert_eq!(m_long_middle.scroll_slot, 4);
        assert_eq!(m_long_middle.slot_in_window, 2);
        assert_eq!(m_long_middle.initial_scroll_y, -120);
        assert_eq!(m_long_middle.top_offset, -64.0);

        // Last item (index 12: "4.00x")
        let m_long_last = DropdownLayoutMetrics::compute(13, Some(12));
        assert_eq!(m_long_last.scroll_slot, 8);
        assert_eq!(m_long_last.slot_in_window, 4);
        assert_eq!(m_long_last.initial_scroll_y, -240);
        assert_eq!(m_long_last.top_offset, -124.0);
    }
}
