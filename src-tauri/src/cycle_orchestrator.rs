use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::database::models::{UserSettings, WorkSchedule};

/// Represents the current phase of the work cycle
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CyclePhase {
    Idle,
    Focus,
    ShortBreak,
    LongBreak,
}

impl std::fmt::Display for CyclePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CyclePhase::Idle => write!(f, "idle"),
            CyclePhase::Focus => write!(f, "focus"),
            CyclePhase::ShortBreak => write!(f, "short_break"),
            CyclePhase::LongBreak => write!(f, "long_break"),
        }
    }
}

/// Current state of the work cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleState {
    pub phase: CyclePhase,
    pub remaining_seconds: u32,
    pub cycle_count: u32,
    pub is_running: bool,
    pub can_start: bool,
    pub session_id: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
}

impl Default for CycleState {
    fn default() -> Self {
        Self {
            phase: CyclePhase::Idle,
            remaining_seconds: 0,
            cycle_count: 0,
            is_running: false,
            can_start: true,
            session_id: None,
            started_at: None,
        }
    }
}

/// Configuration for work cycles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleConfig {
    pub focus_duration: u32,      // seconds
    pub break_duration: u32,      // seconds
    pub long_break_duration: u32, // seconds
    pub cycles_per_long_break: u32,
    pub strict_mode: bool,
    pub work_schedule: Option<WorkSchedule>,
    pub emergency_key: Option<String>,
    pub user_name: Option<String>,
}

impl CycleConfig {
    /// Create configuration from user settings
    pub fn from_user_settings(settings: UserSettings, work_schedule: Option<WorkSchedule>) -> Self {
        Self {
            focus_duration: settings.focus_duration as u32,
            break_duration: settings.short_break_duration as u32,
            long_break_duration: settings.long_break_duration as u32,
            cycles_per_long_break: settings.cycles_per_long_break_v2 as u32,
            strict_mode: settings.strict_mode,
            work_schedule,
            emergency_key: settings.emergency_key_combination,
            user_name: settings.user_name,
        }
    }
}

/// Events that can be emitted by the cycle orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CycleEvent {
    PhaseStarted {
        phase: CyclePhase,
        duration: u32,
        cycle_count: u32,
    },
    PhaseEnded {
        phase: CyclePhase,
        completed: bool,
    },
    Tick {
        remaining: u32,
    },
    PreAlert {
        remaining: u32,
    },
    CycleCompleted {
        cycle_count: u32,
    },
    LongBreakReached {
        cycles_completed: u32,
    },
}

/// Orchestrates work cycles with focus and break periods
pub struct CycleOrchestrator {
    config: CycleConfig,
    state: CycleState,
}

impl CycleOrchestrator {
    /// Create a new cycle orchestrator
    pub fn new(config: CycleConfig) -> Self {
        Self {
            config,
            state: CycleState::default(),
        }
    }

    /// Get the current cycle state
    pub fn get_state(&self) -> CycleState {
        self.state.clone()
    }

    /// Check if we're within work hours (if work schedule is configured)
    pub fn is_within_work_hours(&self) -> bool {
        if let Some(ref schedule) = self.config.work_schedule {
            if !schedule.use_work_schedule {
                return true; // No work schedule restriction
            }

            if let (Some(ref start), Some(ref end)) =
                (&schedule.work_start_time, &schedule.work_end_time)
            {
                let now = chrono::Local::now();
                let current_time = now.format("%H:%M").to_string();

                // Simple string comparison works for HH:MM format
                return current_time >= *start && current_time <= *end;
            }
        }

        true // Default to allowing if no schedule configured
    }

    /// Start a focus session
    pub fn start_focus_session(&mut self) -> Result<Vec<CycleEvent>, String> {
        // Check if we can start (work hours validation)
        if !self.is_within_work_hours() {
            return Err("Cannot start focus session outside work hours".to_string());
        }

        // Can only start from idle state
        if self.state.phase != CyclePhase::Idle {
            return Err(format!(
                "Cannot start focus session from {} state",
                self.state.phase
            ));
        }

        // Generate session ID
        let session_id = uuid::Uuid::new_v4().to_string();

        // Update state
        self.state.phase = CyclePhase::Focus;
        self.state.remaining_seconds = self.config.focus_duration;
        self.state.is_running = true;
        self.state.session_id = Some(session_id);
        self.state.started_at = Some(Utc::now());

        Ok(vec![CycleEvent::PhaseStarted {
            phase: CyclePhase::Focus,
            duration: self.config.focus_duration,
            cycle_count: self.state.cycle_count,
        }])
    }

