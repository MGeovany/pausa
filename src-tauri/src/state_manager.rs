use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::time::{interval, MissedTickBehavior};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::database::{DatabaseManager, models::{Session, SessionType, UserSettings as DbUserSettings}};
use crate::api_models::{FocusSession, BreakSession, BreakType, SessionState, UserSettings};

/// Core application state enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AppState {
    Idle,
    FocusRunning,
    FocusPreAlert,
    FocusEnding,
    BreakRunning,
    LongBreakRunning,
}

/// Events emitted by the state manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateEvent {
    StateChanged { from: AppState, to: AppState },
    SessionStarted { session_id: String },
    SessionPaused { session_id: String },
    SessionResumed { session_id: String },
    SessionCompleted { session_id: String },
    PreAlertTriggered { session_id: String, remaining_seconds: u32 },
    BreakStarted { break_session: BreakSession },
    BreakCompleted { session_id: String },
    TimerTick { remaining_seconds: u32 },
}

/// Timer state for precise timing
#[derive(Debug, Clone)]
struct TimerState {
    start_time: Instant,
    pause_time: Option<Instant>,
    total_paused_duration: Duration,
    planned_duration: Duration,
}

impl TimerState {
    fn new(duration_seconds: u32) -> Self {
        Self {
            start_time: Instant::now(),
            pause_time: None,
            total_paused_duration: Duration::ZERO,
            planned_duration: Duration::from_secs(duration_seconds as u64),
        }
    }

    fn pause(&mut self) {
        if self.pause_time.is_none() {
            self.pause_time = Some(Instant::now());
        }
    }

    fn resume(&mut self) {
        if let Some(pause_start) = self.pause_time.take() {
            self.total_paused_duration += pause_start.elapsed();
        }
    }

    fn elapsed(&self) -> Duration {
        let total_elapsed = self.start_time.elapsed();
        let active_elapsed = if let Some(pause_start) = self.pause_time {
            total_elapsed - pause_start.elapsed()
        } else {
            total_elapsed
        };
        active_elapsed - self.total_paused_duration
    }

    fn remaining(&self) -> Duration {
        self.planned_duration.saturating_sub(self.elapsed())
    }

    fn is_finished(&self) -> bool {
        self.elapsed() >= self.planned_duration
    }

    fn is_paused(&self) -> bool {
        self.pause_time.is_some()
    }
}

/// Core state manager for the application
pub struct StateManager {
    current_state: AppState,
    current_session: Option<FocusSession>,
    current_break: Option<BreakSession>,
    timer_state: Option<TimerState>,
    settings: UserSettings,
    database: Arc<Mutex<DatabaseManager>>,
    cycle_count: u32,
    pre_alert_triggered: bool,
}

impl StateManager {
    /// Create a new state manager
    pub fn new(database: Arc<Mutex<DatabaseManager>>) -> Result<Self, Box<dyn std::error::Error>> {
        let settings = {
            let db = database.lock().map_err(|e| format!("Failed to lock database: {}", e))?;
            match db.get_user_settings() {
                Ok(Some(db_settings)) => db_settings.into(),
                Ok(None) => {
                    // Create default settings
                    let default_settings = DbUserSettings::default();
                    db.save_user_settings(&default_settings)?;
                    default_settings.into()
                }
                Err(e) => return Err(format!("Failed to load settings: {}", e).into()),
            }
        };

        let mut manager = Self {
            current_state: AppState::Idle,
            current_session: None,
            current_break: None,
            timer_state: None,
            settings,
            database,
            cycle_count: 0,
            pre_alert_triggered: false,
        };

        // Attempt to recover any active session
        manager.recover_active_session()?;

        Ok(manager)
    }

    /// Get current application state
    pub fn get_state(&self) -> AppState {
        self.current_state.clone()
    }

    /// Get current focus session
    pub fn get_current_session(&self) -> Option<FocusSession> {
        self.current_session.clone()
    }

    /// Get current break session
    pub fn get_current_break(&self) -> Option<BreakSession> {
        self.current_break.clone()
    }

    /// Get current settings
    pub fn get_settings(&self) -> UserSettings {
        self.settings.clone()
    }

    /// Update settings
    pub fn update_settings(&mut self, new_settings: UserSettings) -> Result<(), Box<dyn std::error::Error>> {
        let db_settings: DbUserSettings = new_settings.clone().into();
        {
            let db = self.database.lock().map_err(|e| format!("Failed to lock database: {}", e))?;
            db.save_user_settings(&db_settings)?;
        }
        self.settings = new_settings;
        Ok(())
    }

