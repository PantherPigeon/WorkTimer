use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerState {
    Idle,
    Running,
    Paused,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    Focus,
    Break,
}

impl SessionType {
    pub fn label(self) -> &'static str {
        match self {
            SessionType::Focus => "Focus",
            SessionType::Break => "Break",
        }
    }

    pub fn next(self) -> Self {
        match self {
            SessionType::Focus => SessionType::Break,
            SessionType::Break => SessionType::Focus,
        }
    }
}

/// Returned by `tick()` when a session naturally completes.
#[derive(Debug, Clone, Copy)]
pub struct SessionEnded {
    pub completed: SessionType,
    pub next: SessionType,
}

pub struct TimerEngine {
    pub state: TimerState,
    pub session_type: SessionType,
    pub duration: Duration,
    elapsed: Duration,
    last_tick: Option<Instant>,
    pub completed_focus_count: u32,
}

impl TimerEngine {
    pub fn new(focus_minutes: u32) -> Self {
        Self {
            state: TimerState::Idle,
            session_type: SessionType::Focus,
            duration: Duration::from_secs(focus_minutes as u64 * 60),
            elapsed: Duration::ZERO,
            last_tick: None,
            completed_focus_count: 0,
        }
    }

    pub fn start(&mut self) {
        match self.state {
            TimerState::Idle | TimerState::Completed => {
                if self.state == TimerState::Completed {
                    // Transition to next session
                    self.session_type = self.session_type.next();
                    self.elapsed = Duration::ZERO;
                }
                self.last_tick = Some(Instant::now());
                self.state = TimerState::Running;
            }
            TimerState::Paused => {
                self.last_tick = Some(Instant::now());
                self.state = TimerState::Running;
            }
            TimerState::Running => {}
        }
    }

    pub fn pause(&mut self) {
        if self.state == TimerState::Running {
            self.accumulate_elapsed();
            self.last_tick = None;
            self.state = TimerState::Paused;
        }
    }

    pub fn toggle_start_pause(&mut self) {
        match self.state {
            TimerState::Running => self.pause(),
            _ => self.start(),
        }
    }

    pub fn reset(&mut self, focus_minutes: u32) {
        self.state = TimerState::Idle;
        self.session_type = SessionType::Focus;
        self.duration = Duration::from_secs(focus_minutes as u64 * 60);
        self.elapsed = Duration::ZERO;
        self.last_tick = None;
        self.completed_focus_count = 0;
    }

    pub fn skip(&mut self) -> Option<SessionEnded> {
        if self.state == TimerState::Idle {
            return None;
        }
        let completed = self.session_type;
        if completed == SessionType::Focus {
            self.completed_focus_count += 1;
        }
        self.elapsed = Duration::ZERO;
        self.last_tick = None;
        self.state = TimerState::Completed;
        Some(SessionEnded {
            completed,
            next: completed.next(),
        })
    }

    /// Call each frame. Returns `Some(SessionEnded)` when the session just completed.
    pub fn tick(&mut self) -> Option<SessionEnded> {
        if self.state != TimerState::Running {
            return None;
        }

        self.accumulate_elapsed();
        self.last_tick = Some(Instant::now());

        if self.elapsed >= self.duration {
            self.elapsed = self.duration;
            let completed = self.session_type;
            if completed == SessionType::Focus {
                self.completed_focus_count += 1;
            }
            self.state = TimerState::Completed;
            self.last_tick = None;
            Some(SessionEnded {
                completed,
                next: completed.next(),
            })
        } else {
            None
        }
    }

    /// Set the duration for the current session type. Used when transitioning.
    pub fn set_next_duration(&mut self, focus_minutes: u32, break_minutes: u32) {
        self.duration = match self.session_type {
            SessionType::Focus => Duration::from_secs(focus_minutes as u64 * 60),
            SessionType::Break => Duration::from_secs(break_minutes as u64 * 60),
        };
    }

    /// Start the next session (after Completed state).
    pub fn start_next_session(&mut self, focus_minutes: u32, break_minutes: u32) {
        self.session_type = self.session_type.next();
        self.elapsed = Duration::ZERO;
        self.duration = match self.session_type {
            SessionType::Focus => Duration::from_secs(focus_minutes as u64 * 60),
            SessionType::Break => Duration::from_secs(break_minutes as u64 * 60),
        };
        self.last_tick = Some(Instant::now());
        self.state = TimerState::Running;
    }

    pub fn remaining(&self) -> Duration {
        let total_elapsed = self.current_elapsed();
        self.duration.saturating_sub(total_elapsed)
    }

    /// Progress fraction from 0.0 (just started) to 1.0 (completed).
    pub fn progress(&self) -> f32 {
        if self.duration.is_zero() {
            return 1.0;
        }
        let elapsed = self.current_elapsed();
        (elapsed.as_secs_f32() / self.duration.as_secs_f32()).clamp(0.0, 1.0)
    }

    fn current_elapsed(&self) -> Duration {
        let mut total = self.elapsed;
        if let Some(last) = self.last_tick {
            total += Instant::now().duration_since(last);
        }
        total.min(self.duration)
    }

    fn accumulate_elapsed(&mut self) {
        if let Some(last) = self.last_tick {
            self.elapsed += Instant::now().duration_since(last);
        }
    }
}
