use freya::prelude::*;
use opsis_extension_api::ExtensionManifest;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::manager::ExtensionManager;

const APP_ICON: &[u8] = include_bytes!("../assets/icon.png");

/// Spawns the native floating Settings window.
pub fn open_settings_window(ext_mgr: Arc<Mutex<ExtensionManager>>) {
    spawn(async move {
        let _ = Platform::get()
            .launch_window(
                WindowConfig::new(move || {
                    let ext_mgr = Arc::clone(&ext_mgr);
                    settings_window_view(ext_mgr)
                })
                .with_title("Settings")
                .with_icon(LaunchConfig::window_icon(APP_ICON))
                .with_size(680.0, 480.0)
                .with_background(Color::from_rgb(18, 18, 20)),
            )
            .await;
    });
}

fn settings_window_view(ext_mgr: Arc<Mutex<ExtensionManager>>) -> Element {
    let active_tab = use_state(|| 2usize); // Default to Extensions tab
    let current_tab = *active_tab.read();

    let refresh_trigger = use_state(|| 0usize);
    let _ = *refresh_trigger.read();

    let (extensions, ext_dir_str, is_portable) = if let Ok(manager) = ext_mgr.lock() {
        let manifests: Vec<ExtensionManifest> = manager
            .loaded_extensions
            .iter()
            .map(|e| e.manifest.clone())
            .collect();
        let dir_str = manager.extensions_dir.display().to_string();
        (manifests, dir_str, manager.is_portable)
    } else {
        (Vec::new(), "extensions".to_string(), true)
    };

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .background(Color::from_rgb(18, 18, 20))
        .on_global_key_down(move |e: Event<KeyboardEventData>| {
            let key_str = match &e.key {
                Key::Character(c) => c.clone(),
                Key::Named(named) => format!("{named:?}"),
            };
            if key_str == "q" || key_str == "Q" || key_str == "Escape" {
                let _ = Platform::get().post_callback(move |window_id, ctx| {
                    ctx.windows.remove(&window_id);
                });
            }
        })
        .direction(Direction::horizontal())
        .child(build_sidebar(current_tab, active_tab))
        .child(
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .padding(Gaps::new_all(20.0))
                .direction(Direction::vertical())
                .child(match current_tab {
                    0 => build_general_pane(),
                    1 => build_appearance_pane(),
                    2 => build_extensions_pane(
                        extensions,
                        ext_dir_str,
                        is_portable,
                        ext_mgr,
                        refresh_trigger,
                    ),
                    3 => build_shortcuts_pane(),
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

    rect()
        .width(Size::px(140.0))
        .height(Size::fill())
        .background(Color::from_rgb(14, 14, 16))
        .border(Border::new().width(1.0).fill(Color::from_rgb(28, 28, 32)))
        .padding(Gaps::new(16.0, 8.0, 16.0, 8.0))
        .direction(Direction::vertical())
        .spacing(2.0)
        .children(tabs.into_iter().map(|(idx, title)| {
            let is_active = current_tab == idx;

            rect()
                .width(Size::fill())
                .background(if is_active {
                    Color::from_rgb(28, 28, 34)
                } else {
                    Color::TRANSPARENT
                })
                .corner_radius(4.0)
                .padding(Gaps::new(6.0, 10.0, 6.0, 10.0))
                .on_press(move |_| {
                    active_tab.set(idx);
                })
                .child(
                    label()
                        .text(title)
                        .font_size(13.0)
                        .font_weight(if is_active {
                            FontWeight::BOLD
                        } else {
                            FontWeight::NORMAL
                        })
                        .color(if is_active {
                            Color::WHITE
                        } else {
                            Color::from_rgb(140, 140, 150)
                        }),
                )
                .into()
        }))
        .into()
}

fn build_general_pane() -> Element {
    rect()
        .width(Size::fill())
        .direction(Direction::vertical())
        .spacing(12.0)
        .child(
            label()
                .text("General Settings")
                .font_size(16.0)
                .font_weight(FontWeight::BOLD)
                .color(Color::WHITE),
        )
        .child(
            label()
                .text("Runtime: Pure Microkernel Host (Freya / Skia)")
                .font_size(12.0)
                .color(Color::from_rgb(150, 150, 160)),
        )
        .into()
}

fn build_appearance_pane() -> Element {
    rect()
        .width(Size::fill())
        .direction(Direction::vertical())
        .spacing(12.0)
        .child(
            label()
                .text("Appearance")
                .font_size(16.0)
                .font_weight(FontWeight::BOLD)
                .color(Color::WHITE),
        )
        .child(
            label()
                .text("Theme: Neutral Dark")
                .font_size(12.0)
                .color(Color::from_rgb(150, 150, 160)),
        )
        .into()
}

fn build_extensions_pane(
    extensions: Vec<ExtensionManifest>,
    ext_dir: String,
    is_portable: bool,
    ext_mgr: Arc<Mutex<ExtensionManager>>,
    mut refresh_trigger: State<usize>,
) -> Element {
    let mut expanded_items = use_state(HashSet::<String>::new);
    let mut status_banner = use_state(|| None::<(String, bool)>);
    let ext_mgr_drop = Arc::clone(&ext_mgr);

    let status_banner_element: Element =
        if let Some((msg, is_success)) = status_banner.read().as_ref() {
            rect()
                .width(Size::fill())
                .padding(Gaps::new(4.0, 8.0, 4.0, 8.0))
                .child(
                    label()
                        .text(msg.clone())
                        .font_size(12.0)
                        .color(if *is_success {
                            Color::from_rgb(140, 220, 160)
                        } else {
                            Color::from_rgb(220, 140, 120)
                        }),
                )
                .into()
        } else {
            rect().into()
        };

    let extensions_list: Element = if extensions.is_empty() {
        rect()
            .padding(Gaps::new_all(16.0))
            .child(
                label()
                    .text("No external extensions installed.")
                    .font_size(12.0)
                    .color(Color::from_rgb(120, 120, 130)),
            )
            .into()
    } else {
        rect()
            .width(Size::fill())
            .direction(Direction::vertical())
            .children(extensions.into_iter().map(|ext| {
                let id = ext.id.clone();
                let is_expanded = expanded_items.read().contains(&id);
                let id_for_click = id.clone();

                rect()
                    .width(Size::fill())
                    .direction(Direction::vertical())
                    .border(Border::new().width(1.0).fill(Color::from_rgb(28, 28, 32)))
                    // Dropdown Header Row
                    .child(
                        rect()
                            .width(Size::fill())
                            .padding(Gaps::new(8.0, 10.0, 8.0, 10.0))
                            .direction(Direction::horizontal())
                            .main_align(Alignment::SpaceBetween)
                            .cross_align(Alignment::Center)
                            .background(if is_expanded {
                                Color::from_rgb(24, 24, 28)
                            } else {
                                Color::from_rgb(18, 18, 22)
                            })
                            .on_press(move |_| {
                                expanded_items.with_mut(|mut set| {
                                    if set.contains(&id_for_click) {
                                        set.remove(&id_for_click);
                                    } else {
                                        set.insert(id_for_click.clone());
                                    }
                                });
                            })
                            .child(
                                rect()
                                    .direction(Direction::horizontal())
                                    .spacing(8.0)
                                    .cross_align(Alignment::Center)
                                    .child(
                                        label()
                                            .text(if is_expanded { "▼" } else { "▶" })
                                            .font_size(10.0)
                                            .color(Color::from_rgb(140, 140, 150)),
                                    )
                                    .child(
                                        label()
                                            .text(ext.name.clone())
                                            .font_size(13.0)
                                            .font_weight(FontWeight::BOLD)
                                            .color(Color::WHITE),
                                    ),
                            )
                            .child(
                                label()
                                    .text(format!("v{}", ext.version))
                                    .font_size(12.0)
                                    .color(Color::from_rgb(120, 120, 130)),
                            ),
                    )
                    // Dropdown Expanded Body
                    .maybe_child(if is_expanded {
                        Some(
                            rect()
                                .width(Size::fill())
                                .padding(Gaps::new(10.0, 26.0, 10.0, 26.0))
                                .background(Color::from_rgb(14, 14, 16))
                                .direction(Direction::vertical())
                                .spacing(4.0)
                                .child(
                                    label()
                                        .text(format!("ID: {}", ext.id))
                                        .font_size(11.0)
                                        .color(Color::from_rgb(130, 130, 140)),
                                )
                                .child(
                                    label()
                                        .text(format!("Author: {}", ext.author))
                                        .font_size(11.0)
                                        .color(Color::from_rgb(130, 130, 140)),
                                )
                                .child(
                                    label()
                                        .text(format!("API Version: {}", ext.api_version))
                                        .font_size(11.0)
                                        .color(Color::from_rgb(130, 130, 140)),
                                )
                                .child(
                                    label()
                                        .text(format!("Description: {}", ext.description))
                                        .font_size(11.0)
                                        .color(Color::from_rgb(160, 160, 170)),
                                ),
                        )
                    } else {
                        None
                    })
                    .into()
            }))
            .into()
    };

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .direction(Direction::vertical())
        .spacing(10.0)
        .child(
            rect()
                .direction(Direction::horizontal())
                .main_align(Alignment::SpaceBetween)
                .cross_align(Alignment::Center)
                .child(
                    label()
                        .text("Extensions")
                        .font_size(16.0)
                        .font_weight(FontWeight::BOLD)
                        .color(Color::WHITE),
                )
                .child(
                    label()
                        .text(if is_portable {
                            "Mode: Portable"
                        } else {
                            "Mode: User Profile"
                        })
                        .font_size(11.0)
                        .color(Color::from_rgb(130, 130, 140)),
                ),
        )
        // Simple Drop & Discovery Area
        .child(
            rect()
                .width(Size::fill())
                .padding(Gaps::new(8.0, 10.0, 8.0, 10.0))
                .background(Color::from_rgb(14, 14, 16))
                .border(Border::new().width(1.0).fill(Color::from_rgb(28, 28, 32)))
                .direction(Direction::horizontal())
                .main_align(Alignment::SpaceBetween)
                .cross_align(Alignment::Center)
                .on_file_drop(move |e: Event<FileEventData>| {
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
                                        status_banner.set(Some((
                                            format!(
                                                "Installed {}",
                                                file_name.to_string_lossy()
                                            ),
                                            true,
                                        )));
                                        refresh_trigger
                                            .with_mut(|mut val| *val = val.wrapping_add(1));
                                    }
                                }
                            }
                        } else {
                            status_banner.set(Some((
                                "Please drop an .opx package or dynamic library".to_string(),
                                false,
                            )));
                        }
                    }
                })
                .child(
                    label()
                        .text("Drop .opx or dynamic library here to install")
                        .font_size(11.0)
                        .color(Color::from_rgb(150, 150, 160)),
                )
                .child(
                    label()
                        .text(ext_dir)
                        .font_size(10.0)
                        .color(Color::from_rgb(100, 100, 110)),
                ),
        )
        .child(status_banner_element)
        // Scrollable Extensions Dropdown List
        .child(
            ScrollView::new()
                .width(Size::fill())
                .height(Size::fill())
                .child(extensions_list),
        )
        .into()
}