    /// Start a new focus session
    pub fn start_focus_session(&mut self, strict_mode: bool) -> Result<Vec<StateEvent>, Box<dyn std::error::Error>> {
        if !matches!(self.current_state, AppState::Idle) {
            return Err("Cannot start focus session: not in idle state".into());
        }

        let session_id = Uuid::new_v4().to_string();
        let start_time = Utc::now();
        let duration_seconds = self.settings.focus_duration * 60;

        // Create focus session
        let focus_session = FocusSession {
            id: session_id.clone(),
            start_time,
            duration: duration_seconds,
            remaining: duration_seconds,
            is_running: true,
            is_strict: strict_mode,
            state: SessionState::Running,
        };

        // Create timer state
        let timer_state = TimerState::new(duration_seconds);

        // Save to database
        {
            let db = self.database.lock().map_err(|e| format!("Failed to lock database: {}", e))?;
            let db_session = focus_session.to_db_session();
            db.create_session(&db_session)?;
        }

        // Update state
        let old_state = self.current_state.clone();
        self.current_state = AppState::FocusRunning;
        self.current_session = Some(focus_session);
        self.timer_state = Some(timer_state);
        self.pre_alert_triggered = false;

        Ok(vec![
            StateEvent::StateChanged { from: old_state, to: self.current_state.clone() },
            StateEvent::SessionStarted { session_id },
        ])
    }

    /// Pause the current session
    pub fn pause_session(&mut self) -> Result<Vec<StateEvent>, Box<dyn std::error::Error>> {
        match self.current_state {
            AppState::FocusRunning | AppState::FocusPreAlert => {
                if let Some(ref mut timer) = self.timer_state {
                    timer.pause();
                }
                if let Some(ref mut session) = self.current_session {
                    session.is_running = false;
                    session.state = SessionState::Idle;
                    
                    let session_id = session.id.clone();
                    return Ok(vec![StateEvent::SessionPaused { session_id }]);
                }
            }
            _ => return Err("No active session to pause".into()),
        }
        Ok(vec![])
    }

    /// Resume the current session
    pub fn resume_session(&mut self) -> Result<Vec<StateEvent>, Box<dyn std::error::Error>> {
        if let Some(ref mut session) = self.current_session {
            if !session.is_running {
                if let Some(ref mut timer) = self.timer_state {
                    timer.resume();
                }
                session.is_running = true;
                session.state = if session.remaining <= self.settings.pre_alert_seconds {
                    SessionState::PreAlert
                } else {
                    SessionState::Running
                };

                // Update app state
                self.current_state = if matches!(session.state, SessionState::PreAlert) {
                    AppState::FocusPreAlert
                } else {
                    AppState::FocusRunning
                };

                let session_id = session.id.clone();
                return Ok(vec![StateEvent::SessionResumed { session_id }]);
            }
        }
        Err("No paused session to resume".into())
    }

    /// End the current session
    pub fn end_session(&mut self) -> Result<Vec<StateEvent>, Box<dyn std::error::Error>> {
        let mut events = Vec::new();

        if let Some(session) = self.current_session.take() {
            // Update session in database
            {
                let db = self.database.lock().map_err(|e| format!("Failed to lock database: {}", e))?;
                let mut db_session = session.to_db_session();
                db_session.end_time = Some(Utc::now());
                db_session.completed = session.remaining == 0;
                db_session.actual_duration = Some((session.duration - session.remaining) as i32);
                db.update_session(&db_session)?;
            }

            let session_id = session.id.clone();
            let was_completed = session.remaining == 0;

            // Clear timer state
            self.timer_state = None;
            self.pre_alert_triggered = false;

            events.push(StateEvent::SessionCompleted { session_id });

            // If session was completed, start appropriate break
            if was_completed {
                self.cycle_count += 1;
                let break_events = self.start_break_session()?;
                events.extend(break_events);
            } else {
                // Session was manually ended, return to idle
                let old_state = self.current_state.clone();
                self.current_state = AppState::Idle;
                events.push(StateEvent::StateChanged { from: old_state, to: self.current_state.clone() });
            }
        }

        Ok(events)
    }

