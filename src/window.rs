use crate::canvas::{canvas_view, CanvasState};
use crate::file_io::{get_adjacent_image_path, load_image, pick_image_file};
use crate::hotkeys::{CoreAction, KeyDispatchResult};
use crate::manager::ExtensionManager;
use crate::ui::use_init_opsis_theme;
use freya::prelude::*;
use opsis_extension_api::{InputContext, OverlayContext, ViewportContext};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

const APP_ICON: &[u8] = include_bytes!("../assets/icon.png");

/// Launch the Opsis window host and Freya UI runtime.
pub fn run(path: Option<PathBuf>, extension_manager: Arc<Mutex<ExtensionManager>>) {
    launch(
        LaunchConfig::new().with_window(
            WindowConfig::new(move || {
                let path = path.clone();
                let ext_mgr = Arc::clone(&extension_manager);
                app(path, ext_mgr)
            })
            .with_title("Opsis")
            .with_icon(LaunchConfig::window_icon(APP_ICON))
            .with_size(800.0, 600.0)
            .with_transparency(true)
            .with_background(Color::TRANSPARENT)
            .with_on_close(|_ctx, _window_id| {
                std::process::exit(0);
            }),
        ),
    );
}

/// Dynamically resize the OS window to match the image's aspect ratio scaled to 75% of screen size.
pub fn resize_window_to_image_aspect(img_dims: (u32, u32)) {
    Platform::get().with_window(None, move |w| {
        let monitor = w.current_monitor().or_else(|| w.primary_monitor());
        let scale_factor = w.scale_factor();
        let screen_size = if let Some(m) = monitor {
            let phys = m.size();
            (
                (phys.width as f64) / scale_factor,
                (phys.height as f64) / scale_factor,
            )
        } else {
            (1920.0, 1080.0)
        };

        let (tw, th) = crate::canvas::calculate_target_window_size(img_dims, screen_size);
        crate::log_window!(
            "Auto-sizing window for image {}x{} -> target {:.0}x{:.0} px (screen {:.0}x{:.0})",
            img_dims.0,
            img_dims.1,
            tw,
            th,
            screen_size.0,
            screen_size.1
        );
        let _ = w.request_inner_size(freya::winit::dpi::LogicalSize::new(tw, th));
    });
}

