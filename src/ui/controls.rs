use crate::timer::TimerState;
use egui::{Button, Color32, RichText, Ui, Vec2};

/// Action returned by the control buttons.
#[derive(Debug, Clone, Copy)]
pub enum ControlAction {
    ToggleStartPause,
    Reset,
    Skip,
}

/// Render the Start/Pause, Reset, Skip button row. Returns any clicked action.
pub fn render_controls(ui: &mut Ui, timer_state: TimerState) -> Option<ControlAction> {
    let mut action = None;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;

        // Start/Pause button
        let (label, color) = match timer_state {
            TimerState::Running => ("Pause", Color32::from_rgb(230, 180, 50)),
            TimerState::Paused => ("Resume", Color32::from_rgb(80, 200, 120)),
            TimerState::Completed => ("Next", Color32::from_rgb(80, 200, 120)),
            TimerState::Idle => ("Start", Color32::from_rgb(80, 200, 120)),
        };
        let btn = Button::new(RichText::new(label).color(Color32::WHITE).size(14.0))
            .fill(color)
            .min_size(Vec2::new(64.0, 28.0));
        if ui.add(btn).clicked() {
            action = Some(ControlAction::ToggleStartPause);
        }

        // Reset button
        let btn = Button::new(RichText::new("Reset").color(Color32::WHITE).size(14.0))
            .fill(Color32::from_rgb(120, 120, 130))
            .min_size(Vec2::new(56.0, 28.0));
        if ui.add(btn).clicked() {
            action = Some(ControlAction::Reset);
        }

        // Skip button
        let btn = Button::new(RichText::new("Skip").color(Color32::WHITE).size(14.0))
            .fill(Color32::from_rgb(100, 100, 110))
            .min_size(Vec2::new(50.0, 28.0));
        if ui.add(btn).clicked() {
            action = Some(ControlAction::Skip);
        }
    });

    action
}