    /// Start a break session
    fn start_break_session(&mut self) -> Result<Vec<StateEvent>, Box<dyn std::error::Error>> {
        let is_long_break = self.cycle_count % self.settings.cycles_per_long_break == 0;
        let (break_type, duration_minutes) = if is_long_break {
            (BreakType::Long, self.settings.long_break_duration)
        } else {
            (BreakType::Short, self.settings.short_break_duration)
        };

        let break_session = BreakSession::new(break_type.clone(), duration_minutes, self.settings.strict_mode);
        
        // Save break session to database
        {
            let db = self.database.lock().map_err(|e| format!("Failed to lock database: {}", e))?;
            let db_session = break_session.to_db_session(Utc::now());
            db.create_session(&db_session)?;
        }

        // Update state
        let old_state = self.current_state.clone();
        self.current_state = match break_type {
            BreakType::Long => AppState::LongBreakRunning,
            BreakType::Short => AppState::BreakRunning,
        };

        // Create timer for break
        self.timer_state = Some(TimerState::new(break_session.duration));
        self.current_break = Some(break_session.clone());

        Ok(vec![
            StateEvent::StateChanged { from: old_state, to: self.current_state.clone() },
            StateEvent::BreakStarted { break_session },
        ])
    }

    /// Complete the current break
    pub fn complete_break(&mut self) -> Result<Vec<StateEvent>, Box<dyn std::error::Error>> {
        let mut events = Vec::new();

        if let Some(break_session) = self.current_break.take() {
            // Update break session in database
            {
                let db = self.database.lock().map_err(|e| format!("Failed to lock database: {}", e))?;
                let mut db_session = break_session.to_db_session(Utc::now() - chrono::Duration::seconds(break_session.duration as i64));
                db_session.end_time = Some(Utc::now());
                db_session.completed = true;
                db_session.actual_duration = Some(break_session.duration as i32);
                db.update_session(&db_session)?;
            }

            let session_id = break_session.id.clone();
            events.push(StateEvent::BreakCompleted { session_id });

            // Return to idle state
            let old_state = self.current_state.clone();
            self.current_state = AppState::Idle;
            self.timer_state = None;

            events.push(StateEvent::StateChanged { from: old_state, to: self.current_state.clone() });
        }

        Ok(events)
    }

