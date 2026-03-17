use crate::audio::AudioPlayer;
use crate::config::AppConfig;
use crate::events::{AppEvent, EventReceiver, EventSender};
use crate::hotkeys::{self, HotkeyState};
use crate::timer::{SessionType, TimerEngine, TimerState};
use crate::tray::{self, TrayState};
use crate::ui;
use crate::ui::controls::ControlAction;
use egui::{Align, CentralPanel, Color32, Frame, Layout, Pos2, RichText, Vec2, ViewportCommand};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    Normal,
    Compact,
}

pub struct WorkTimerApp {
    timer: TimerEngine,
    config: AppConfig,
    event_rx: EventReceiver,
    event_tx: EventSender,
    audio: AudioPlayer,
    is_visible: bool,
    always_on_top: bool,
    display_mode: DisplayMode,
    _tray: Option<TrayState>,
    _hotkeys: Option<HotkeyState>,
    needs_session_transition: bool,
}

impl WorkTimerApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        config: AppConfig,
        event_tx: EventSender,
        event_rx: EventReceiver,
        tray: Option<TrayState>,
        hotkeys: Option<HotkeyState>,
    ) -> Self {
        // Configure dark visuals
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::TRANSPARENT;
        cc.egui_ctx.set_visuals(visuals);

        // Spawn tray event thread
        if let Some(ref tray) = tray {
            tray::spawn_tray_event_thread(tray, event_tx.clone(), cc.egui_ctx.clone());
        }

        // Spawn hotkey event thread
        if let Some(ref hk) = hotkeys {
            hotkeys::spawn_hotkey_event_thread(hk, event_tx.clone(), cc.egui_ctx.clone());
        }

        let always_on_top = config.always_on_top;
        let display_mode = if config.compact_mode {
            DisplayMode::Compact
        } else {
            DisplayMode::Normal
        };

        Self {
            timer: TimerEngine::new(config.focus_minutes),
            audio: AudioPlayer::new(),
            config,
            event_rx,
            event_tx,
            is_visible: true,
            always_on_top,
            display_mode,
            _tray: tray,
            _hotkeys: hotkeys,
            needs_session_transition: false,
        }
    }

    fn handle_events(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                AppEvent::ToggleStartPause => self.timer.toggle_start_pause(),
                AppEvent::Reset => self.timer.reset(self.config.focus_minutes),
                AppEvent::Skip => {
                    if let Some(ended) = self.timer.skip() {
                        self.on_session_ended(ended.completed);
                    }
                }
                AppEvent::ToggleVisibility => {
                    self.is_visible = !self.is_visible;
                    ctx.send_viewport_cmd(ViewportCommand::Visible(self.is_visible));
                }
                AppEvent::ShowWindow => {
                    self.is_visible = true;
                    ctx.send_viewport_cmd(ViewportCommand::Visible(true));
                    ctx.send_viewport_cmd(ViewportCommand::Focus);
                }
                AppEvent::HideWindow => {
                    self.is_visible = false;
                    ctx.send_viewport_cmd(ViewportCommand::Visible(false));
                }
                AppEvent::ToggleAlwaysOnTop => {
                    self.always_on_top = !self.always_on_top;
                    let level = if self.always_on_top {
                        egui::viewport::WindowLevel::AlwaysOnTop
                    } else {
                        egui::viewport::WindowLevel::Normal
                    };
                    ctx.send_viewport_cmd(ViewportCommand::WindowLevel(level));
                }
                AppEvent::Quit => {
                    self.save_window_state(ctx);
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                }
            }
        }
    }

    fn on_session_ended(&mut self, completed: SessionType) {
        if self.config.sound_enabled {
            match completed {
                SessionType::Focus => self.audio.play_focus_chime(),
                SessionType::Break => self.audio.play_break_chime(),
            }
        }
        self.needs_session_transition = true;
    }

    fn check_auto_start(&mut self) {
        if !self.needs_session_transition {
            return;
        }
        self.needs_session_transition = false;

        let should_auto = match self.timer.session_type {
            SessionType::Focus => self.config.auto_start_break,
            SessionType::Break => self.config.auto_start_focus,
        };

        if should_auto && self.timer.state == TimerState::Completed {
            self.timer
                .start_next_session(self.config.focus_minutes, self.config.break_minutes);
        }
    }

    fn save_window_state(&mut self, ctx: &egui::Context) {
        if self.config.remember_window_position {
            ctx.input(|i| {
                if let Some(rect) = i.viewport().outer_rect {
                    self.config.window_x = Some(rect.min.x);
                    self.config.window_y = Some(rect.min.y);
                }
                if let Some(rect) = i.viewport().inner_rect {
                    self.config.window_width = Some(rect.width());
                    self.config.window_height = Some(rect.height());
                }
            });
        }
        self.config.compact_mode = self.display_mode == DisplayMode::Compact;
        self.config.always_on_top = self.always_on_top;
        let _ = self.config.save();
    }

    fn session_color(&self) -> Color32 {
        match self.timer.state {
            TimerState::Completed => Color32::from_rgb(230, 160, 50), // orange
            _ => match self.timer.session_type {
                SessionType::Focus => Color32::from_rgb(80, 200, 120), // green
                SessionType::Break => Color32::from_rgb(80, 150, 230), // blue
            },
        }
    }

    fn render_ui(&mut self, ctx: &egui::Context) {
        let panel_frame = Frame::new()
            .fill(Color32::from_rgba_unmultiplied(30, 30, 35, (self.config.window_opacity * 255.0) as u8))
            .inner_margin(12.0)
            .corner_radius(10.0);

        CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
            // Enable dragging from anywhere on the panel
            let response = ui.interact(
                ui.available_rect_before_wrap(),
                ui.id().with("drag_area"),
                egui::Sense::click_and_drag(),
            );
            if response.is_pointer_button_down_on() {
                ctx.send_viewport_cmd(ViewportCommand::StartDrag);
            }

            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                // Session label
                let label_color = self.session_color();
                ui.label(
                    RichText::new(self.timer.session_type.label())
                        .color(label_color)
                        .size(14.0),
                );

                ui.add_space(8.0);

                // Progress ring + timer
                let ring_radius = if self.display_mode == DisplayMode::Compact {
                    45.0
                } else {
                    60.0
                };
                let ring_size = Vec2::splat(ring_radius * 2.0 + 20.0);
                let (rect, _) = ui.allocate_exact_size(ring_size, egui::Sense::hover());
                let center = rect.center();

                let fg_color = self.session_color();
                let bg_color = Color32::from_rgb(50, 50, 55);

                ui::render_progress_ring(
                    ui.painter(),
                    center,
                    ring_radius,
                    self.timer.progress(),
                    fg_color,
                    bg_color,
                    6.0,
                );

                // Timer text centered in the ring
                let remaining = self.timer.remaining();
                let mins = remaining.as_secs() / 60;
                let secs = remaining.as_secs() % 60;
                let timer_text = format!("{:02}:{:02}", mins, secs);

                let font_size = if self.display_mode == DisplayMode::Compact {
                    22.0
                } else {
                    28.0
                };
                let text_galley = ui.painter().layout_no_wrap(
                    timer_text,
                    egui::FontId::monospace(font_size),
                    Color32::WHITE,
                );
                let text_pos = Pos2::new(
                    center.x - text_galley.size().x / 2.0,
                    center.y - text_galley.size().y / 2.0,
                );
                ui.painter().galley(text_pos, text_galley, Color32::WHITE);

                ui.add_space(8.0);

                // Controls
                if let Some(action) = ui::render_controls(ui, self.timer.state) {
                    match action {
                        ControlAction::ToggleStartPause => {
                            self.timer.toggle_start_pause();
                        }
                        ControlAction::Reset => {
                            self.timer.reset(self.config.focus_minutes);
                        }
                        ControlAction::Skip => {
                            if let Some(ended) = self.timer.skip() {
                                self.on_session_ended(ended.completed);
                            }
                        }
                    }
                }

                // Focus count + mode toggle (normal mode only)
                if self.display_mode == DisplayMode::Normal {
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("Sessions: {}", self.timer.completed_focus_count))
                                .color(Color32::from_rgb(150, 150, 155))
                                .size(11.0),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .small_button(if self.always_on_top { "📌" } else { "📄" })
                                .on_hover_text("Toggle Always On Top")
                                .clicked()
                            {
                                let _ = self.event_tx.send(AppEvent::ToggleAlwaysOnTop);
                            }
                        });
                    });
                }

                // Compact/Normal toggle
                ui.add_space(4.0);
                let toggle_label = match self.display_mode {
                    DisplayMode::Normal => "Compact",
                    DisplayMode::Compact => "Expand",
                };
                if ui
                    .small_button(
                        RichText::new(toggle_label)
                            .color(Color32::from_rgb(130, 130, 135))
                            .size(10.0),
                    )
                    .clicked()
                {
                    self.display_mode = match self.display_mode {
                        DisplayMode::Normal => DisplayMode::Compact,
                        DisplayMode::Compact => DisplayMode::Normal,
                    };
                    let size = match self.display_mode {
                        DisplayMode::Normal => Vec2::new(220.0, 280.0),
                        DisplayMode::Compact => Vec2::new(160.0, 210.0),
                    };
                    ctx.send_viewport_cmd(ViewportCommand::InnerSize(size));
                }
            });
        });
    }
}

impl eframe::App for WorkTimerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle close-to-tray
        if ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            self.is_visible = false;
            ctx.send_viewport_cmd(ViewportCommand::Visible(false));
        }

        // Process external events
        self.handle_events(ctx);

        // Tick the timer
        if let Some(ended) = self.timer.tick() {
            self.on_session_ended(ended.completed);
        }

        // Auto-start next session if configured
        self.check_auto_start();

        // Render
        self.render_ui(ctx);

        // Schedule repaint based on timer state
        match self.timer.state {
            TimerState::Running => {
                ctx.request_repaint_after(Duration::from_millis(250));
            }
            TimerState::Completed => {
                // Repaint occasionally for any visual effects
                ctx.request_repaint_after(Duration::from_secs(1));
            }
            _ => {
                // Idle/Paused: only wake on events (tray, hotkey, mouse)
            }
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0] // transparent background
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Note: can't access ctx here easily, so we save config with whatever we have
        let _ = self.config.save();
    }
}
