#![windows_subsystem = "windows"]

mod app;
mod audio;
mod config;
mod events;
mod hotkeys;
mod timer;
mod tray;
mod ui;

use anyhow::Result;
use config::AppConfig;
use egui::Vec2;

fn main() -> Result<()> {
    let config = AppConfig::load().unwrap_or_else(|e| {
        eprintln!("Config load error: {e}. Using defaults.");
        AppConfig::default()
    });

    let (event_tx, event_rx) = events::create_channel();

    // Build tray icon (must happen on main thread before event loop)
    let tray_state = match tray::build_tray() {
        Ok(t) => Some(t),
        Err(e) => {
            eprintln!("Tray init failed: {e}. Running without tray.");
            None
        }
    };

    // Register global hotkeys (must happen on main thread)
    let hotkey_state = match hotkeys::register_hotkeys() {
        Ok(h) => Some(h),
        Err(e) => {
            eprintln!("Hotkey registration failed: {e}. Running without hotkeys.");
            None
        }
    };

    // Window configuration
    let initial_size = if config.compact_mode {
        Vec2::new(160.0, 210.0)
    } else {
        Vec2::new(220.0, 280.0)
    };

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("WorkTimer")
        .with_inner_size(initial_size)
        .with_min_inner_size(Vec2::new(140.0, 180.0))
        .with_decorations(false)
        .with_transparent(true);

    if config.always_on_top {
        viewport = viewport.with_always_on_top();
    }

    if config.remember_window_position {
        if let (Some(x), Some(y)) = (config.window_x, config.window_y) {
            viewport = viewport.with_position(egui::pos2(x, y));
        }
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "WorkTimer",
        native_options,
        Box::new(move |cc| {
            Ok(Box::new(app::WorkTimerApp::new(
                cc,
                config,
                event_tx,
                event_rx,
                tray_state,
                hotkey_state,
            )))
        }),
    )
    .map_err(|e| anyhow::anyhow!("eframe error: {e}"))?;

    Ok(())
}