    /// Handle timer tick - should be called regularly (e.g., every second)
    pub fn handle_timer_tick(&mut self) -> Result<Vec<StateEvent>, Box<dyn std::error::Error>> {
        let mut events = Vec::new();

        if let Some(ref timer) = self.timer_state {
            if timer.is_paused() {
                return Ok(events);
            }

            let remaining_duration = timer.remaining();
            let remaining_seconds = remaining_duration.as_secs() as u32;

            // Update current session or break with remaining time
            match self.current_state {
                AppState::FocusRunning | AppState::FocusPreAlert | AppState::FocusEnding => {
                    if let Some(ref mut session) = self.current_session {
                        session.remaining = remaining_seconds;

                        // Check for pre-alert trigger
                        if remaining_seconds <= self.settings.pre_alert_seconds && 
                           remaining_seconds > 0 && 
                           !self.pre_alert_triggered &&
                           matches!(self.current_state, AppState::FocusRunning) {
                            
                            self.pre_alert_triggered = true;
                            let old_state = self.current_state.clone();
                            self.current_state = AppState::FocusPreAlert;
                            session.state = SessionState::PreAlert;

                            events.push(StateEvent::StateChanged { from: old_state, to: self.current_state.clone() });
                            events.push(StateEvent::PreAlertTriggered { 
                                session_id: session.id.clone(), 
                                remaining_seconds 
                            });
                        }

                        // Check for session completion
                        if timer.is_finished() && !matches!(self.current_state, AppState::FocusEnding) {
                            let old_state = self.current_state.clone();
                            self.current_state = AppState::FocusEnding;
                            session.state = SessionState::Ending;
                            session.remaining = 0;

                            events.push(StateEvent::StateChanged { from: old_state, to: self.current_state.clone() });
                            
                            // Auto-end the session after a brief moment
                            let end_events = self.end_session()?;
                            events.extend(end_events);
                        } else {
                            events.push(StateEvent::TimerTick { remaining_seconds });
                        }
                    }
                }
                AppState::BreakRunning | AppState::LongBreakRunning => {
                    if let Some(ref mut break_session) = self.current_break {
                        break_session.remaining = remaining_seconds;

                        // Check for break completion
                        if timer.is_finished() {
                            let complete_events = self.complete_break()?;
                            events.extend(complete_events);
                        } else {
                            events.push(StateEvent::TimerTick { remaining_seconds });
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(events)
    }

    /// Recover active session on app restart
    fn recover_active_session(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let db = self.database.lock().map_err(|e| format!("Failed to lock database: {}", e))?;
        
        // Look for the most recent incomplete session
        if let Some(db_session) = db.get_active_session()? {
            let current_time = Utc::now();
            
            match db_session.session_type {
                SessionType::Focus => {
                    if let Some(focus_session) = FocusSession::from_db_session(db_session, current_time) {
                        if focus_session.is_running && focus_session.remaining > 0 {
                            // Recover focus session
                            let elapsed_seconds = focus_session.duration - focus_session.remaining;
                            let mut timer_state = TimerState::new(focus_session.duration);
                            
                            // Adjust timer to account for elapsed time
                            timer_state.start_time = Instant::now() - Duration::from_secs(elapsed_seconds as u64);
                            
                            self.current_session = Some(focus_session);
                            self.timer_state = Some(timer_state);
                            self.current_state = AppState::FocusRunning;
                            self.pre_alert_triggered = false;
                        }
                    }
                }
                SessionType::ShortBreak | SessionType::LongBreak => {
                    // For breaks, we'll just complete them on recovery for simplicity
                    // In a real app, you might want to recover break state as well
                    let mut completed_session = db_session;
                    completed_session.end_time = Some(current_time);
                    completed_session.completed = true;
                    completed_session.actual_duration = Some(completed_session.planned_duration);
                    db.update_session(&completed_session)?;
                }
            }
        }

        Ok(())
    }

    /// Reset the cycle count (useful for testing or manual reset)
    pub fn reset_cycle_count(&mut self) {
        self.cycle_count = 0;
    }

    /// Get current cycle count
    pub fn get_cycle_count(&self) -> u32 {
        self.cycle_count
    }

    /// Start the timer service for this state manager
    pub fn start_timer_service(state_manager: Arc<Mutex<StateManager>>) -> mpsc::UnboundedReceiver<Vec<StateEvent>> {
        let (tx, rx) = mpsc::unbounded_channel();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
            
            loop {
                interval.tick().await;
                
                // Handle timer tick
                if let Ok(mut manager) = state_manager.try_lock() {
                    match manager.handle_timer_tick() {
                        Ok(events) => {
                            if !events.is_empty() {
                                if let Err(_) = tx.send(events) {
                                    // Receiver dropped, exit the loop
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Timer tick error: {}", e);
                        }
                    }
                } else {
                    // State manager is locked, skip this tick
                    continue;
                }
            }
        });
        
        rx
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tempfile::tempdir;

    fn create_test_state_manager() -> StateManager {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db_manager = DatabaseManager::new(db_path).unwrap();
        let db_arc = Arc::new(Mutex::new(db_manager));
        StateManager::new(db_arc).unwrap()
    }

    #[test]
    fn test_initial_state() {
        let manager = create_test_state_manager();
        assert_eq!(manager.get_state(), AppState::Idle);
        assert!(manager.get_current_session().is_none());
        assert!(manager.get_current_break().is_none());
    }

    #[test]
    fn test_start_focus_session() {
        let mut manager = create_test_state_manager();
        let events = manager.start_focus_session(false).unwrap();
        
        assert_eq!(manager.get_state(), AppState::FocusRunning);
        assert!(manager.get_current_session().is_some());
        assert_eq!(events.len(), 2);
        
        let session = manager.get_current_session().unwrap();
        assert_eq!(session.duration, 25 * 60); // 25 minutes default
        assert_eq!(session.is_strict, false);
        assert!(session.is_running);
    }

    #[test]
    fn test_pause_resume_session() {
        let mut manager = create_test_state_manager();
        manager.start_focus_session(false).unwrap();
        
        // Pause
        let pause_events = manager.pause_session().unwrap();
        assert_eq!(pause_events.len(), 1);
        let session = manager.get_current_session().unwrap();
        assert!(!session.is_running);
        
        // Resume
        let resume_events = manager.resume_session().unwrap();
        assert_eq!(resume_events.len(), 1);
        let session = manager.get_current_session().unwrap();
        assert!(session.is_running);
    }

    #[test]
    fn test_timer_state() {
        let mut timer = TimerState::new(60); // 1 minute
        
        assert!(!timer.is_finished());
        assert!(!timer.is_paused());
        
        timer.pause();
        assert!(timer.is_paused());
        
        timer.resume();
        assert!(!timer.is_paused());
        
        // Timer should have some remaining time
        assert!(timer.remaining().as_secs() <= 60);
    }

    #[test]
    fn test_state_transitions() {
        let mut manager = create_test_state_manager();
        
        // Idle -> FocusRunning
        manager.start_focus_session(false).unwrap();
        assert_eq!(manager.get_state(), AppState::FocusRunning);
        
        // End session manually (incomplete)
        manager.end_session().unwrap();
        assert_eq!(manager.get_state(), AppState::Idle);
    }

    #[test]
    fn test_cycle_counting() {
        let mut manager = create_test_state_manager();
        assert_eq!(manager.get_cycle_count(), 0);
        
        manager.reset_cycle_count();
        assert_eq!(manager.get_cycle_count(), 0);
    }
}
