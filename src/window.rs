use crate::canvas::{canvas_view, CanvasState};
use crate::file_io::{load_image, pick_image_file};
use crate::manager::ExtensionManager;
use freya::prelude::*;
use opsis_extension_api::{InputContext, InputEvent, OverlayContext, ViewportContext};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

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
            .with_background(Color::from_rgb(18, 18, 20))
            .with_on_close(|_ctx, _window_id| {
                std::process::exit(0);
            }),
        ),
    );
}

fn app(path: Option<PathBuf>, ext_mgr: Arc<Mutex<ExtensionManager>>) -> impl IntoElement {
    let window_size = (800.0, 600.0);
    let mut canvas_state = use_state(CanvasState::default);

    // Load initial image if provided via CLI args
    use_hook(|| {
        if let Some(ref initial_path) = path {
            if let Ok(loaded) = load_image(initial_path) {
                canvas_state.with_mut(|mut st| st.set_image(loaded, window_size));
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
                            (builder)(trigger_redraw)
                        })
                        .with_title(title_static)
                        .with_size(size.0, size.1)
                        .with_background(Color::from_rgb(18, 18, 20)),
                    )
                    .await;
            });
        },
    );

    let overlay_ctx = OverlayContext {
        image_path: current_image_path.clone(),
        window_size,
        extensions_dir: extensions_dir.clone(),
        installed_extensions: installed_extensions.clone(),
        launch_window: Some(Arc::clone(&launch_window)),
    };

    let input_ctx = InputContext {
        image_path: current_image_path,
        window_size,
        extensions_dir: extensions_dir.clone(),
        installed_extensions: installed_extensions.clone(),
        launch_window: Some(launch_window),
    };

    let mut custom_viewport = None;
    let mut overlays = Vec::new();

    if let Ok(manager) = ext_mgr.lock() {
        let viewport_ctx = ViewportContext {
            image_path: canvas_state
                .read()
                .image
                .as_ref()
                .map(|img| img.metadata.path.clone()),
            image_bytes: None,
            window_size,
        };

        custom_viewport = manager.render_viewport(&viewport_ctx);
        overlays = manager.render_overlays(&overlay_ctx);
    }

    let mut root = rect()
        .width(Size::fill())
        .height(Size::fill())
        .background(Color::from_rgb(18, 18, 20));

    // Core Canvas: use extension viewport if an extension provided one, otherwise use core 2D canvas with post-processing filters
    let main_content = if let Some(viewport_element) = custom_viewport {
        viewport_element
    } else {
        canvas_view(canvas_state, window_size, Some(&ext_mgr))
    };

    root = root.child(main_content);

    // Layer active extension overlays
    for overlay in overlays {
        root = root.child(overlay);
    }

    let mut ext_redraw_trigger = use_state(|| 0usize);
    let _ = *ext_redraw_trigger.read();

    // Global Input Handling
    let ext_mgr_for_key = Arc::clone(&ext_mgr);

    root = root.on_global_key_down(move |e: Event<KeyboardEventData>| {
        let key_str = match &e.key {
            Key::Character(c) => c.clone(),
            Key::Named(named) => format!("{named:?}"),
        };

        // First give active extensions a chance to intercept input
        let mut handled = false;
        if let Ok(mut manager) = ext_mgr_for_key.lock() {
            if manager.dispatch_input(&InputEvent::KeyDown(key_str.clone()), &input_ctx)
                == opsis_extension_api::EventAction::Handled
            {
                handled = true;
                ext_redraw_trigger.with_mut(|mut count| *count = count.wrapping_add(1));
            }
        }

        if !handled {
            match key_str.as_str() {
                "o" | "O" => {
                    if let Some(path) = pick_image_file() {
                        match load_image(&path) {
                            Ok(img) => {
                                canvas_state.with_mut(|mut st| st.set_image(img, window_size))
                            }
                            Err(err) => canvas_state.with_mut(|mut st| st.set_error(err)),
                        }
                    }
                }
                "+" | "=" => {
                    if canvas_state.read().image.is_some() {
                        canvas_state.with_mut(|mut st| st.zoom_in());
                    }
                }
                "-" | "_" => {
                    if canvas_state.read().image.is_some() {
                        canvas_state.with_mut(|mut st| st.zoom_out());
                    }
                }
                "0" => {
                    if canvas_state.read().image.is_some() {
                        canvas_state.with_mut(|mut st| st.reset_zoom());
                    }
                }
                "f" | "F" => {
                    if canvas_state.read().image.is_some() {
                        canvas_state.with_mut(|mut st| st.fit_to_window(window_size));
                    }
                }
                "Escape" => {
                    if canvas_state.read().image.is_some() {
                        canvas_state.with_mut(|mut st| st.clear_image());
                    }
                }
                "s" | "S" => {
                    crate::settings::open_settings_window(Arc::clone(&ext_mgr_for_key));
                }
                _ => {}
            }
        }
    });

    root
}