fn build_shortcuts_pane() -> Element {
    let shortcuts = [
        ("Open Image Picker", "O"),
        ("Zoom In", "+ / ="),
        ("Zoom Out", "- / _"),
        ("100% Original Size (1:1)", "0"),
        ("Toggle Horizontal / Vertical Fit", "H"),
        ("Toggle Window Maximize", "F"),
        ("Clear Loaded Image", "Escape"),
        ("Toggle Settings Window", "S"),
        ("Close Window / Exit", "Q"),
        ("Load Image via Drag & Drop", "Drag & Drop"),
    ];

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .direction(Direction::vertical())
        .spacing(10.0)
        .child(
            label()
                .text("Keyboard Shortcuts")
                .font_size(16.0)
                .font_weight(FontWeight::BOLD)
                .color(Color::WHITE),
        )
        .child(
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .direction(Direction::vertical())
                .border(Border::new().width(1.0).fill(Color::from_rgb(28, 28, 32)))
                .child(shortcuts_table_header("Action", "Key"))
                .child(
                    ScrollView::new()
                        .width(Size::fill())
                        .height(Size::fill())
                        .child(
                            rect()
                                .width(Size::fill())
                                .direction(Direction::vertical())
                                .children(shortcuts.into_iter().enumerate().map(|(idx, (action, key))| {
                                    shortcuts_table_row(action, key, idx % 2 == 0)
                                })),
                        ),
                ),
        )
        .into()
}

