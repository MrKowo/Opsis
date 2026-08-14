use freya::prelude::*;
use opsis_extension_api::ExtensionManifest;
use std::path::PathBuf;

/// Spawns the native floating Settings & Extensions window.
pub fn open_settings_window(extensions: Vec<ExtensionManifest>, ext_dir: PathBuf) {
    let ext_dir_str = ext_dir.display().to_string();

    spawn(async move {
        let _ = Platform::get()
            .launch_window(
                WindowConfig::new(move || {
                    let extensions = extensions.clone();
                    let ext_dir_str = ext_dir_str.clone();
                    settings_window_view(extensions, ext_dir_str)
                })
                .with_title("Opsis - Settings")
                .with_size(720.0, 520.0)
                .with_background(Color::from_rgb(14, 14, 18)),
            )
            .await;
    });
}

fn settings_window_view(extensions: Vec<ExtensionManifest>, ext_dir: String) -> Element {
    let active_tab = use_state(|| 2usize); // Default to Extensions tab
    let current_tab = *active_tab.read();

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .background(Color::from_rgb(14, 14, 18))
        .direction(Direction::horizontal())
        .children([
            // Left Sidebar: Vertical Tab Navigation Rail
            build_sidebar(current_tab, active_tab),
            // Right Area: Active Tab Content Pane
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .padding(Gaps::new_all(24.0))
                .direction(Direction::vertical())
                .spacing(16.0)
                .child(match current_tab {
                    0 => build_general_pane(),
                    1 => build_appearance_pane(),
                    2 => build_extensions_pane(extensions, ext_dir),
                    3 => build_shortcuts_pane(),
                    _ => build_about_pane(),
                })
                .into(),
        ])
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

// ---------------------------------------------------------------------------
// Tab 0: General
// ---------------------------------------------------------------------------
fn build_general_pane() -> Element {
    rect()
        .width(Size::fill())
        .direction(Direction::vertical())
        .spacing(14.0)
        .children([
            pane_header(
                "General Preferences",
                "Manage general application configuration",
            ),
            rect()
                .padding(Gaps::new_all(24.0))
                .direction(Direction::vertical())
                .spacing(6.0)
                .cross_align(Alignment::Center)
                .child(
                    label()
                        .text("No general configuration options available.")
                        .font_size(13.0)
                        .color(Color::from_rgb(140, 140, 150)),
                )
                .into(),
        ])
        .into()
}

// ---------------------------------------------------------------------------
// Tab 1: Appearance
// ---------------------------------------------------------------------------
fn build_appearance_pane() -> Element {
    rect()
        .width(Size::fill())
        .direction(Direction::vertical())
        .spacing(14.0)
        .children([
            pane_header(
                "Appearance",
                "Customize interface theme and visual presentation",
            ),
            rect()
                .padding(Gaps::new_all(24.0))
                .direction(Direction::vertical())
                .spacing(6.0)
                .cross_align(Alignment::Center)
                .child(
                    label()
                        .text("No appearance options available.")
                        .font_size(13.0)
                        .color(Color::from_rgb(140, 140, 150)),
                )
                .into(),
        ])
        .into()
}

// ---------------------------------------------------------------------------
// Tab 2: Extensions (Live dynamic extension inspector)
// ---------------------------------------------------------------------------
fn build_extensions_pane(extensions: Vec<ExtensionManifest>, ext_dir: String) -> Element {
    let count = extensions.len();

    rect()
        .width(Size::fill())
        .direction(Direction::vertical())
        .spacing(14.0)
        .children([
            rect()
                .direction(Direction::horizontal())
                .cross_align(Alignment::Center)
                .spacing(10.0)
                .children([
                    label()
                        .text("Installed Extensions")
                        .font_size(18.0)
                        .font_weight(FontWeight::BOLD)
                        .color(Color::WHITE)
                        .into(),
                    rect()
                        .background(Color::from_rgb(38, 55, 45))
                        .corner_radius(12.0)
                        .padding(Gaps::new(3.0, 8.0, 3.0, 8.0))
                        .child(
                            label()
                                .text(format!("{count} Active"))
                                .font_size(11.0)
                                .color(Color::from_rgb(140, 220, 160)),
                        )
                        .into(),
                ])
                .into(),
            label()
                .text(format!("Directory: {ext_dir}"))
                .font_size(11.0)
                .color(Color::from_rgb(120, 120, 130))
                .into(),
            if extensions.is_empty() {
                rect()
                    .padding(Gaps::new_all(24.0))
                    .direction(Direction::vertical())
                    .spacing(6.0)
                    .cross_align(Alignment::Center)
                    .children([
                        label()
                            .text("No external extensions installed.")
                            .font_size(14.0)
                            .color(Color::from_rgb(180, 180, 190))
                            .into(),
                        label()
                            .text("Place .opx bundles or native dynamic libraries in the extensions directory.")
                            .font_size(12.0)
                            .color(Color::from_rgb(110, 110, 120))
                            .into(),
                    ])
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
                                .spacing(6.0)
                                .children([
                                    rect()
                                        .direction(Direction::horizontal())
                                        .cross_align(Alignment::Center)
                                        .spacing(8.0)
                                        .children([
                                            label()
                                                .text(ext.name)
                                                .font_size(14.0)
                                                .font_weight(FontWeight::BOLD)
                                                .color(Color::WHITE)
                                                .into(),
                                            rect()
                                                .background(Color::from_rgb(40, 48, 60))
                                                .corner_radius(4.0)
                                                .padding(Gaps::new(2.0, 6.0, 2.0, 6.0))
                                                .child(
                                                    label()
                                                        .text(format!("v{}", ext.version))
                                                        .font_size(10.0)
                                                        .color(Color::from_rgb(160, 190, 240)),
                                                )
                                                .into(),
                                        ])
                                        .into(),
                                    label()
                                        .text(format!("Author: {}", ext.author))
                                        .font_size(11.0)
                                        .color(Color::from_rgb(140, 140, 150))
                                        .into(),
                                    label()
                                        .text(ext.description)
                                        .font_size(12.0)
                                        .color(Color::from_rgb(190, 190, 200))
                                        .into(),
                                ])
                                .into()
                        }),
                    )
                    .into()
            },
        ])
        .into()
}

