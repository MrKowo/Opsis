use freya::prelude::*;
use opsis_extension_api::ExtensionManifest;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::hotkeys::ActionDisplayItem;
use crate::manager::ExtensionManager;
use crate::ui::{
    button_row, button_secondary, card, divider, dropdown_row, empty_state,
    expandable_card, file_dropzone, info_row, key_badge, pane_header, section_header, status_pill,
    switch_row, table, table_header, table_row, text_field, use_init_opsis_theme,
    DropdownLayoutMetrics, DropdownRowProps, ScrollbarState, TableProps, Theme,
};

const APP_ICON: &[u8] = include_bytes!("../assets/icon.png");

const EXAMPLE_DROPDOWN_OPTIONS: &[&str] = &[
    "Option 1",
    "Option 2",
    "Option 3",
    "Option 4",
    "Option 5",
    "Option 6",
    "Option 7",
    "Option 8",
];

/// Spawns the native floating Settings window.
pub fn open_settings_window(
    ext_mgr: Arc<Mutex<ExtensionManager>>,
    on_settings_changed: Option<Arc<dyn Fn() + Send + Sync + 'static>>,
) {
    spawn(async move {
        let _ = Platform::get()
            .launch_window(
                WindowConfig::new(move || {
                    let ext_mgr = Arc::clone(&ext_mgr);
                    let on_settings_changed = on_settings_changed.clone();
                    settings_window_view(ext_mgr, on_settings_changed)
                })
                .with_title("Settings")
                .with_icon(LaunchConfig::window_icon(APP_ICON))
                .with_size(680.0, 480.0)
                .with_background(Theme::surface_base()),
            )
            .await;
    });
}