fn app(path: Option<PathBuf>, ext_mgr: Arc<Mutex<ExtensionManager>>) -> impl IntoElement {
    use_init_opsis_theme();

    let mut window_size = use_state(|| (800.0, 600.0));
    let current_window_size = *window_size.read();
    let mut canvas_state = use_state(CanvasState::default);

    // Extension reload trigger for dynamic updates from background loader
    let mut ext_version = use_state(|| 0usize);
    let _ = *ext_version.read();

    let (ext_tx, ext_rx) = use_hook(async_channel::unbounded::<()>);

    // Start background extension loading concurrently with window/image display
    use_hook(|| {
        let ext_rx = ext_rx.clone();
        spawn(async move {
            while ext_rx.recv().await.is_ok() {
                ext_version.with_mut(|mut v| *v = v.wrapping_add(1));
            }
        });

        let ext_tx_clone = ext_tx.clone();
        let trigger_update: Arc<dyn Fn() + Send + Sync + 'static> = Arc::new(move || {
            let _ = ext_tx_clone.try_send(());
        });

        ExtensionManager::load_in_background(Arc::clone(&ext_mgr), Some(trigger_update));
    });

    let ext_tx_settings = ext_tx.clone();
    let trigger_settings_change: Arc<dyn Fn() + Send + Sync + 'static> = Arc::new(move || {
        let _ = ext_tx_settings.try_send(());
    });

    let acrylic_enabled = if let Ok(manager) = ext_mgr.lock() {
        manager.settings.acrylic_background
    } else {
        false
    };

    // Apply and synchronize native OS acrylic backdrop and title bar color with settings
    Platform::get().with_window(None, move |w| {
        #[cfg(target_os = "windows")]
        {
            use freya::winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
            if let Ok(handle) = w.window_handle() {
                if let RawWindowHandle::Win32(win32_handle) = handle.as_raw() {
                    crate::ui::acrylic::apply_windows_acrylic(win32_handle.hwnd.get(), acrylic_enabled);
                }
            }
        }
    });

    // Load initial image if provided via CLI args
    use_hook(|| {
        if let Some(ref initial_path) = path {
            crate::log_io!("Loading startup CLI image: '{}'", initial_path.display());
            match load_image(initial_path) {
                Ok(loaded) => {
                    resize_window_to_image_aspect(loaded.metadata.dimensions);
                    canvas_state.with_mut(|mut st| st.set_image(loaded, current_window_size));
                }
                Err(err) => canvas_state.with_mut(|mut st| st.set_error_for_path(err, initial_path.clone())),
            }
        }
    });

    let current_image_path = canvas_state
        .read()
        .image
        .as_ref()
        .map(|img| img.metadata.path.clone());

    let (installed_extensions, extensions_dir) = if let Ok(manager) = ext_mgr.lock() {
        (
            manager
                .loaded_extensions
                .iter()
                .map(|ext| ext.manifest.clone())
                .collect(),
            manager.extensions_dir.clone(),
        )
    } else {
        (Vec::new(), PathBuf::from("extensions"))
    };

    let launch_window: opsis_extension_api::WindowLauncherFn = Arc::new(
        move |title: String, size: (f64, f64), builder: opsis_extension_api::WindowBuilderFn| {
            let title_static: &'static str = Box::leak(title.into_boxed_str());
            spawn(async move {
                let _ = Platform::get()
                    .launch_window(
                        WindowConfig::new(move || {
                            use_init_opsis_theme();

                            let mut trigger = use_state(|| false);
                            let _ = *trigger.read();

                            let (tx, rx) = use_hook(async_channel::unbounded::<()>);

                            use_hook(|| {
                                let rx = rx.clone();
                                spawn(async move {
                                    while rx.recv().await.is_ok() {
                                        trigger.toggle();
                                    }
                                });
                            });

                            let tx_clone = tx.clone();
                            let trigger_redraw: opsis_extension_api::RedrawTriggerFn =
                                Arc::new(move || {
                                    let _ = tx_clone.try_send(());
                                });
                            let view = (builder)(trigger_redraw);
                            rect()
                                .width(Size::fill())
                                .height(Size::fill())
                                .on_global_key_down(move |e: Event<KeyboardEventData>| {
                                    let key_str = match &e.key {
                                        Key::Character(c) => c.clone(),
                                        Key::Named(named) => format!("{named:?}"),
                                    };
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
                                .child(view)
                        })
                        .with_title(title_static)
                        .with_icon(LaunchConfig::window_icon(APP_ICON))
                        .with_size(size.0, size.1)
                        .with_background(Color::from_rgb(18, 18, 20)),
                    )
                    .await;
            });
        },
    );

    let overlay_ctx = OverlayContext {
        image_path: current_image_path.clone(),
        window_size: current_window_size,
        extensions_dir: extensions_dir.clone(),
        installed_extensions: installed_extensions.clone(),
        launch_window: Some(Arc::clone(&launch_window)),
    };

    let mut custom_viewport = None;
    let mut overlays = Vec::new();
    let mut ext_sidebar_titles = Vec::new();
    let mut ext_sidebar_elements = Vec::new();

    if let Ok(manager) = ext_mgr.lock() {
        let viewport_ctx = ViewportContext {
            image_path: canvas_state
                .read()
                .image
                .as_ref()
                .map(|img| img.metadata.path.clone()),
            image_bytes: None,
            window_size: current_window_size,
        };

        custom_viewport = manager.render_viewport(&viewport_ctx);
        overlays = manager.render_overlays(&overlay_ctx);

        for provider in &manager.registry.sidebar_tab_providers {
            ext_sidebar_titles.push(provider.tab_title());
            ext_sidebar_elements.push(provider.render_tab(&overlay_ctx));
        }
    }

    let mut show_sidebar = use_state(|| false);
    let active_sidebar_tab = use_state(|| 0usize);
    let mut zen_mode = use_state(|| false);

    let is_zen = *zen_mode.read();
    let is_sidebar_visible = !is_zen && *show_sidebar.read();
    let current_tab = *active_sidebar_tab.read();

    let image_meta = canvas_state
        .read()
        .image
        .as_ref()
        .map(|img| (img.metadata.clone(), canvas_state.read().zoom));

    // --- Collapsible N-Panel Sidebar ---
    let sidebar_panel = if is_sidebar_visible {
        let mut tab_buttons = Vec::new();
        let base_tabs = ["Details", "Tools", "Plugins"];

        for (idx, tab_name) in base_tabs.iter().enumerate() {
            let is_active = current_tab == idx;
            let mut active_tab_state = active_sidebar_tab;
            tab_buttons.push(
                rect()
                    .padding(Gaps::new(4.0, 8.0, 4.0, 8.0))
                    .background(if is_active {
                        crate::ui::Theme::accent_muted()
                    } else {
                        Color::TRANSPARENT
                    })
                    .border(Border::new().width(1.0).fill(if is_active {
                        crate::ui::Theme::accent_primary()
                    } else {
                        Color::TRANSPARENT
                    }))
                    .corner_radius(crate::ui::Theme::RADIUS_MD)
                    .on_press(move |_| {
                        active_tab_state.set(idx);
                    })
                    .child(
                        label()
                            .text(*tab_name)
                            .font_size(crate::ui::Theme::FONT_CAPTION)
                            .font_weight(if is_active {
                                FontWeight::BOLD
                            } else {
                                FontWeight::NORMAL
                            })
                            .color(if is_active {
                                crate::ui::Theme::accent_primary()
                            } else {
                                crate::ui::Theme::text_secondary()
                            }),
                    ),
            );
        }

        for (offset, ext_title) in ext_sidebar_titles.iter().enumerate() {
            let idx = base_tabs.len() + offset;
            let is_active = current_tab == idx;
            let mut active_tab_state = active_sidebar_tab;
            let title_text = ext_title.clone();
            tab_buttons.push(
                rect()
                    .padding(Gaps::new(4.0, 8.0, 4.0, 8.0))
                    .background(if is_active {
                        crate::ui::Theme::accent_muted()
                    } else {
                        Color::TRANSPARENT
                    })
                    .border(Border::new().width(1.0).fill(if is_active {
                        crate::ui::Theme::accent_primary()
                    } else {
                        Color::TRANSPARENT
                    }))
                    .corner_radius(crate::ui::Theme::RADIUS_MD)
                    .on_press(move |_| {
                        active_tab_state.set(idx);
                    })
                    .child(
                        label()
                            .text(title_text)
                            .font_size(crate::ui::Theme::FONT_CAPTION)
                            .font_weight(if is_active {
                                FontWeight::BOLD
                            } else {
                                FontWeight::NORMAL
                            })
                            .color(if is_active {
                                crate::ui::Theme::accent_primary()
                            } else {
                                crate::ui::Theme::text_secondary()
                            }),
                    ),
            );
        }

        let tabs_bar = rect()
            .width(Size::fill())
            .padding(Gaps::new(6.0, 8.0, 6.0, 8.0))
            .background(crate::ui::Theme::surface_panel())
            .border(
                Border::new()
                    .width(1.0)
                    .fill(crate::ui::Theme::border_subtle()),
            )
            .direction(Direction::horizontal())
            .spacing(4.0)
            .children(tab_buttons.into_iter().map(|b| b.into()));

        let panel_content = match current_tab {
            0 => {
                // Details Tab
                let mut col = rect()
                    .width(Size::fill())
                    .direction(Direction::vertical())
                    .child(crate::ui::section_header("Image Properties"));

                if let Some((ref meta, zoom)) = image_meta {
                    let (w, h) = meta.dimensions;
                    let megapixels = (w as f64 * h as f64) / 1_000_000.0;
                    col = col
                        .child(crate::ui::info_row("Filename", &meta.filename))
                        .child(crate::ui::info_row(
                            "Dimensions",
                            crate::file_io::format_dimensions((w, h)),
                        ))
                        .child(crate::ui::info_row(
                            "Megapixels",
                            format!("{megapixels:.2} MP"),
                        ))
                        .child(crate::ui::info_row(
                            "Aspect Ratio",
                            crate::file_io::format_aspect_ratio((w, h)),
                        ))
                        .child(crate::ui::info_row("Format", &meta.format_name))
                        .child(crate::ui::info_row(
                            "File Size",
                            crate::file_io::format_file_size(meta.file_size_bytes),
                        ))
                        .child(crate::ui::info_row("Zoom Scale", format!("{:.0}%", zoom * 100.0)))
                        .child(crate::ui::info_row(
                            "Directory",
                            meta.path
                                .parent()
                                .map(|p| p.display().to_string())
                                .unwrap_or_else(|| "-".to_string()),
                        ));
                } else {
                    col = col
                        .child(crate::ui::info_row("Status", "No Image Open"))
                        .child(crate::ui::info_row("Shortcut", "Press 'O' to open"));
                }
                col
            }
            1 => {
                // Tools Tab
                rect()
                    .width(Size::fill())
                    .direction(Direction::vertical())
                    .child(crate::ui::section_header("Quick Actions"))
                    .child(
                        rect()
                            .width(Size::fill())
                            .padding(Gaps::new(10.0, 10.0, 10.0, 10.0))
                            .direction(Direction::vertical())
                            .spacing(6.0)
                            .child(crate::ui::button_secondary("Open Image (O)", move |_| {
                                if let Some(path) = pick_image_file() {
                                    let win_size = *window_size.read();
                                    match load_image(&path) {
                                        Ok(img) => {
                                            canvas_state.with_mut(|mut st| st.set_image(img, win_size))
                                        }
                                        Err(err) => {
                                            canvas_state.with_mut(|mut st| st.set_error(err))
                                        }
                                    }
                                }
                            }))
                            .child(crate::ui::button_secondary("Fit Axis (H)", move |_| {
                                let has_image = canvas_state.read().image.is_some();
                                if has_image {
                                    let win_size = *window_size.read();
                                    canvas_state.with_mut(|mut st| st.toggle_fit_axis(win_size));
                                }
                            }))
                            .child(crate::ui::button_secondary("1:1 Pixel Scale (0)", move |_| {
                                let has_image = canvas_state.read().image.is_some();
                                if has_image {
                                    canvas_state.with_mut(|mut st| st.reset_zoom());
                                }
                            }))
                            .child(crate::ui::button_secondary("Clear Image (Esc)", move |_| {
                                let has_image = canvas_state.read().image.is_some();
                                if has_image {
                                    canvas_state.with_mut(|mut st| st.clear_image());
                                }
                            })),
                    )
            }
            2 => {
                // Plugins Tab
                let mut col = rect()
                    .width(Size::fill())
                    .direction(Direction::vertical())
                    .child(crate::ui::section_header("Active Extensions"));

                if installed_extensions.is_empty() {
                    col = col.child(crate::ui::info_row("Status", "No extensions loaded"));
                } else {
                    for ext in &installed_extensions {
                        col = col.child(crate::ui::info_row(
                            &ext.name,
                            format!("v{}", ext.version),
                        ));
                    }
                }
                col
            }
            _ => {
                let ext_idx = current_tab - base_tabs.len();
                if let Some(Some(elem)) = ext_sidebar_elements.into_iter().nth(ext_idx) {
                    rect().width(Size::fill()).child(elem)
                } else {
                    rect()
                        .width(Size::fill())
                        .child(crate::ui::info_row("Status", "Empty tab"))
                }
            }
        };

        rect()
            .width(Size::px(260.0))
            .height(Size::fill())
            .background(crate::ui::Theme::surface_panel())
            .border(
                Border::new()
                    .width(1.0)
                    .fill(crate::ui::Theme::border_subtle()),
            )
            .direction(Direction::vertical())
            .child(tabs_bar)
            .child(panel_content)
    } else {
        rect().width(Size::px(0.0)).height(Size::px(0.0))
    };

    let main_canvas = if let Some(viewport_element) = custom_viewport {
        viewport_element
    } else {
        canvas_view(canvas_state, current_window_size, Some(&ext_mgr))
    };

    let mut canvas_container = rect()
        .width(Size::fill())
        .height(Size::fill())
        .child(main_canvas);

    for overlay in overlays {
        canvas_container = canvas_container.child(overlay);
    }



    let mut root = rect()
        .width(Size::fill())
        .height(Size::fill())
        .background(crate::ui::Theme::canvas_background(acrylic_enabled))
        .direction(Direction::horizontal())
        .on_sized(move |e: Event<SizedEventData>| {
            let (w, h) = (e.area.size.width as f64, e.area.size.height as f64);
            let prev = *window_size.read();
            if w > 0.0 && h > 0.0 && (w, h) != prev {
                crate::log_window!("Main window resized to {:.0}x{:.0} px", w, h);
                window_size.set((w, h));
                canvas_state.with_mut(|mut st| {
                    if st.pan_offset == (0.0, 0.0) && !st.is_dragging {
                        st.fit_to_window((w, h));
                    }
                });
            }
        })
        .child(canvas_container)
        .child(sidebar_panel);

    let mut ext_redraw_trigger = use_state(|| 0usize);
    let _ = *ext_redraw_trigger.read();

    // Global Input Handling
    let ext_mgr_for_key = Arc::clone(&ext_mgr);
    let trigger_settings_key = Arc::clone(&trigger_settings_change);

    root = root
        .on_capture_global_pointer_press(move |_| {
            canvas_state.with_mut(|mut st| st.is_dragging = false);
        })
        .on_global_key_down(move |e: Event<KeyboardEventData>| {
        let key_str = match &e.key {
            Key::Character(c) => c.clone(),
            Key::Named(named) => format!("{named:?}"),
        };

        let dispatch_result = if let Ok(mut manager) = ext_mgr_for_key.lock() {
            let input_ctx = InputContext {
                image_path: current_image_path.clone(),
                window_size: *window_size.read(),
                extensions_dir: manager.extensions_dir.clone(),
                installed_extensions: manager
                    .loaded_extensions
                    .iter()
                    .map(|e| e.manifest.clone())
                    .collect(),
                launch_window: None,
            };
            manager.dispatch_key(&key_str, &input_ctx)
        } else {
            KeyDispatchResult::Pass
        };

        crate::log_input!("Key down: '{key_str}' -> Dispatch: {:?}", dispatch_result);

        match dispatch_result {
            KeyDispatchResult::Handled => {
                ext_redraw_trigger.with_mut(|mut count| *count = count.wrapping_add(1));
            }
            KeyDispatchResult::Core(CoreAction::OpenImage) => {
                if let Some(path) = pick_image_file() {
                    let win_size = *window_size.read();
                    match load_image(&path) {
                        Ok(img) => {
                            resize_window_to_image_aspect(img.metadata.dimensions);
                            canvas_state.with_mut(|mut st| st.set_image(img, win_size));
                        }
                        Err(err) => canvas_state.with_mut(|mut st| st.set_error_for_path(err, path)),
                    }
                }
            }
            KeyDispatchResult::Core(CoreAction::NextImage) => {
                let current_path = canvas_state.read().active_path();
                if let Some(path) = current_path {
                    if let Some(next_path) = get_adjacent_image_path(&path, true) {
                        let win_size = *window_size.read();
                        match load_image(&next_path) {
                            Ok(img) => canvas_state.with_mut(|mut st| st.set_image(img, win_size)),
                            Err(err) => canvas_state.with_mut(|mut st| st.set_error_for_path(err, next_path)),
                        }
                    }
                }
            }
            KeyDispatchResult::Core(CoreAction::PrevImage) => {
                let current_path = canvas_state.read().active_path();
                if let Some(path) = current_path {
                    if let Some(prev_path) = get_adjacent_image_path(&path, false) {
                        let win_size = *window_size.read();
                        match load_image(&prev_path) {
                            Ok(img) => canvas_state.with_mut(|mut st| st.set_image(img, win_size)),
                            Err(err) => canvas_state.with_mut(|mut st| st.set_error_for_path(err, prev_path)),
                        }
                    }
                }
            }
            KeyDispatchResult::Core(CoreAction::ZoomIn) => {
                let has_image = canvas_state.read().image.is_some();
                if has_image {
                    canvas_state.with_mut(|mut st| st.zoom_in());
                }
            }
            KeyDispatchResult::Core(CoreAction::ZoomOut) => {
                let has_image = canvas_state.read().image.is_some();
                if has_image {
                    canvas_state.with_mut(|mut st| st.zoom_out());
                }
            }
            KeyDispatchResult::Core(CoreAction::ResetZoom) => {
                let has_image = canvas_state.read().image.is_some();
                if has_image {
                    canvas_state.with_mut(|mut st| st.reset_zoom());
                }
            }
            KeyDispatchResult::Core(CoreAction::ToggleFitAxis) => {
                let has_image = canvas_state.read().image.is_some();
                if has_image {
                    let win_size = *window_size.read();
                    canvas_state.with_mut(|mut st| st.toggle_fit_axis(win_size));
                }
            }
            KeyDispatchResult::Core(CoreAction::ToggleMaximize) => {
                Platform::get().with_window(None, |w| {
                    let is_max = w.is_maximized();
                    w.set_maximized(!is_max);
                });
            }
            KeyDispatchResult::Core(CoreAction::ToggleSidebar) => {
                show_sidebar.toggle();
            }
            KeyDispatchResult::Core(CoreAction::ToggleZenMode) => {
                zen_mode.toggle();
            }
            KeyDispatchResult::Core(CoreAction::ClearImage) => {
                let has_image_or_err = canvas_state.read().image.is_some() || canvas_state.read().error_message.is_some();
                if has_image_or_err {
                    canvas_state.with_mut(|mut st| st.clear_image());
                }
            }
            KeyDispatchResult::Core(CoreAction::OpenSettings) => {
                crate::settings::open_settings_window(
                    Arc::clone(&ext_mgr_for_key),
                    Some(Arc::clone(&trigger_settings_key)),
                );
            }
            KeyDispatchResult::Core(CoreAction::CloseWindow) => {
                std::process::exit(0);
            }
            KeyDispatchResult::Pass => {}
        }
    });

    root
}
