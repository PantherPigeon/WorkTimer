use crossbeam_channel::{Receiver, Sender};

/// Events sent from tray menu, global hotkeys, or internal logic to the main app loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEvent {
    ToggleStartPause,
    Reset,
    Skip,
    ToggleVisibility,
    ToggleAlwaysOnTop,
    ShowWindow,
    HideWindow,
    Quit,
}

pub type EventSender = Sender<AppEvent>;
pub type EventReceiver = Receiver<AppEvent>;

pub fn create_channel() -> (EventSender, EventReceiver) {
    crossbeam_channel::unbounded()
}