fn settings_window_view(
    ext_mgr: Arc<Mutex<ExtensionManager>>,
    on_settings_changed: Option<Arc<dyn Fn() + Send + Sync + 'static>>,
) -> Element {
    use_init_opsis_theme();

    let active_tab = use_state(|| 0usize); // Default to General tab
    let current_tab = *active_tab.read();

    let refresh_trigger = use_state(|| 0usize);
    let _ = *refresh_trigger.read();

    let rebind_target = use_state(|| None::<String>);

    let mut toggle_example = use_state(|| true);
    let mut dropdown_idx = use_state(|| 0usize);
    let mut dropdown_open = use_state(|| false);
    let mut dropdown_hovered = use_state(|| None::<usize>);
    let mut dropdown_scroll = use_scroll_controller(ScrollConfig::default);
    let mut dropdown_drag = use_state(|| None::<(f32, i32)>);
    let mut dropdown_hover = use_state(|| false);
    let mut button_clicks = use_state(|| 0usize);

    let initial_settings = if let Ok(manager) = ext_mgr.lock() {
        manager.settings.clone()
    } else {
        crate::config::AppSettings::default()
    };

    let mut dark_mode = use_state(|| initial_settings.dark_mode);
    let mut show_watermark = use_state(|| initial_settings.show_watermark);
    let mut acrylic_background = use_state(|| initial_settings.acrylic_background);

    let expanded_items = use_state(HashSet::<String>::new);
    let status_banner = use_state(|| None::<(String, bool)>);
    let search_query = use_state(String::new);

    let (extensions, ext_dir_str, is_portable, is_loading) = if let Ok(manager) = ext_mgr.lock() {
        let manifests: Vec<ExtensionManifest> = manager
            .loaded_extensions
            .iter()
            .map(|e| e.manifest.clone())
            .collect();
        let dir_str = manager.extensions_dir.display().to_string();
        (manifests, dir_str, manager.is_portable, manager.is_loading())
    } else {
        (Vec::new(), "extensions".to_string(), true, false)
    };

    let ext_mgr_key = Arc::clone(&ext_mgr);
    let ext_mgr_dark = Arc::clone(&ext_mgr);
    let ext_mgr_watermark = Arc::clone(&ext_mgr);
    let ext_mgr_acrylic = Arc::clone(&ext_mgr);
    let on_change_dark = on_settings_changed.clone();
    let on_change_watermark = on_settings_changed.clone();
    let on_change_acrylic = on_settings_changed.clone();
    let mut rebind_target_key = rebind_target;
    let mut refresh_trigger_key = refresh_trigger;

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .background(Theme::surface_base())
        .on_press(move |_| {
            let was_dragging = dropdown_drag.peek().is_some();
            if was_dragging {
                dropdown_drag.set(None);
                dropdown_hover.set(false);
            } else if *dropdown_open.peek() {
                dropdown_open.set(false);
            }
        })
        .on_global_key_down(move |e: Event<KeyboardEventData>| {
            let key_str = match &e.key {
                Key::Character(c) => c.clone(),
                Key::Named(named) => format!("{named:?}"),
            };

            let active_target = { rebind_target_key.read().clone() };

            // If in listening mode, assign the pressed key to the target action
            if let Some(target_action_id) = active_target {
                if key_str == "Escape" {
                    rebind_target_key.set(None);
                } else {
                    if let Ok(mut manager) = ext_mgr_key.lock() {
                        manager.rebind_hotkey(&target_action_id, key_str);
                    }
                    rebind_target_key.set(None);
                    refresh_trigger_key.with_mut(|mut count| *count = count.wrapping_add(1));
                }
                return;
            }

            if key_str == "Escape" && *dropdown_open.read() {
                dropdown_open.set(false);
                return;
            }

            if key_str == "q" || key_str == "Q" || key_str == "Escape" {
                spawn(async move {
                    let _ = Platform::get()
                        .post_callback(move |window_id, ctx| {
                            ctx.windows.remove(&window_id);
                        })
                        .await;
                });
            }
        })
        .direction(Direction::horizontal())
        .child(build_sidebar(current_tab, active_tab))
        .child(
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .background(Theme::surface_base())
                .padding(Gaps::new_all(20.0))
                .direction(Direction::vertical())
                .on_press(move |_| {
                    let was_dragging = dropdown_drag.peek().is_some();
                    if was_dragging {
                        dropdown_drag.set(None);
                        dropdown_hover.set(false);
                    } else if *dropdown_open.peek() {
                        dropdown_open.set(false);
                    }
                })
                .child(match current_tab {
                    0 => build_general_pane(
                        GeneralSettingsState {
                            toggle_value: *toggle_example.read(),
                            dropdown_idx: *dropdown_idx.read(),
                            dropdown_open: *dropdown_open.read(),
                            dropdown_hovered: *dropdown_hovered.read(),
                            dropdown_scroll: Some(dropdown_scroll),
                            dropdown_scrollbar: Some(ScrollbarState {
                                drag: Some(dropdown_drag),
                                hover: Some(dropdown_hover),
                            }),
                            button_clicks: *button_clicks.read(),
                        },
                        move |_| toggle_example.with_mut(|mut v| *v = !*v),
                        move |_| {
                            let will_open = !*dropdown_open.read();
                            if will_open {
                                let metrics = DropdownLayoutMetrics::compute(
                                    EXAMPLE_DROPDOWN_OPTIONS.len(),
                                    Some(*dropdown_idx.read()),
                                );
                                dropdown_scroll.scroll_to_y(metrics.initial_scroll_y);
                            }
                            dropdown_open.set(will_open);
                        },
                        move |idx| {
                            dropdown_idx.set(idx);
                            dropdown_open.set(false);
                        },
                        move |hov| {
                            dropdown_hovered.set(hov);
                        },
                        move |_| {
                            button_clicks.with_mut(|mut count| *count = count.wrapping_add(1));
                        },
                    ),
                    1 => build_appearance_pane(
                        *dark_mode.read(),
                        *show_watermark.read(),
                        *acrylic_background.read(),
                        move |_| {
                            let next = !*dark_mode.read();
                            dark_mode.set(next);
                            if let Ok(mut mgr) = ext_mgr_dark.lock() {
                                mgr.settings.dark_mode = next;
                                mgr.save_settings();
                            }
                            if let Some(ref cb) = on_change_dark {
                                cb();
                            }
                        },
                        move |_| {
                            let next = !*show_watermark.read();
                            show_watermark.set(next);
                            if let Ok(mut mgr) = ext_mgr_watermark.lock() {
                                mgr.settings.show_watermark = next;
                                mgr.save_settings();
                            }
                            if let Some(ref cb) = on_change_watermark {
                                cb();
                            }
                        },
                        move |_| {
                            let next = !*acrylic_background.read();
                            acrylic_background.set(next);
                            if let Ok(mut mgr) = ext_mgr_acrylic.lock() {
                                mgr.settings.acrylic_background = next;
                                mgr.save_settings();
                            }
                            if let Some(ref cb) = on_change_acrylic {
                                cb();
                            }
                        },
                    ),
                    2 => build_extensions_pane(ExtensionsPaneProps {
                        extensions,
                        ext_dir: ext_dir_str,
                        is_portable,
                        is_loading,
                        ext_mgr,
                        refresh_trigger,
                        expanded_items,
                        status_banner,
                    }),
                    3 => build_shortcuts_pane(
                        ext_mgr,
                        refresh_trigger,
                        rebind_target,
                        search_query,
                    ),
                    _ => build_about_pane(),
                }),
        )
        .into()
}

