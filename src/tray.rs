use crate::events::{AppEvent, EventSender};
use anyhow::Result;
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

pub struct TrayState {
    pub _tray_icon: TrayIcon,
    pub show_hide_id: MenuId,
    pub start_pause_id: MenuId,
    pub reset_id: MenuId,
    pub always_on_top_id: MenuId,
    pub quit_id: MenuId,
}

/// Build a 16x16 RGBA icon procedurally (simple colored circle).
fn generate_tray_icon() -> Icon {
    let size = 16u32;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let cx = (x as f32 - size as f32 / 2.0 + 0.5).abs();
            let cy = (y as f32 - size as f32 / 2.0 + 0.5).abs();
            let dist = (cx * cx + cy * cy).sqrt();
            if dist < size as f32 / 2.0 - 1.0 {
                rgba.extend_from_slice(&[80, 200, 120, 255]); // green
            } else {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }
    Icon::from_rgba(rgba, size, size).expect("Failed to create tray icon")
}

pub fn build_tray() -> Result<TrayState> {
    let menu = Menu::new();

    let show_hide = MenuItem::new("Show/Hide", true, None);
    let start_pause = MenuItem::new("Start/Pause", true, None);
    let reset = MenuItem::new("Reset", true, None);
    let always_on_top = MenuItem::new("Toggle Always On Top", true, None);
    let quit = MenuItem::new("Quit", true, None);

    let show_hide_id = show_hide.id().clone();
    let start_pause_id = start_pause.id().clone();
    let reset_id = reset.id().clone();
    let always_on_top_id = always_on_top.id().clone();
    let quit_id = quit.id().clone();

    menu.append(&show_hide)?;
    menu.append(&start_pause)?;
    menu.append(&reset)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&always_on_top)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&quit)?;

    let icon = generate_tray_icon();
    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("WorkTimer")
        .with_icon(icon)
        .build()?;

    Ok(TrayState {
        _tray_icon: tray_icon,
        show_hide_id,
        start_pause_id,
        reset_id,
        always_on_top_id,
        quit_id,
    })
}

/// Spawn a thread that polls tray menu events and forwards them as AppEvents.
pub fn spawn_tray_event_thread(
    tray: &TrayState,
    tx: EventSender,
    ctx: egui::Context,
) {
    let show_hide_id = tray.show_hide_id.clone();
    let start_pause_id = tray.start_pause_id.clone();
    let reset_id = tray.reset_id.clone();
    let always_on_top_id = tray.always_on_top_id.clone();
    let quit_id = tray.quit_id.clone();

    std::thread::Builder::new()
        .name("tray-events".into())
        .spawn(move || {
            let receiver = MenuEvent::receiver();
            loop {
                if let Ok(event) = receiver.recv() {
                    let id = event.id().clone();
                    let app_event = if id == show_hide_id {
                        AppEvent::ToggleVisibility
                    } else if id == start_pause_id {
                        AppEvent::ToggleStartPause
                    } else if id == reset_id {
                        AppEvent::Reset
                    } else if id == always_on_top_id {
                        AppEvent::ToggleAlwaysOnTop
                    } else if id == quit_id {
                        AppEvent::Quit
                    } else {
                        continue;
                    };
                    let _ = tx.send(app_event);
                    ctx.request_repaint();
                }
            }
        })
        .expect("Failed to spawn tray event thread");
}
