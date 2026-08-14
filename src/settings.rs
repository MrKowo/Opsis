use freya::prelude::*;
use opsis_extension_api::ExtensionManifest;
use std::sync::{Arc, Mutex};

use crate::manager::ExtensionManager;

/// Spawns the native floating Settings & Extensions window.
pub fn open_settings_window(ext_mgr: Arc<Mutex<ExtensionManager>>) {
    spawn(async move {
        let _ = Platform::get()
            .launch_window(
                WindowConfig::new(move || {
                    let ext_mgr = Arc::clone(&ext_mgr);
                    settings_window_view(ext_mgr)
                })
                .with_title("Opsis - Settings")
                .with_size(740.0, 560.0)
                .with_background(Color::from_rgb(14, 14, 18)),
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
        .background(Color::from_rgb(14, 14, 18))
        .direction(Direction::horizontal())
        .child(build_sidebar(current_tab, active_tab))
        .child(
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .padding(Gaps::new_all(24.0))
                .direction(Direction::vertical())
                .spacing(16.0)
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
        .width(Size::px(180.0))
        .height(Size::fill())
        .background(Color::from_rgb(20, 20, 24))
        .border(Border::new().width(1.0).fill(Color::from_rgb(35, 35, 42)))
        .padding(Gaps::new(18.0, 10.0, 18.0, 10.0))
        .direction(Direction::vertical())
        .spacing(6.0)
        .children(
            std::iter::once(
                rect()
                    .padding(Gaps::new(0.0, 10.0, 10.0, 10.0))
                    .child(
                        label()
                            .text("Settings")
                            .font_size(16.0)
                            .font_weight(FontWeight::BOLD)
                            .color(Color::WHITE),
                    )
                    .into(),
            )
            .chain(tabs.into_iter().map(|(idx, title)| {
                let is_active = current_tab == idx;

                rect()
                    .width(Size::fill())
                    .background(if is_active {
                        Color::from_rgb(36, 36, 44)
                    } else {
                        Color::TRANSPARENT
                    })
                    .border(if is_active {
                        Border::new().width(1.0).fill(Color::from_rgb(52, 52, 62))
                    } else {
                        Border::new().width(0.0).fill(Color::TRANSPARENT)
                    })
                    .corner_radius(6.0)
                    .padding(Gaps::new(8.0, 12.0, 8.0, 12.0))
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
                                Color::from_rgb(160, 160, 170)
                            }),
                    )
                    .into()
            })),
        )
        .into()
}

fn build_general_pane() -> Element {
    rect()
        .width(Size::fill())
        .direction(Direction::vertical())
        .spacing(14.0)
        .child(
            label()
                .text("General Settings")
                .font_size(18.0)
                .font_weight(FontWeight::BOLD)
                .color(Color::WHITE),
        )
        .child(
            label()
                .text("Host runtime configurations and defaults.")
                .font_size(13.0)
                .color(Color::from_rgb(140, 140, 150)),
        )
        .into()
}

fn build_appearance_pane() -> Element {
    rect()
        .width(Size::fill())
        .direction(Direction::vertical())
        .spacing(14.0)
        .child(
            label()
                .text("Appearance")
                .font_size(18.0)
                .font_weight(FontWeight::BOLD)
                .color(Color::WHITE),
        )
        .child(
            label()
                .text("Canvas theme, background styling, and visual options.")
                .font_size(13.0)
                .color(Color::from_rgb(140, 140, 150)),
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
    let mut status_banner = use_state(|| None::<(String, bool)>);
    let count = extensions.len();
    let ext_mgr_drop = Arc::clone(&ext_mgr);

    let status_banner_element: Element =
        if let Some((msg, is_success)) = status_banner.read().as_ref() {
            rect()
                .width(Size::fill())
                .background(if *is_success {
                    Color::from_rgb(25, 45, 35)
                } else {
                    Color::from_rgb(50, 35, 25)
                })
                .border(Border::new().width(1.0).fill(if *is_success {
                    Color::from_rgb(45, 90, 65)
                } else {
                    Color::from_rgb(110, 60, 40)
                }))
                .corner_radius(8.0)
                .padding(Gaps::new(8.0, 12.0, 8.0, 12.0))
                .child(
                    label()
                        .text(msg.clone())
                        .font_size(12.0)
                        .color(if *is_success {
                            Color::from_rgb(160, 240, 180)
                        } else {
                            Color::from_rgb(240, 170, 130)
                        }),
                )
                .into()
        } else {
            rect().into()
        };

    let extensions_list_element: Element = if extensions.is_empty() {
        rect()
            .padding(Gaps::new_all(24.0))
            .direction(Direction::vertical())
            .spacing(6.0)
            .cross_align(Alignment::Center)
            .child(
                label()
                    .text("No external extensions installed.")
                    .font_size(14.0)
                    .color(Color::from_rgb(180, 180, 190)),
            )
            .child(
                label()
                    .text("Drag & drop .opx packages above or place them in the extensions directory.")
                    .font_size(12.0)
                    .color(Color::from_rgb(110, 110, 120)),
            )
            .into()
    } else {
        rect()
            .direction(Direction::vertical())
            .spacing(10.0)
            .children(
                extensions.into_iter().map(|ext| {
                    rect()
                        .background(Color::from_rgb(22, 22, 28))
                        .border(Border::new().width(1.0).fill(Color::from_rgb(40, 40, 48)))
                        .corner_radius(10.0)
                        .padding(Gaps::new_all(14.0))
                        .direction(Direction::vertical())
                        .spacing(8.0)
                        .child(
                            rect()
                                .direction(Direction::horizontal())
                                .main_align(Alignment::SpaceBetween)
                                .cross_align(Alignment::Center)
                                .child(
                                    rect()
                                        .direction(Direction::horizontal())
                                        .spacing(8.0)
                                        .cross_align(Alignment::Center)
                                        .child(
                                            label()
                                                .text(ext.name)
                                                .font_size(14.0)
                                                .font_weight(FontWeight::BOLD)
                                                .color(Color::WHITE),
                                        )
                                        .child(
                                            rect()
                                                .background(Color::from_rgb(35, 55, 45))
                                                .border(
                                                    Border::new()
                                                        .width(1.0)
                                                        .fill(Color::from_rgb(50, 100, 75)),
                                                )
                                                .corner_radius(4.0)
                                                .padding(Gaps::new(2.0, 6.0, 2.0, 6.0))
                                                .child(
                                                    label()
                                                        .text(format!("v{}", ext.version))
                                                        .font_size(11.0)
                                                        .color(Color::from_rgb(140, 230, 170)),
                                                ),
                                        ),
                                )
                                .child(
                                    label()
                                        .text(ext.id)
                                        .font_size(11.0)
                                        .color(Color::from_rgb(100, 100, 115)),
                                ),
                        )
                        .child(
                            label()
                                .text(ext.description)
                                .font_size(12.0)
                                .color(Color::from_rgb(160, 160, 170)),
                        )
                        .child(
                            rect()
                                .direction(Direction::horizontal())
                                .spacing(12.0)
                                .cross_align(Alignment::Center)
                                .child(
                                    label()
                                        .text(format!("Author: {}", ext.author))
                                        .font_size(11.0)
                                        .color(Color::from_rgb(120, 120, 130)),
                                )
                                .child(
                                    label()
                                        .text(format!("API Version: {}", ext.api_version))
                                        .font_size(11.0)
                                        .color(Color::from_rgb(120, 120, 130)),
                                ),
                        )
                        .into()
                }),
            )
            .into()
    };

    rect()
        .width(Size::fill())
        .direction(Direction::vertical())
        .spacing(14.0)
        .child(
            rect()
                .direction(Direction::horizontal())
                .main_align(Alignment::SpaceBetween)
                .cross_align(Alignment::Center)
                .child(
                    rect()
                        .direction(Direction::vertical())
                        .spacing(4.0)
                        .child(
                            label()
                                .text("Installed Extensions")
                                .font_size(18.0)
                                .font_weight(FontWeight::BOLD)
                                .color(Color::WHITE),
                        )
                        .child(
                            label()
                                .text(format!(
                                    "{} extension{} currently active",
                                    count,
                                    if count == 1 { "" } else { "s" }
                                ))
                                .font_size(12.0)
                                .color(Color::from_rgb(140, 140, 150)),
                        ),
                )
                .child(
                    rect()
                        .background(Color::from_rgb(26, 26, 34))
                        .border(Border::new().width(1.0).fill(Color::from_rgb(44, 44, 56)))
                        .corner_radius(6.0)
                        .padding(Gaps::new(4.0, 10.0, 4.0, 10.0))
                        .child(
                            label()
                                .text(if is_portable {
                                    "Mode: Portable"
                                } else {
                                    "Mode: System Profile"
                                })
                                .font_size(11.0)
                                .font_weight(FontWeight::BOLD)
                                .color(if is_portable {
                                    Color::from_rgb(130, 200, 255)
                                } else {
                                    Color::from_rgb(220, 180, 100)
                                }),
                        ),
                ),
        )
        .child(
            rect()
                .background(Color::from_rgb(22, 22, 28))
                .border(Border::new().width(1.0).fill(Color::from_rgb(38, 38, 46)))
                .corner_radius(8.0)
                .padding(Gaps::new(10.0, 14.0, 10.0, 14.0))
                .direction(Direction::horizontal())
                .main_align(Alignment::SpaceBetween)
                .cross_align(Alignment::Center)
                .child(
                    rect()
                        .direction(Direction::vertical())
                        .spacing(2.0)
                        .child(
                            label()
                                .text("Discovery Directory")
                                .font_size(12.0)
                                .font_weight(FontWeight::BOLD)
                                .color(Color::from_rgb(180, 180, 190)),
                        )
                        .child(
                            label()
                                .text(ext_dir)
                                .font_size(11.0)
                                .color(Color::from_rgb(110, 110, 125)),
                        ),
                ),
        )
        .child(
            rect()
                .width(Size::fill())
                .background(Color::from_rgb(20, 24, 32))
                .border(Border::new().width(1.5).fill(Color::from_rgb(45, 65, 95)))
                .corner_radius(8.0)
                .padding(Gaps::new_all(16.0))
                .direction(Direction::vertical())
                .cross_align(Alignment::Center)
                .spacing(4.0)
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
                                                "Successfully installed {}",
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
                                "Please drop an .opx bundle or native dynamic library (.dll/.so/.dylib)"
                                    .to_string(),
                                false,
                            )));
                        }
                    }
                })
                .child(
                    label()
                        .text("Drag & drop .opx packages or dynamic libraries here to install")
                        .font_size(13.0)
                        .font_weight(FontWeight::BOLD)
                        .color(Color::from_rgb(200, 200, 220)),
                )
                .child(
                    label()
                        .text(
                            "Extensions activate instantly and are saved to the extensions directory.",
                        )
                        .font_size(11.0)
                        .color(Color::from_rgb(130, 130, 140)),
                ),
        )
        .child(status_banner_element)
        .child(extensions_list_element)
        .into()
}