fn build_sidebar(current_tab: usize, mut active_tab: State<usize>) -> Element {
    let tabs = [
        (0, "General"),
        (1, "Appearance"),
        (2, "Extensions"),
        (3, "Shortcuts"),
        (4, "About"),
    ];

    let tab_elements: Vec<Element> = tabs
        .into_iter()
        .map(|(idx, title)| {
            let is_active = current_tab == idx;
            rect()
                .width(Size::fill())
                .padding(Gaps::new(6.0, 10.0, 6.0, 10.0))
                .background(if is_active {
                    Color::from_argb(50, 56, 189, 248)
                } else {
                    Color::TRANSPARENT
                })
                .border(Border::new().width(1.0).fill(if is_active {
                    Theme::accent_primary()
                } else {
                    Color::TRANSPARENT
                }))
                .corner_radius(Theme::RADIUS_MD)
                .on_press(move |_| {
                    active_tab.set(idx);
                })
                .child(
                    label()
                        .text(title)
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
        })
        .collect();

    rect()
        .width(Size::px(140.0))
        .height(Size::fill())
        .background(Theme::surface_panel())
        .border(
            Border::new()
                .width(1.0)
                .fill(Theme::border_subtle()),
        )
        .padding(Gaps::new(16.0, 8.0, 16.0, 8.0))
        .direction(Direction::vertical())
        .spacing(4.0)
        .children(tab_elements)
        .into()
}

struct GeneralSettingsState {
    pub toggle_value: bool,
    pub dropdown_idx: usize,
    pub dropdown_open: bool,
    pub dropdown_hovered: Option<usize>,
    pub dropdown_scroll: Option<ScrollController>,
    pub dropdown_scrollbar: Option<ScrollbarState>,
    pub button_clicks: usize,
}

fn build_general_pane(
    state: GeneralSettingsState,
    on_toggle: impl FnMut(Event<PressEventData>) + 'static,
    on_toggle_dropdown: impl FnMut(Event<PressEventData>) + 'static,
    on_select_dropdown: impl FnMut(usize) + 'static,
    on_hover_dropdown: impl FnMut(Option<usize>) + 'static,
    on_button_press: impl FnMut(Event<PressEventData>) + 'static,
) -> Element {
    let dropdown_label = EXAMPLE_DROPDOWN_OPTIONS
        .get(state.dropdown_idx)
        .copied()
        .unwrap_or("Option 1");

    let button_label = if state.button_clicks == 0 {
        "Button".to_string()
    } else {
        format!("Clicked ({}x)", state.button_clicks)
    };

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .direction(Direction::vertical())
        .spacing(12.0)
        .child(pane_header(
            "General",
            Some("Configure application preferences and interactive controls."),
            None,
        ))
        .child(section_header("Example Controls"))
        .child(card(vec![
            switch_row("Toggle example", state.toggle_value, on_toggle),
            dropdown_row(
                DropdownRowProps {
                    label_text: "Dropdown example".to_string(),
                    selected_label: dropdown_label.to_string(),
                    options: EXAMPLE_DROPDOWN_OPTIONS.iter().copied(),
                    is_open: state.dropdown_open,
                    hovered_idx: state.dropdown_hovered,
                    scroll_controller: state.dropdown_scroll,
                    scrollbar_state: state.dropdown_scrollbar,
                },
                on_toggle_dropdown,
                on_select_dropdown,
                on_hover_dropdown,
            ),
            button_row("Button example", button_label, on_button_press),
        ]))
        .into()
}

fn build_appearance_pane(
    dark_mode: bool,
    show_watermark: bool,
    acrylic_background: bool,
    on_toggle_dark: impl FnMut(Event<PressEventData>) + 'static,
    on_toggle_watermark: impl FnMut(Event<PressEventData>) + 'static,
    on_toggle_acrylic: impl FnMut(Event<PressEventData>) + 'static,
) -> Element {
    rect()
        .width(Size::fill())
        .height(Size::fill())
        .direction(Direction::vertical())
        .spacing(12.0)
        .child(pane_header(
            "Appearance",
            Some("Customize visual themes, canvas watermark, and surface styling."),
            None,
        ))
        .child(section_header("Visual Preferences"))
        .child(card(vec![
            switch_row("Dark Theme Mode", dark_mode, on_toggle_dark),
            switch_row("Canvas Logo Watermark", show_watermark, on_toggle_watermark),
            switch_row("Enable Acrylic Background", acrylic_background, on_toggle_acrylic),
        ]))
        .child(section_header("Theme & Palette"))
        .child(card(vec![
            info_row("Active Theme", "Neutral Dark"),
            info_row("Accent Primary", "Sky Cyan (#38BDF8)"),
            info_row("Surface Elevation", "Deep Base (#121214) / Panel (#18181C)"),
        ]))
        .into()
}

struct ExtensionsPaneProps {
    pub extensions: Vec<ExtensionManifest>,
    pub ext_dir: String,
    pub is_portable: bool,
    pub is_loading: bool,
    pub ext_mgr: Arc<Mutex<ExtensionManager>>,
    pub refresh_trigger: State<usize>,
    pub expanded_items: State<HashSet<String>>,
    pub status_banner: State<Option<(String, bool)>>,
}

fn build_extensions_pane(mut props: ExtensionsPaneProps) -> Element {
    let ext_mgr_drop = Arc::clone(&props.ext_mgr);

    let status_banner_element: Element =
        if let Some((msg, is_success)) = props.status_banner.read().as_ref() {
            rect()
                .width(Size::fill())
                .padding(Gaps::new(2.0, 4.0, 2.0, 4.0))
                .child(status_pill(msg.clone(), *is_success))
                .into()
        } else {
            rect().into()
        };

    let extensions_list: Element = if props.is_loading && props.extensions.is_empty() {
        empty_state(
            "⏳",
            "Scanning extensions...",
            Some("Loading native dynamic libraries and bundles in background."),
        )
    } else if props.extensions.is_empty() {
        empty_state(
            "🔌",
            "No external extensions installed",
            Some("Drop an .opx bundle or place native dynamic libraries into the extensions folder."),
        )
    } else {
        rect()
            .width(Size::fill())
            .direction(Direction::vertical())
            .spacing(6.0)
            .children(props.extensions.into_iter().map(|ext| {
                let id = ext.id.clone();
                let is_expanded = props.expanded_items.read().contains(&id);
                let id_for_click = id.clone();
                let mut expanded_items_row = props.expanded_items;

                expandable_card(
                    ext.name.clone(),
                    Some(format!("v{}", ext.version)),
                    is_expanded,
                    move |_| {
                        expanded_items_row.with_mut(|mut set| {
                            if set.contains(&id_for_click) {
                                set.remove(&id_for_click);
                            } else {
                                set.insert(id_for_click.clone());
                            }
                        });
                    },
                    vec![
                        info_row("Extension ID", &ext.id),
                        info_row("Author", &ext.author),
                        info_row("API Version", format!("v{}", ext.api_version)),
                        info_row("Description", &ext.description),
                    ],
                )
            }))
            .into()
    };

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .direction(Direction::vertical())
        .spacing(12.0)
        .child(pane_header(
            "Extensions",
            Some("Manage installed extension modules, inspect capabilities, and install packages."),
            Some(status_pill(
                if props.is_portable {
                    "Mode: Portable"
                } else {
                    "Mode: User Profile"
                },
                props.is_portable,
            )),
        ))
        .child(file_dropzone(
            "Drop .opx bundle or native library here to install",
            props.ext_dir,
            move |e: Event<FileEventData>| {
                if let Some(ref src_path) = e.file_path {
                    let ext = src_path
                        .extension()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_lowercase();

                    if ext == "opx" || ext == "dll" || ext == "so" || ext == "dylib" {
                        if let Ok(mut manager) = ext_mgr_drop.lock() {
                            let target_dir = manager.extensions_dir.clone();
                            let _ = std::fs::create_dir_all(&target_dir);

                            if let Some(file_name) = src_path.file_name() {
                                let dest_path = target_dir.join(file_name);
                                if std::fs::copy(src_path, &dest_path).is_ok() {
                                    manager.discover_and_load_all();
                                    props.status_banner.set(Some((
                                        format!("Installed {}", file_name.to_string_lossy()),
                                        true,
                                    )));
                                    props.refresh_trigger.with_mut(|mut val| *val = val.wrapping_add(1));
                                }
                            }
                        }
                    } else {
                        props.status_banner.set(Some((
                            "Please drop an .opx package or dynamic library".to_string(),
                            false,
                        )));
                    }
                }
            },
        ))
        .child(status_banner_element)
        .child(section_header("Installed Extensions"))
        .child(
            ScrollView::new()
                .width(Size::fill())
                .height(Size::fill())
                .child(extensions_list),
        )
        .into()
}

fn build_shortcuts_pane(
    ext_mgr: Arc<Mutex<ExtensionManager>>,
    mut refresh_trigger: State<usize>,
    mut rebind_target: State<Option<String>>,
    search_query: State<String>,
) -> Element {
    let filter_text = search_query.read().to_lowercase();

    let (actions, has_custom) = if let Ok(mgr) = ext_mgr.lock() {
        let items = mgr.hotkey_registry.all_actions_for_display();
        let any_custom = items.iter().any(|i| i.is_customized);
        (items, any_custom)
    } else {
        (Vec::new(), false)
    };

    let active_rebind_id = rebind_target.read().clone();
    let ext_mgr_for_reset_all = Arc::clone(&ext_mgr);

    let filtered_actions: Vec<&ActionDisplayItem> = actions
        .iter()
        .filter(|a| {
            if filter_text.is_empty() {
                true
            } else {
                a.name.to_lowercase().contains(&filter_text)
                    || a.category.to_lowercase().contains(&filter_text)
                    || a.keys_display.to_lowercase().contains(&filter_text)
            }
        })
        .collect();

    let column_widths = vec![Size::percent(55.0), Size::percent(45.0)];

    let rows: Vec<Element> = if filtered_actions.is_empty() {
        vec![empty_state(
            "🔍",
            "No shortcuts found",
            Some("Try adjusting your search query."),
        )]
    } else {
        filtered_actions
            .into_iter()
            .enumerate()
            .map(|(idx, action)| {
                let action_id = action.id.clone();
                let action_name = action.name.clone();
                let keys_display = action.keys_display.clone();
                let is_customized = action.is_customized;
                let is_recording = active_rebind_id.as_deref() == Some(&action_id);

                let ext_mgr_row = Arc::clone(&ext_mgr);
                let action_id_for_rebind = action_id.clone();
                let action_id_for_reset = action_id.clone();

                let left_col = rect()
                    .width(Size::fill())
                    .child(
                        label()
                            .text(action_name)
                            .font_size(Theme::FONT_BODY_SM)
                            .color(Theme::text_primary()),
                    )
                    .into();

                let mut right_col = rect()
                    .width(Size::fill())
                    .direction(Direction::horizontal())
                    .main_align(Alignment::End)
                    .cross_align(Alignment::Center)
                    .spacing(6.0);

                if is_customized {
                    right_col = right_col.child(status_pill("Custom", true));
                }

                if is_recording {
                    right_col = right_col
                        .child(
                            rect()
                                .padding(Gaps::new(2.0, 7.0, 2.0, 7.0))
                                .background(Theme::accent_warm_bg())
                                .border(Border::new().width(1.0).fill(Theme::accent_warm()))
                                .corner_radius(Theme::RADIUS_SM)
                                .child(
                                    label()
                                        .text("Press key...")
                                        .font_size(Theme::FONT_CAPTION)
                                        .font_weight(FontWeight::BOLD)
                                        .color(Theme::accent_warm()),
                                ),
                        )
                        .child(button_secondary("Cancel", move |_| {
                            rebind_target.set(None);
                        }));
                } else {
                    let mut rebind_target_row = rebind_target;
                    right_col = right_col.child(
                        rect()
                            .on_press(move |_| {
                                rebind_target_row.set(Some(action_id_for_rebind.clone()));
                            })
                            .child(key_badge(keys_display)),
                    );

                    if is_customized {
                        right_col = right_col.child(button_secondary("Reset", move |_| {
                            if let Ok(mut mgr) = ext_mgr_row.lock() {
                                mgr.reset_hotkey(&action_id_for_reset);
                            }
                            refresh_trigger.with_mut(|mut count| *count = count.wrapping_add(1));
                        }));
                    }
                }

                table_row(
                    vec![left_col, right_col.into()],
                    &column_widths,
                    idx % 2 == 0,
                    None::<fn(Event<PressEventData>)>,
                )
            })
            .collect()
    };

    let table_header_elem = table_header(vec!["Action / Function", "Assigned Key"], &column_widths);

    let reset_action = if has_custom {
        Some(button_secondary("Reset All Defaults", move |_| {
            if let Ok(mut mgr) = ext_mgr_for_reset_all.lock() {
                mgr.reset_all_hotkeys();
            }
            refresh_trigger.with_mut(|mut count| *count = count.wrapping_add(1));
        }))
    } else {
        None
    };

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .direction(Direction::vertical())
        .spacing(12.0)
        .child(pane_header(
            "Keyboard Shortcuts",
            Some("Click any shortcut key badge to rebind. Press Escape while recording to cancel."),
            reset_action,
        ))
        .child(
            rect()
                .width(Size::fill())
                .direction(Direction::horizontal())
                .main_align(Alignment::End)
                .child(text_field(
                    search_query.read().clone(),
                    "Search shortcuts...",
                    Size::px(220.0),
                )),
        )
        .child(table(TableProps {
            column_widths,
            header: Some(table_header_elem),
            rows,
            show_borders: true,
        }))
        .into()
}

fn build_about_pane() -> Element {
    const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .direction(Direction::vertical())
        .spacing(12.0)
        .child(pane_header(
            "About Opsis",
            Some("Ultra-lightweight, portable image viewer built with Rust, Freya, and Skia."),
            None,
        ))
        .child(section_header("Application Information"))
        .child(card(vec![
            info_row("Version", PKG_VERSION),
            info_row("License", "MIT"),
            info_row("Architecture", "Microkernel Extension-First"),
            info_row("Graphics Backend", "Skia 2D Hardware-Accelerated"),
            info_row("Window Manager", "Freya / Winit"),
        ]))
        .child(divider())
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_panes_render() {
        let _general = build_general_pane(
            GeneralSettingsState {
                toggle_value: true,
                dropdown_idx: 0,
                dropdown_open: false,
                dropdown_hovered: None,
                dropdown_scroll: None,
                dropdown_scrollbar: None,
                button_clicks: 0,
            },
            |_| {},
            |_| {},
            |_| {},
            |_| {},
            |_| {},
        );
        let _appearance = build_appearance_pane(true, true, false, |_| {}, |_| {}, |_| {});
        let _about = build_about_pane();
        let _tbl_hdr = table_header(vec!["Action", "Key"], &[Size::percent(50.0), Size::percent(50.0)]);
    }
}