    /// Start a break (short or long based on cycle count)
    pub fn start_break(&mut self, force_long: bool) -> Result<Vec<CycleEvent>, String> {
        // Can only start break from idle state
        if self.state.phase != CyclePhase::Idle {
            return Err(format!(
                "Cannot start break from {} state",
                self.state.phase
            ));
        }

        // Determine if this should be a long break
        let is_long_break = force_long
            || (self.state.cycle_count > 0
                && self.state.cycle_count % self.config.cycles_per_long_break == 0);

        let (phase, duration) = if is_long_break {
            (CyclePhase::LongBreak, self.config.long_break_duration)
        } else {
            (CyclePhase::ShortBreak, self.config.break_duration)
        };

        // Generate session ID
        let session_id = uuid::Uuid::new_v4().to_string();

        // Update state
        self.state.phase = phase.clone();
        self.state.remaining_seconds = duration;
        self.state.is_running = true;
        self.state.session_id = Some(session_id);
        self.state.started_at = Some(Utc::now());

        let mut events = vec![CycleEvent::PhaseStarted {
            phase: phase.clone(),
            duration,
            cycle_count: self.state.cycle_count,
        }];

        // Emit long break event if applicable
        if is_long_break {
            events.push(CycleEvent::LongBreakReached {
                cycles_completed: self.state.cycle_count,
            });
        }

        Ok(events)
    }

    /// Pause the current session
    pub fn pause(&mut self) -> Result<(), String> {
        if !self.state.is_running {
            return Err("No active session to pause".to_string());
        }

        self.state.is_running = false;
        Ok(())
    }

    /// Resume the current session
    pub fn resume(&mut self) -> Result<(), String> {
        if self.state.is_running {
            return Err("Session is already running".to_string());
        }

        if self.state.phase == CyclePhase::Idle {
            return Err("No session to resume".to_string());
        }

        self.state.is_running = true;
        Ok(())
    }

    /// End the current session and transition to idle
    pub fn end_session(&mut self, completed: bool) -> Result<Vec<CycleEvent>, String> {
        let current_phase = self.state.phase.clone();

        if current_phase == CyclePhase::Idle {
            return Err("No active session to end".to_string());
        }

        let mut events = vec![CycleEvent::PhaseEnded {
            phase: current_phase.clone(),
            completed,
        }];

        // If a focus session was completed, increment cycle count
        if completed && current_phase == CyclePhase::Focus {
            self.state.cycle_count += 1;
            events.push(CycleEvent::CycleCompleted {
                cycle_count: self.state.cycle_count,
            });
        }

        // Reset to idle state
        self.state.phase = CyclePhase::Idle;
        self.state.remaining_seconds = 0;
        self.state.is_running = false;
        self.state.session_id = None;
        self.state.started_at = None;

        Ok(events)
    }

    /// Handle a timer tick (called every second)
    pub fn tick(&mut self) -> Result<Vec<CycleEvent>, String> {
        if !self.state.is_running || self.state.phase == CyclePhase::Idle {
            return Ok(vec![]);
        }

        let mut events = vec![];

        // Decrement remaining time
        if self.state.remaining_seconds > 0 {
            self.state.remaining_seconds -= 1;

            // Emit tick event
            events.push(CycleEvent::Tick {
                remaining: self.state.remaining_seconds,
            });

            // Check for pre-alert (2 minutes before end)
            if self.state.remaining_seconds == 120 {
                events.push(CycleEvent::PreAlert {
                    remaining: self.state.remaining_seconds,
                });
            }

            // Check if session completed
            if self.state.remaining_seconds == 0 {
                // Auto-complete the session
                let completion_events = self.end_session(true)?;
                events.extend(completion_events);
            }
        }

        Ok(events)
    }

    /// Reset the cycle counter (useful after a long break)
    pub fn reset_cycle_count(&mut self) {
        self.state.cycle_count = 0;
    }

    /// Update the configuration
    pub fn update_config(&mut self, config: CycleConfig) {
        self.config = config;
    }
}