// ---------------------------------------------------------------------------
// Tab 3: Shortcuts (Real active keybindings)
// ---------------------------------------------------------------------------
fn build_shortcuts_pane() -> Element {
    let shortcuts = [
        ("O", "Open image file dialog"),
        ("+ / =", "Zoom in by 25%"),
        ("- / _", "Zoom out by 25%"),
        ("0", "Reset zoom to 100% (1:1 pixel scale)"),
        ("F", "Fit image to current window"),
        ("Escape", "Clear loaded image"),
        ("S", "Open Settings & Extensions window"),
    ];

    rect()
        .width(Size::fill())
        .direction(Direction::vertical())
        .spacing(14.0)
        .children([
            pane_header("Keyboard Shortcuts", "Standard keybindings configured across Opsis"),
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
                        .cross_align(Alignment::Center)
                        .spacing(12.0)
                        .children([
                            rect()
                                .background(Color::from_rgb(34, 34, 42))
                                .border(Border::new().width(1.0).fill(Color::from_rgb(55, 55, 65)))
                                .corner_radius(4.0)
                                .padding(Gaps::new(3.0, 8.0, 3.0, 8.0))
                                .child(
                                    label()
                                        .text(key)
                                        .font_size(12.0)
                                        .font_weight(FontWeight::BOLD)
                                        .color(Color::from_rgb(220, 220, 230)),
                                )
                                .into(),
                            label()
                                .text(desc)
                                .font_size(12.0)
                                .color(Color::from_rgb(180, 180, 190))
                                .into(),
                        ])
                        .into()
                }))
                .into(),
        ])
        .into()
}

// ---------------------------------------------------------------------------
// Tab 4: About
// ---------------------------------------------------------------------------
fn build_about_pane() -> Element {
    rect()
        .width(Size::fill())
        .direction(Direction::vertical())
        .spacing(14.0)
        .children([
            pane_header("About Opsis", "Project details and architecture"),
            rect()
                .background(Color::from_rgb(22, 22, 28))
                .border(Border::new().width(1.0).fill(Color::from_rgb(38, 38, 46)))
                .corner_radius(10.0)
                .padding(Gaps::new_all(16.0))
                .direction(Direction::vertical())
                .spacing(8.0)
                .children([
                    label()
                        .text("Opsis v0.1.0")
                        .font_size(16.0)
                        .font_weight(FontWeight::BOLD)
                        .color(Color::WHITE)
                        .into(),
                    label()
                        .text("Ultra-minimal, hyper-fast image viewer with a Microkernel Extension-First Architecture.")
                        .font_size(12.0)
                        .color(Color::from_rgb(180, 180, 190))
                        .into(),
                    rect()
                        .direction(Direction::vertical())
                        .spacing(4.0)
                        .padding(Gaps::new(8.0, 0.0, 0.0, 0.0))
                        .children([
                            label().text("Core: Rust (Zero-Cost Abstractions)").font_size(11.0).color(Color::from_rgb(140, 140, 150)).into(),
                            label().text("UI Engine: Freya 0.4 (Skia 2D + Winit)").font_size(11.0).color(Color::from_rgb(140, 140, 150)).into(),
                            label().text("Plugin Format: Universal .opx / Native cdylib").font_size(11.0).color(Color::from_rgb(140, 140, 150)).into(),
                            label().text("License: MIT").font_size(11.0).color(Color::from_rgb(140, 140, 150)).into(),
                        ])
                        .into(),
                ])
                .into(),
        ])
        .into()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------
fn pane_header(title: &'static str, subtitle: &'static str) -> Element {
    rect()
        .direction(Direction::vertical())
        .spacing(4.0)
        .children([
            label()
                .text(title)
                .font_size(18.0)
                .font_weight(FontWeight::BOLD)
                .color(Color::WHITE)
                .into(),
            label()
                .text(subtitle)
                .font_size(12.0)
                .color(Color::from_rgb(130, 130, 140))
                .into(),
        ])
        .into()
}