fn build_shortcuts_pane() -> Element {
    let shortcuts = [
        ("O", "Open Image Picker"),
        ("+ / =", "Zoom In"),
        ("- / _", "Zoom Out"),
        ("0", "100% Original Size (1:1)"),
        ("F", "Fit Image to Window"),
        ("Escape", "Clear Loaded Image"),
        ("S", "Toggle Settings Window"),
        ("Drag & Drop", "Load Image from Desktop or File Manager"),
    ];

    rect()
        .width(Size::fill())
        .direction(Direction::vertical())
        .spacing(14.0)
        .child(
            label()
                .text("Keyboard Shortcuts")
                .font_size(18.0)
                .font_weight(FontWeight::BOLD)
                .color(Color::WHITE),
        )
        .child(
            rect()
                .direction(Direction::vertical())
                .spacing(8.0)
                .children(shortcuts.into_iter().map(|(key, desc)| {
                    rect()
                        .background(Color::from_rgb(22, 22, 28))
                        .border(Border::new().width(1.0).fill(Color::from_rgb(38, 38, 46)))
                        .corner_radius(8.0)
                        .padding(Gaps::new(10.0, 14.0, 10.0, 14.0))
                        .direction(Direction::horizontal())
                        .main_align(Alignment::SpaceBetween)
                        .cross_align(Alignment::Center)
                        .child(
                            label()
                                .text(desc)
                                .font_size(13.0)
                                .color(Color::from_rgb(210, 210, 220)),
                        )
                        .child(
                            rect()
                                .background(Color::from_rgb(36, 36, 44))
                                .border(
                                    Border::new().width(1.0).fill(Color::from_rgb(52, 52, 64)),
                                )
                                .corner_radius(6.0)
                                .padding(Gaps::new(4.0, 8.0, 4.0, 8.0))
                                .child(
                                    label()
                                        .text(key)
                                        .font_size(12.0)
                                        .font_weight(FontWeight::BOLD)
                                        .color(Color::from_rgb(180, 210, 255)),
                                ),
                        )
                        .into()
                })),
        )
        .into()
}

