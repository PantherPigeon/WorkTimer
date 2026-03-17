use crate::events::{AppEvent, EventSender};
use anyhow::Result;
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};

pub struct HotkeyState {
    pub _manager: GlobalHotKeyManager,
    pub start_pause_id: u32,
    pub reset_id: u32,
    pub visibility_id: u32,
    pub always_on_top_id: u32,
}

pub fn register_hotkeys() -> Result<HotkeyState> {
    let manager = GlobalHotKeyManager::new()?;

    let mods = Modifiers::CONTROL | Modifiers::ALT;

    let start_pause = HotKey::new(Some(mods), Code::KeyS);
    let reset = HotKey::new(Some(mods), Code::KeyR);
    let visibility = HotKey::new(Some(mods), Code::KeyF);
    let always_on_top = HotKey::new(Some(mods), Code::KeyT);

    let start_pause_id = start_pause.id();
    let reset_id = reset.id();
    let visibility_id = visibility.id();
    let always_on_top_id = always_on_top.id();

    manager.register(start_pause)?;
    manager.register(reset)?;
    manager.register(visibility)?;
    manager.register(always_on_top)?;

    Ok(HotkeyState {
        _manager: manager,
        start_pause_id,
        reset_id,
        visibility_id,
        always_on_top_id,
    })
}

/// Spawn a thread that polls global hotkey events and forwards them as AppEvents.
pub fn spawn_hotkey_event_thread(
    hotkeys: &HotkeyState,
    tx: EventSender,
    ctx: egui::Context,
) {
    let start_pause_id = hotkeys.start_pause_id;
    let reset_id = hotkeys.reset_id;
    let visibility_id = hotkeys.visibility_id;
    let always_on_top_id = hotkeys.always_on_top_id;

    std::thread::Builder::new()
        .name("hotkey-events".into())
        .spawn(move || {
            let receiver = GlobalHotKeyEvent::receiver();
            loop {
                if let Ok(event) = receiver.recv() {
                    let app_event = if event.id == start_pause_id {
                        AppEvent::ToggleStartPause
                    } else if event.id == reset_id {
                        AppEvent::Reset
                    } else if event.id == visibility_id {
                        AppEvent::ToggleVisibility
                    } else if event.id == always_on_top_id {
                        AppEvent::ToggleAlwaysOnTop
                    } else {
                        continue;
                    };
                    let _ = tx.send(app_event);
                    ctx.request_repaint();
                }
            }
        })
        .expect("Failed to spawn hotkey event thread");
}