fn shortcuts_table_header(left: &'static str, right: &'static str) -> Element {
    rect()
        .width(Size::fill())
        .background(Color::from_rgb(14, 14, 16))
        .padding(Gaps::new(8.0, 20.0, 8.0, 16.0))
        .direction(Direction::horizontal())
        .border(Border::new().width(1.0).fill(Color::from_rgb(28, 28, 32)))
        .child(
            rect()
                .width(Size::percent(50.0))
                .child(
                    label()
                        .text(left)
                        .font_size(12.0)
                        .font_weight(FontWeight::BOLD)
                        .color(Color::from_rgb(140, 140, 150)),
                ),
        )
        .child(
            rect()
                .width(Size::percent(50.0))
                .child(
                    label()
                        .text(right)
                        .width(Size::fill())
                        .text_align(TextAlign::Right)
                        .font_size(12.0)
                        .font_weight(FontWeight::BOLD)
                        .color(Color::from_rgb(140, 140, 150)),
                ),
        )
        .into()
}

fn shortcuts_table_row(action: &'static str, key: &'static str, is_even: bool) -> Element {
    rect()
        .width(Size::fill())
        .background(if is_even {
            Color::from_rgb(18, 18, 20)
        } else {
            Color::from_rgb(15, 15, 17)
        })
        .padding(Gaps::new(8.0, 20.0, 8.0, 16.0))
        .direction(Direction::horizontal())
        .cross_align(Alignment::Center)
        .child(
            rect()
                .width(Size::percent(50.0))
                .child(
                    label()
                        .text(action)
                        .font_size(12.0)
                        .color(Color::from_rgb(170, 170, 180)),
                ),
        )
        .child(
            rect()
                .width(Size::percent(50.0))
                .child(
                    label()
                        .text(key)
                        .width(Size::fill())
                        .text_align(TextAlign::Right)
                        .font_size(12.0)
                        .font_weight(FontWeight::BOLD)
                        .color(Color::WHITE),
                ),
        )
        .into()
}

fn build_about_pane() -> Element {
    rect()
        .width(Size::fill())
        .direction(Direction::vertical())
        .spacing(10.0)
        .child(
            label()
                .text("About")
                .font_size(16.0)
                .font_weight(FontWeight::BOLD)
                .color(Color::WHITE),
        )
        .child(
            label()
                .text("Opsis Image Viewer • Version 0.1.0 • License: MIT")
                .font_size(12.0)
                .color(Color::from_rgb(160, 160, 170)),
        )
        .child(
            label()
                .text("Ultra-lightweight, portable image viewer built with Rust, Freya, and Skia.")
                .font_size(11.0)
                .color(Color::from_rgb(120, 120, 130)),
        )
        .into()
}