fn build_about_pane() -> Element {
    rect()
        .width(Size::fill())
        .direction(Direction::vertical())
        .spacing(14.0)
        .child(
            label()
                .text("About Opsis")
                .font_size(18.0)
                .font_weight(FontWeight::BOLD)
                .color(Color::WHITE),
        )
        .child(
            rect()
                .background(Color::from_rgb(22, 22, 28))
                .border(Border::new().width(1.0).fill(Color::from_rgb(38, 38, 46)))
                .corner_radius(10.0)
                .padding(Gaps::new_all(18.0))
                .direction(Direction::vertical())
                .spacing(10.0)
                .child(
                    label()
                        .text("Opsis Image Viewer")
                        .font_size(15.0)
                        .font_weight(FontWeight::BOLD)
                        .color(Color::WHITE),
                )
                .child(
                    label()
                        .text("Ultra-lightweight, portable, high-performance image viewer built with Rust, Freya, and Skia.")
                        .font_size(12.0)
                        .color(Color::from_rgb(160, 160, 175)),
                )
                .child(
                    label()
                        .text("Architecture: Pure Microkernel Extension-First with Dedicated Window Host")
                        .font_size(11.0)
                        .color(Color::from_rgb(120, 180, 240)),
                )
                .child(
                    label()
                        .text("Version: 0.1.0 • License: MIT")
                        .font_size(11.0)
                        .color(Color::from_rgb(110, 110, 120)),
                ),
        )
        .into()
}
