use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};

use super::models::{StrictModeConfig, StrictModeState, StrictModeWindowType};
use super::system_lock_manager::SystemLockManager;
use crate::cycle_orchestrator::CycleEvent;
use crate::window_manager::WindowManager;

/// Custom error types for StrictModeOrchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrictModeError {
    /// Failed to register emergency hotkey
    HotkeyRegistrationFailed(String),
    /// Failed to create or show a window
    WindowCreationFailed(String),
    /// Failed to lock the system
    SystemLockFailed(String),
    /// State is out of sync with cycle orchestrator
    StateDesynchronization(String),
    /// Emergency exit failed
    EmergencyExitFailed(String),
    /// Database operation failed
    DatabaseError(String),
    /// General error
    General(String),
}

impl std::fmt::Display for StrictModeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StrictModeError::HotkeyRegistrationFailed(msg) => {
                write!(f, "Hotkey registration failed: {}", msg)
            }
            StrictModeError::WindowCreationFailed(msg) => {
                write!(f, "Window creation failed: {}", msg)
            }
            StrictModeError::SystemLockFailed(msg) => write!(f, "System lock failed: {}", msg),
            StrictModeError::StateDesynchronization(msg) => {
                write!(f, "State desynchronization: {}", msg)
            }
            StrictModeError::EmergencyExitFailed(msg) => {
                write!(f, "Emergency exit failed: {}", msg)
            }
            StrictModeError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            StrictModeError::General(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for StrictModeError {}

/// Orchestrates strict mode functionality, managing window transitions and system locks
pub struct StrictModeOrchestrator {
    config: StrictModeConfig,
    state: StrictModeState,
    app_handle: AppHandle,
    window_manager: Arc<Mutex<WindowManager>>,
    system_lock_manager: Arc<Mutex<SystemLockManager>>,
}

impl StrictModeOrchestrator {
    /// Create a new StrictModeOrchestrator
    pub fn new(
        config: StrictModeConfig,
        app_handle: AppHandle,
        window_manager: Arc<Mutex<WindowManager>>,
    ) -> Self {
        let system_lock_manager = Arc::new(Mutex::new(SystemLockManager::new(app_handle.clone())));

        Self {
            config,
            state: StrictModeState::default(),
            app_handle,
            window_manager,
            system_lock_manager,
        }
    }

    /// Activate strict mode
    pub fn activate(&mut self) -> Result<(), String> {
        if self.state.is_active {
            return Err("Strict mode is already active".to_string());
        }

        self.state.is_active = true;

        // Save state to database
        self.save_state_to_database()?;

        println!("✅ [StrictModeOrchestrator] Strict mode activated");

        Ok(())
    }

    /// Deactivate strict mode
    pub fn deactivate(&mut self) -> Result<(), String> {
        if !self.state.is_active {
            return Err("Strict mode is not active".to_string());
        }

        // Clean up any active windows
        if self.state.is_locked {
            self.unlock_system()?;
        }

        // Hide any strict mode windows
        self.hide_all_strict_windows()?;

        self.state.is_active = false;
        self.state.current_window_type = None;

        // Save state to database
        self.save_state_to_database()?;

        println!("✅ [StrictModeOrchestrator] Strict mode deactivated");

        Ok(())
    }

    /// Get the current strict mode state
    pub fn get_state(&self) -> StrictModeState {
        self.state.clone()
    }

    /// Update the configuration
    pub fn update_config(&mut self, config: StrictModeConfig) {
        self.config = config;
    }

    /// Get the current configuration
    pub fn get_config(&self) -> StrictModeConfig {
        self.config.clone()
    }

    /// Check if strict mode is active
    pub fn is_active(&self) -> bool {
        self.state.is_active
    }

    /// Unlock the system
    fn unlock_system(&mut self) -> Result<(), String> {
        println!("🔓 [StrictModeOrchestrator] Unlocking system");

        let mut lock_manager = self
            .system_lock_manager
            .lock()
            .map_err(|e| format!("Failed to lock system lock manager: {}", e))?;

        // Get the break overlay window if it exists
        let window = self.app_handle.get_webview_window("break-overlay");

        lock_manager
            .unlock_system(window.as_ref())
            .map_err(|e| format!("Failed to unlock system: {}", e))?;

        self.state.is_locked = false;

        // Save state to database
        self.save_state_to_database()?;

        println!("✅ [StrictModeOrchestrator] System unlocked");
        Ok(())
    }

    /// Hide all strict mode windows
    fn hide_all_strict_windows(&self) -> Result<(), String> {
        println!("🪟 [StrictModeOrchestrator] Hiding all strict mode windows");

        let window_manager = self
            .window_manager
            .lock()
            .map_err(|e| format!("Failed to lock window manager: {}", e))?;

        // Hide break overlay if it exists
        if let Err(e) = window_manager.hide_break_overlay() {
            eprintln!("Warning: Failed to hide break overlay: {}", e);
        }

        // Hide break transition if it exists
        if let Err(e) = window_manager.hide_break_transition() {
            eprintln!("Warning: Failed to hide break transition: {}", e);
        }

        // Hide menu bar popover if it exists
        if let Err(e) = window_manager.hide_menu_bar_popover() {
            eprintln!("Warning: Failed to hide menu bar popover: {}", e);
        }

        println!("✅ [StrictModeOrchestrator] All strict mode windows hidden");
        Ok(())
    }

    /// Handle cycle events and manage window transitions
    pub fn handle_cycle_event(
        &mut self,
        event: &CycleEvent,
    ) -> Result<Vec<StrictModeEvent>, String> {
        println!(
            "🔔 [StrictModeOrchestrator::handle_cycle_event] Received event: {:?}",
            event
        );
        println!(
            "🔔 [StrictModeOrchestrator::handle_cycle_event] Is active: {}",
            self.state.is_active
        );

        if !self.state.is_active {
            println!("⚠️ [StrictModeOrchestrator::handle_cycle_event] Strict mode is NOT active, ignoring event");
            return Ok(vec![]);
        }

        let mut events = vec![];

        match event {
            CycleEvent::PhaseStarted {
                phase,
                duration,
                cycle_count,
            } => {
                println!(
                    "🎯 [StrictModeOrchestrator] Phase started: {:?}, duration: {}, cycle: {}",
                    phase, duration, cycle_count
                );

                match phase {
                    crate::cycle_orchestrator::CyclePhase::Focus => {
                        // When focus starts in strict mode, minimize to menu bar
                        println!(
                            "📍 [StrictModeOrchestrator] Focus started - minimizing to menu bar"
                        );

                        // First, hide any break overlay if it's showing
                        if self.state.current_window_type
                            == Some(StrictModeWindowType::FullscreenBreakOverlay)
                        {
                            println!("🪟 [StrictModeOrchestrator] Hiding break overlay before starting focus");
                            self.hide_fullscreen_break_overlay()?;
                        }

                        let window_manager = self
                            .window_manager
                            .lock()
                            .map_err(|e| format!("Failed to lock window manager: {}", e))?;

                        println!("📍 [StrictModeOrchestrator] Got window manager lock, calling minimize_to_menu_bar()");

                        window_manager
                            .minimize_to_menu_bar()
                            .map_err(|e| format!("Failed to minimize to menu bar: {}", e))?;

                        println!("✅ [StrictModeOrchestrator] Successfully minimized to menu bar");

                        events.push(StrictModeEvent::MinimizeToMenuBar);
                        self.state.current_window_type = Some(StrictModeWindowType::MenuBarIcon);
                        let _ = self.save_state_to_database();
                    }
                    crate::cycle_orchestrator::CyclePhase::ShortBreak
                    | crate::cycle_orchestrator::CyclePhase::LongBreak => {
                        // When break starts, show transition window
                        println!("☕ [StrictModeOrchestrator] Break starting - showing transition window");

                        self.show_break_transition()?;

                        println!("✅ [StrictModeOrchestrator] Break transition window shown");

                        events.push(StrictModeEvent::ShowBreakTransition);
                    }
                    _ => {
                        println!(
                            "ℹ️ [StrictModeOrchestrator] Phase {:?} - no action needed",
                            phase
                        );
                    }
                }
            }
            CycleEvent::PhaseEnded { phase, completed } => {
                println!(
                    "🏁 [StrictModeOrchestrator] Phase ended: {:?}, completed: {}",
                    phase, completed
                );

                match phase {
                    crate::cycle_orchestrator::CyclePhase::Focus => {
                        // Focus ended, break transition will be shown when break PhaseStarted event fires
                        println!("📍 [StrictModeOrchestrator] Focus ended - break transition will show on break start");
                    }
                    crate::cycle_orchestrator::CyclePhase::ShortBreak => {
                        // Short break ended, hide overlay and return to menu bar
                        // Next focus will auto-start (handled by CycleOrchestrator)
                        println!("☕ [StrictModeOrchestrator] Short break ended - hiding overlay, next focus will auto-start");

                        if self.state.current_window_type
                            == Some(StrictModeWindowType::FullscreenBreakOverlay)
                        {
                            self.hide_fullscreen_break_overlay()?;
                        }

                        events.push(StrictModeEvent::ReturnToMenuBar);
                        self.state.current_window_type = Some(StrictModeWindowType::MenuBarIcon);
                        let _ = self.save_state_to_database();
                    }
                    crate::cycle_orchestrator::CyclePhase::LongBreak => {
                        // Long break ended, hide overlay and return to menu bar
                        // Stay in idle (no auto-start of next focus)
                        println!("☕ [StrictModeOrchestrator] Long break ended - hiding overlay, staying in idle");

                        if self.state.current_window_type
                            == Some(StrictModeWindowType::FullscreenBreakOverlay)
                        {
                            self.hide_fullscreen_break_overlay()?;
                        }

                        events.push(StrictModeEvent::ReturnToMenuBar);
                        self.state.current_window_type = Some(StrictModeWindowType::MenuBarIcon);
                        let _ = self.save_state_to_database();
                    }
                    _ => {}
                }
            }
            _ => {
                // Other events don't require strict mode handling
            }
        }

        Ok(events)
    }

    /// Show the break transition window
    pub fn show_break_transition(&mut self) -> Result<(), String> {
        println!("🪟 [StrictModeOrchestrator] Showing break transition window");

        let result = {
            let window_manager = self
                .window_manager
                .lock()
                .map_err(|e| format!("Failed to lock window manager: {}", e))?;

            window_manager.show_break_transition()
        };

        match result {
            Ok(_) => {
                println!("✅ [StrictModeOrchestrator] Break transition window shown");
            }
            Err(e) => {
                eprintln!(
                    "❌ [StrictModeOrchestrator] Failed to show break transition: {}",
                    e
                );
                return self.handle_error(StrictModeError::WindowCreationFailed(e.to_string()));
            }
        }

        self.state.current_window_type = Some(StrictModeWindowType::BreakTransition);

        if let Err(e) = self.save_state_to_database() {
            eprintln!("⚠️ [StrictModeOrchestrator] Failed to save state: {}", e);
            let _ = self.handle_error(StrictModeError::DatabaseError(e));
        }

        Ok(())
    }

    /// Hide the break transition window
    pub fn hide_break_transition(&mut self) -> Result<(), String> {
        println!("🪟 [StrictModeOrchestrator] Hiding break transition window");

        let window_manager = self
            .window_manager
            .lock()
            .map_err(|e| format!("Failed to lock window manager: {}", e))?;

        window_manager
            .hide_break_transition()
            .map_err(|e| format!("Failed to hide break transition: {}", e))?;

        Ok(())
    }

    /// Start break from transition (after countdown or manual trigger)
    pub fn start_break_from_transition(&mut self) -> Result<(), String> {
        println!("🪟 [StrictModeOrchestrator] Starting break from transition");

        // Hide the break transition window first
        self.hide_break_transition()?;

        // Then show the fullscreen break overlay
        self.show_fullscreen_break_overlay()?;
        Ok(())
    }

    /// Show the fullscreen break overlay with system lock
    pub fn show_fullscreen_break_overlay(&mut self) -> Result<(), String> {
        println!("🪟 [StrictModeOrchestrator] Showing fullscreen break overlay");

        // Show the break overlay window
        let show_result = {
            let window_manager = self
                .window_manager
                .lock()
                .map_err(|e| format!("Failed to lock window manager: {}", e))?;

            window_manager.show_break_overlay()
        };

        match show_result {
            Ok(_) => {
                println!("✅ [StrictModeOrchestrator] Break overlay window shown");
            }
            Err(e) => {
                eprintln!(
                    "❌ [StrictModeOrchestrator] Failed to show break overlay: {}",
                    e
                );
                return self.handle_error(StrictModeError::WindowCreationFailed(e.to_string()));
            }
        }

        // Get the break overlay window
        let window = match self.app_handle.get_webview_window("break-overlay") {
            Some(w) => w,
            None => {
                eprintln!(
                    "❌ [StrictModeOrchestrator] Break overlay window not found after creation"
                );
                return self.handle_error(StrictModeError::WindowCreationFailed(
                    "Break overlay window not found".to_string(),
                ));
            }
        };

        // Lock the system
        let lock_result = {
            let mut lock_manager = self
                .system_lock_manager
                .lock()
                .map_err(|e| format!("Failed to lock system lock manager: {}", e))?;

            lock_manager.lock_system(&window)
        };

        match lock_result {
            Ok(_) => {
                println!("✅ [StrictModeOrchestrator] System locked");
            }
            Err(e) => {
                eprintln!("⚠️ [StrictModeOrchestrator] Failed to lock system: {}", e);
                // Continue without full lock but handle the error
                let _ = self.handle_error(StrictModeError::SystemLockFailed(e));
            }
        }

        self.state.current_window_type = Some(StrictModeWindowType::FullscreenBreakOverlay);
        self.state.is_locked = true;

        // Save state to database
        if let Err(e) = self.save_state_to_database() {
            eprintln!("⚠️ [StrictModeOrchestrator] Failed to save state: {}", e);
            let _ = self.handle_error(StrictModeError::DatabaseError(e));
        }

        println!("✅ [StrictModeOrchestrator] Fullscreen break overlay shown and system locked");
        Ok(())
    }

    /// Hide the fullscreen break overlay and unlock system
    pub fn hide_fullscreen_break_overlay(&mut self) -> Result<(), String> {
        println!("🪟 [StrictModeOrchestrator] Hiding fullscreen break overlay");

        // Unlock the system first
        if self.state.is_locked {
            self.unlock_system()?;
        }

        // Hide the break overlay window
        let window_manager = self
            .window_manager
            .lock()
            .map_err(|e| format!("Failed to lock window manager: {}", e))?;

        window_manager
            .hide_break_overlay()
            .map_err(|e| format!("Failed to hide break overlay: {}", e))?;

        self.state.current_window_type = Some(StrictModeWindowType::MenuBarIcon);

        // Save state to database
        self.save_state_to_database()?;

        println!("✅ [StrictModeOrchestrator] Fullscreen break overlay hidden and system unlocked");
        Ok(())
    }

    /// Show menu bar popover
    pub fn show_menu_bar_popover(&mut self) -> Result<(), String> {
        println!("🪟 [StrictModeOrchestrator] Showing menu bar popover");

        let window_manager = self
            .window_manager
            .lock()
            .map_err(|e| format!("Failed to lock window manager: {}", e))?;

        window_manager
            .show_menu_bar_popover()
            .map_err(|e| format!("Failed to show menu bar popover: {}", e))?;

        self.state.current_window_type = Some(StrictModeWindowType::MenuBarPopover);
        Ok(())
    }

    /// Hide menu bar popover
    pub fn hide_menu_bar_popover(&mut self) -> Result<(), String> {
        println!("🪟 [StrictModeOrchestrator] Hiding menu bar popover");

        let window_manager = self
            .window_manager
            .lock()
            .map_err(|e| format!("Failed to lock window manager: {}", e))?;

        window_manager
            .hide_menu_bar_popover()
            .map_err(|e| format!("Failed to hide menu bar popover: {}", e))?;

        self.state.current_window_type = Some(StrictModeWindowType::MenuBarIcon);
        Ok(())
    }

    /// Emergency exit from strict mode
    pub fn emergency_exit(&mut self) -> Result<(), String> {
        println!("🚨 [StrictModeOrchestrator] Emergency exit triggered");

        // Unlock system immediately
        if self.state.is_locked {
            if let Err(e) = self.unlock_system() {
                eprintln!(
                    "⚠️ [StrictModeOrchestrator] Failed to unlock system during emergency exit: {}",
                    e
                );
                // Try force unlock
                return self.handle_error(StrictModeError::EmergencyExitFailed(e));
            }
        }

        // Hide all strict mode windows
        if let Err(e) = self.hide_all_strict_windows() {
            eprintln!(
                "⚠️ [StrictModeOrchestrator] Failed to hide windows during emergency exit: {}",
                e
            );
            // Continue anyway
        }

        // Restore main window
        let window_manager = self
            .window_manager
            .lock()
            .map_err(|e| format!("Failed to lock window manager: {}", e))?;

        if let Err(e) = window_manager.restore_from_menu_bar() {
            eprintln!(
                "⚠️ [StrictModeOrchestrator] Failed to restore main window: {}",
                e
            );
            // Continue anyway
        }

        drop(window_manager);

        // Deactivate strict mode
        self.state.is_active = false;
        self.state.current_window_type = None;

        // Save state
        if let Err(e) = self.save_state_to_database() {
            eprintln!(
                "⚠️ [StrictModeOrchestrator] Failed to save state during emergency exit: {}",
                e
            );
        }

        println!("✅ [StrictModeOrchestrator] Emergency exit completed");

        Ok(())
    }

    /// Minimize main window to menu bar
    pub fn minimize_to_menu_bar(&mut self) -> Result<(), String> {
        println!("📍 [StrictModeOrchestrator] Minimizing to menu bar");

        let window_manager = self
            .window_manager
            .lock()
            .map_err(|e| format!("Failed to lock window manager: {}", e))?;

        window_manager
            .minimize_to_menu_bar()
            .map_err(|e| format!("Failed to minimize to menu bar: {}", e))?;

        self.state.current_window_type = Some(StrictModeWindowType::MenuBarIcon);
        Ok(())
    }

    /// Restore main window from menu bar
    pub fn restore_from_menu_bar(&mut self) -> Result<(), String> {
        println!("📍 [StrictModeOrchestrator] Restoring from menu bar");

        let window_manager = self
            .window_manager
            .lock()
            .map_err(|e| format!("Failed to lock window manager: {}", e))?;

        window_manager
            .restore_from_menu_bar()
            .map_err(|e| format!("Failed to restore from menu bar: {}", e))?;

        self.state.current_window_type = None;
        Ok(())
    }

    /// Register an emergency hotkey combination
    pub fn register_emergency_hotkey(&mut self, combination: String) -> Result<(), String> {
        println!(
            "🔑 [StrictModeOrchestrator] Registering emergency hotkey: {}",
            combination
        );

        let result = {
            let mut lock_manager = self
                .system_lock_manager
                .lock()
                .map_err(|e| format!("Failed to lock system lock manager: {}", e))?;

            lock_manager.register_emergency_hotkey(combination.clone())
        };

        match result {
            Ok(_) => {
                // Update config
                self.config.emergency_key_combination = Some(combination);
                println!("✅ [StrictModeOrchestrator] Emergency hotkey registered");
                Ok(())
            }
            Err(e) => {
                eprintln!(
                    "❌ [StrictModeOrchestrator] Failed to register emergency hotkey: {}",
                    e
                );
                // Try to register default emergency key as fallback
                self.handle_error(StrictModeError::HotkeyRegistrationFailed(e))
            }
        }
    }

    /// Unregister the emergency hotkey
    pub fn unregister_emergency_hotkey(&mut self) -> Result<(), String> {
        println!("🔑 [StrictModeOrchestrator] Unregistering emergency hotkey");

        let mut lock_manager = self
            .system_lock_manager
            .lock()
            .map_err(|e| format!("Failed to lock system lock manager: {}", e))?;

        lock_manager
            .unregister_emergency_hotkey()
            .map_err(|e| format!("Failed to unregister emergency hotkey: {}", e))?;

        // Update config
        self.config.emergency_key_combination = None;

        println!("✅ [StrictModeOrchestrator] Emergency hotkey unregistered");
        Ok(())
    }

    /// Get the system lock manager (for external access if needed)
    pub fn get_system_lock_manager(&self) -> Arc<Mutex<SystemLockManager>> {
        self.system_lock_manager.clone()
    }

    /// Save the current strict mode state to the database
    pub fn save_state_to_database(&self) -> Result<(), String> {
        println!("💾 [StrictModeOrchestrator] Saving state to database");

        // Get database connection from app handle
        let app_state = self
            .app_handle
            .try_state::<crate::state::AppState>()
            .ok_or_else(|| "Failed to get app state".to_string())?;

        // Convert window type to string
        let window_type_str = self.state.current_window_type.as_ref().map(|wt| match wt {
            StrictModeWindowType::MenuBarIcon => "menu_bar_icon",
            StrictModeWindowType::MenuBarPopover => "menu_bar_popover",
            StrictModeWindowType::BreakTransition => "break_transition",
            StrictModeWindowType::FullscreenBreakOverlay => "fullscreen_break_overlay",
        });

        let is_active = self.state.is_active;
        let is_locked = self.state.is_locked;

        // Update the strict mode state in the database
        app_state
            .database
            .with_connection(|conn| {
                conn.execute(
                    "UPDATE strict_mode_state SET is_active = ?, is_locked = ?, current_window_type = ?, updated_at = CURRENT_TIMESTAMP WHERE id = 1",
                    rusqlite::params![is_active, is_locked, window_type_str],
                )
                .map_err(crate::database::DatabaseError::Sqlite)
            })
            .map_err(|e| format!("Failed to save strict mode state: {}", e))?;

        println!("✅ [StrictModeOrchestrator] State saved to database");
        Ok(())
    }

    /// Restore strict mode state from the database
    pub fn restore_state_from_database(&mut self) -> Result<(), String> {
        println!("📂 [StrictModeOrchestrator] Restoring state from database");

        // Get database connection from app handle
        let app_state = self
            .app_handle
            .try_state::<crate::state::AppState>()
            .ok_or_else(|| "Failed to get app state".to_string())?;

        // Query the strict mode state from the database
        let (is_active, is_locked, window_type_str): (bool, bool, Option<String>) = app_state
            .database
            .with_connection(|conn| {
                conn.query_row(
                    "SELECT is_active, is_locked, current_window_type FROM strict_mode_state WHERE id = 1",
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .map_err(crate::database::DatabaseError::Sqlite)
            })
            .map_err(|e| format!("Failed to restore strict mode state: {}", e))?;

        // Convert string to window type
        let window_type = window_type_str.and_then(|s| match s.as_str() {
            "menu_bar_icon" => Some(StrictModeWindowType::MenuBarIcon),
            "menu_bar_popover" => Some(StrictModeWindowType::MenuBarPopover),
            "break_transition" => Some(StrictModeWindowType::BreakTransition),
            "fullscreen_break_overlay" => Some(StrictModeWindowType::FullscreenBreakOverlay),
            _ => None,
        });

        // Update the orchestrator state
        self.state.is_active = is_active;
        self.state.is_locked = is_locked;
        self.state.current_window_type = window_type.clone();

        println!(
            "✅ [StrictModeOrchestrator] State restored: is_active={}, is_locked={}, window_type={:?}",
            is_active, is_locked, window_type
        );

        // If strict mode was active and locked when the app closed, we need to handle recovery
        if is_active && is_locked {
            println!("⚠️ [StrictModeOrchestrator] App was closed during locked state - performing recovery");

            // Unlock the system
            if let Err(e) = self.unlock_system() {
                eprintln!("Failed to unlock system during recovery: {}", e);
            }

            // Hide any strict mode windows
            if let Err(e) = self.hide_all_strict_windows() {
                eprintln!("Failed to hide strict mode windows during recovery: {}", e);
            }

            // Reset to menu bar icon state
            self.state.is_locked = false;
            self.state.current_window_type = Some(StrictModeWindowType::MenuBarIcon);

            // Save the recovered state
            if let Err(e) = self.save_state_to_database() {
                eprintln!("Failed to save recovered state: {}", e);
            }
        }

        Ok(())
    }

    /// Handle errors with appropriate fallback strategies
    fn handle_error(&mut self, error: StrictModeError) -> Result<(), String> {
        eprintln!("🚨 [StrictModeOrchestrator] Handling error: {}", error);

        match error {
            StrictModeError::HotkeyRegistrationFailed(msg) => {
                eprintln!(
                    "⚠️ [StrictModeOrchestrator] Hotkey registration failed: {}",
                    msg
                );
                // Try to register default emergency key as fallback
                match self.register_default_emergency_key() {
                    Ok(_) => {
                        println!("✅ [StrictModeOrchestrator] Registered default emergency key as fallback");
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("❌ [StrictModeOrchestrator] Failed to register default emergency key: {}", e);
                        Err(format!("Failed to register emergency hotkey: {}", msg))
                    }
                }
            }
            StrictModeError::WindowCreationFailed(msg) => {
                eprintln!(
                    "⚠️ [StrictModeOrchestrator] Window creation failed: {}",
                    msg
                );
                // Deactivate strict mode temporarily
                if let Err(e) = self.safe_deactivate() {
                    eprintln!(
                        "❌ [StrictModeOrchestrator] Failed to deactivate strict mode: {}",
                        e
                    );
                }
                Err(format!(
                    "Strict mode disabled due to window creation failure: {}",
                    msg
                ))
            }
            StrictModeError::SystemLockFailed(msg) => {
                eprintln!("⚠️ [StrictModeOrchestrator] System lock failed: {}", msg);
                // Continue without full lock but log warning
                println!("⚠️ [StrictModeOrchestrator] Continuing without full system lock");
                Ok(())
            }
            StrictModeError::StateDesynchronization(msg) => {
                eprintln!(
                    "⚠️ [StrictModeOrchestrator] State desynchronization: {}",
                    msg
                );
                // Try to sync with cycle orchestrator
                if let Err(e) = self.sync_state() {
                    eprintln!("❌ [StrictModeOrchestrator] Failed to sync state: {}", e);
                }
                Ok(())
            }
            StrictModeError::EmergencyExitFailed(msg) => {
                eprintln!("🚨 [StrictModeOrchestrator] Emergency exit failed: {}", msg);
                // Force unlock the system
                if let Err(e) = self.force_unlock_and_cleanup() {
                    eprintln!("❌ [StrictModeOrchestrator] Failed to force unlock: {}", e);
                    return Err(format!(
                        "Critical: Emergency exit failed and force unlock failed: {}",
                        e
                    ));
                }
                Ok(())
            }
            StrictModeError::DatabaseError(msg) => {
                eprintln!("⚠️ [StrictModeOrchestrator] Database error: {}", msg);
                // Continue operation but log error
                println!("⚠️ [StrictModeOrchestrator] Continuing without database persistence");
                Ok(())
            }
            StrictModeError::General(msg) => {
                eprintln!("⚠️ [StrictModeOrchestrator] General error: {}", msg);
                Err(msg)
            }
        }
    }

    /// Register default emergency key combination (Cmd+Shift+Esc)
    fn register_default_emergency_key(&mut self) -> Result<(), String> {
        println!("🔑 [StrictModeOrchestrator] Registering default emergency key: Cmd+Shift+Esc");

        let default_combination = "Cmd+Shift+Esc".to_string();

        let mut lock_manager = self
            .system_lock_manager
            .lock()
            .map_err(|e| format!("Failed to lock system lock manager: {}", e))?;

        lock_manager
            .register_emergency_hotkey(default_combination.clone())
            .map_err(|e| format!("Failed to register default emergency hotkey: {}", e))?;

        // Update config
        self.config.emergency_key_combination = Some(default_combination);

        println!("✅ [StrictModeOrchestrator] Default emergency key registered");
        Ok(())
    }

    /// Safely deactivate strict mode without throwing errors
    fn safe_deactivate(&mut self) -> Result<(), String> {
        println!("🛑 [StrictModeOrchestrator] Safely deactivating strict mode");

        // Try to unlock system if locked
        if self.state.is_locked {
            if let Err(e) = self.unlock_system() {
                eprintln!("⚠️ [StrictModeOrchestrator] Failed to unlock system during safe deactivation: {}", e);
                // Continue anyway
            }
        }

        // Try to hide all windows
        if let Err(e) = self.hide_all_strict_windows() {
            eprintln!(
                "⚠️ [StrictModeOrchestrator] Failed to hide windows during safe deactivation: {}",
                e
            );
            // Continue anyway
        }

        // Update state
        self.state.is_active = false;
        self.state.current_window_type = None;

        // Try to save state
        if let Err(e) = self.save_state_to_database() {
            eprintln!(
                "⚠️ [StrictModeOrchestrator] Failed to save state during safe deactivation: {}",
                e
            );
            // Continue anyway
        }

        println!("✅ [StrictModeOrchestrator] Strict mode safely deactivated");
        Ok(())
    }

    /// Sync state with cycle orchestrator
    fn sync_state(&mut self) -> Result<(), String> {
        println!("🔄 [StrictModeOrchestrator] Syncing state");

        // Reset to a known good state
        if self.state.is_locked {
            if let Err(e) = self.unlock_system() {
                eprintln!(
                    "⚠️ [StrictModeOrchestrator] Failed to unlock during sync: {}",
                    e
                );
            }
        }

        // Hide all windows
        if let Err(e) = self.hide_all_strict_windows() {
            eprintln!(
                "⚠️ [StrictModeOrchestrator] Failed to hide windows during sync: {}",
                e
            );
        }

        // Reset to menu bar icon if active
        if self.state.is_active {
            self.state.current_window_type = Some(StrictModeWindowType::MenuBarIcon);
        } else {
            self.state.current_window_type = None;
        }

        // Save synced state
        if let Err(e) = self.save_state_to_database() {
            eprintln!(
                "⚠️ [StrictModeOrchestrator] Failed to save synced state: {}",
                e
            );
        }

        println!("✅ [StrictModeOrchestrator] State synced");
        Ok(())
    }

    /// Force unlock and cleanup (used in critical error situations)
    fn force_unlock_and_cleanup(&mut self) -> Result<(), String> {
        println!("🚨 [StrictModeOrchestrator] Force unlocking and cleaning up");

        // Force unlock the system
        let mut lock_manager = self
            .system_lock_manager
            .lock()
            .map_err(|e| format!("Failed to lock system lock manager: {}", e))?;

        if let Err(e) = lock_manager.force_unlock() {
            eprintln!("❌ [StrictModeOrchestrator] Failed to force unlock: {}", e);
        }

        drop(lock_manager);

        // Update state
        self.state.is_locked = false;

        // Try to hide all windows (best effort)
        let _ = self.hide_all_strict_windows();

        // Try to restore main window
        let window_manager = self
            .window_manager
            .lock()
            .map_err(|e| format!("Failed to lock window manager: {}", e))?;

        if let Err(e) = window_manager.restore_from_menu_bar() {
            eprintln!(
                "⚠️ [StrictModeOrchestrator] Failed to restore main window: {}",
                e
            );
        }

        drop(window_manager);

        // Deactivate strict mode
        self.state.is_active = false;
        self.state.current_window_type = None;

        // Try to save state
        let _ = self.save_state_to_database();

        println!("✅ [StrictModeOrchestrator] Force unlock and cleanup completed");
        Ok(())
    }

    /// Handle monitor change during strict mode
    /// This ensures the break overlay remains fullscreen on the current monitor
    pub fn handle_monitor_change(&mut self) -> Result<(), String> {
        println!("🖥️ [StrictModeOrchestrator] Handling monitor change");

        // Only relevant if we're showing the break overlay
        if self.state.current_window_type != Some(StrictModeWindowType::FullscreenBreakOverlay) {
            println!("ℹ️ [StrictModeOrchestrator] Not showing break overlay, no action needed");
            return Ok(());
        }

        // Get the break overlay window
        if let Some(window) = self.app_handle.get_webview_window("break-overlay") {
            println!("🖥️ [StrictModeOrchestrator] Refreshing break overlay window properties");

            // Re-apply fullscreen and always-on-top properties
            if let Err(e) = window.set_fullscreen(true) {
                eprintln!(
                    "⚠️ [StrictModeOrchestrator] Failed to re-apply fullscreen: {}",
                    e
                );
            }

            if let Err(e) = window.set_always_on_top(true) {
                eprintln!(
                    "⚠️ [StrictModeOrchestrator] Failed to re-apply always-on-top: {}",
                    e
                );
            }

            if let Err(e) = window.set_focus() {
                eprintln!(
                    "⚠️ [StrictModeOrchestrator] Failed to re-focus window: {}",
                    e
                );
            }

            println!("✅ [StrictModeOrchestrator] Monitor change handled");
        } else {
            eprintln!("⚠️ [StrictModeOrchestrator] Break overlay window not found");
        }

        Ok(())
    }

    /// Validate state consistency
    /// This checks if the internal state matches the actual window state
    pub fn validate_state(&self) -> Result<(), String> {
        println!("🔍 [StrictModeOrchestrator] Validating state consistency");

        // Check if locked state matches window existence
        if self.state.is_locked {
            if self
                .app_handle
                .get_webview_window("break-overlay")
                .is_none()
            {
                eprintln!(
                    "⚠️ [StrictModeOrchestrator] State says locked but break overlay doesn't exist"
                );
                return Err("State inconsistency: locked but no break overlay".to_string());
            }
        }

        // Check if current window type matches actual windows
        match &self.state.current_window_type {
            Some(StrictModeWindowType::BreakTransition) => {
                if self
                    .app_handle
                    .get_webview_window("break-transition")
                    .is_none()
                {
                    eprintln!("⚠️ [StrictModeOrchestrator] State says break transition but window doesn't exist");
                    return Err("State inconsistency: break transition window missing".to_string());
                }
            }
            Some(StrictModeWindowType::FullscreenBreakOverlay) => {
                if self
                    .app_handle
                    .get_webview_window("break-overlay")
                    .is_none()
                {
                    eprintln!("⚠️ [StrictModeOrchestrator] State says break overlay but window doesn't exist");
                    return Err("State inconsistency: break overlay window missing".to_string());
                }
            }
            Some(StrictModeWindowType::MenuBarPopover) => {
                if self
                    .app_handle
                    .get_webview_window("menu-bar-popover")
                    .is_none()
                {
                    eprintln!("⚠️ [StrictModeOrchestrator] State says menu bar popover but window doesn't exist");
                    return Err("State inconsistency: menu bar popover window missing".to_string());
                }
            }
            _ => {}
        }

        println!("✅ [StrictModeOrchestrator] State is consistent");
        Ok(())
    }

    /// Get diagnostic information for debugging
    /// Returns a detailed snapshot of the current state
    pub fn get_diagnostics(&self) -> StrictModeDiagnostics {
        println!("📊 [StrictModeOrchestrator] Generating diagnostics");

        let windows_status = StrictModeWindowsStatus {
            main_window_exists: self.app_handle.get_webview_window("main").is_some(),
            break_overlay_exists: self
                .app_handle
                .get_webview_window("break-overlay")
                .is_some(),
            break_transition_exists: self
                .app_handle
                .get_webview_window("break-transition")
                .is_some(),
            menu_bar_popover_exists: self
                .app_handle
                .get_webview_window("menu-bar-popover")
                .is_some(),
        };

        let lock_manager_status = {
            if let Ok(lock_manager) = self.system_lock_manager.lock() {
                Some(StrictModeLockStatus {
                    is_locked: lock_manager.is_locked(),
                    emergency_hotkey: lock_manager.get_emergency_hotkey(),
                })
            } else {
                None
            }
        };

        StrictModeDiagnostics {
            state: self.state.clone(),
            config: self.config.clone(),
            windows_status,
            lock_manager_status,
            state_validation: self.validate_state().is_ok(),
        }
    }

    /// Log current state for debugging
    pub fn log_state(&self) {
        println!("📋 [StrictModeOrchestrator] === Current State ===");
        println!("  is_active: {}", self.state.is_active);
        println!("  is_locked: {}", self.state.is_locked);
        println!(
            "  current_window_type: {:?}",
            self.state.current_window_type
        );
        println!(
            "  emergency_key: {:?}",
            self.config.emergency_key_combination
        );
        println!(
            "  transition_countdown: {}",
            self.config.transition_countdown_seconds
        );

        // Log window existence
        println!("📋 [StrictModeOrchestrator] === Windows Status ===");
        println!(
            "  main: {}",
            self.app_handle.get_webview_window("main").is_some()
        );
        println!(
            "  break-overlay: {}",
            self.app_handle
                .get_webview_window("break-overlay")
                .is_some()
        );
        println!(
            "  break-transition: {}",
            self.app_handle
                .get_webview_window("break-transition")
                .is_some()
        );
        println!(
            "  menu-bar-popover: {}",
            self.app_handle
                .get_webview_window("menu-bar-popover")
                .is_some()
        );

        // Log lock manager status
        if let Ok(lock_manager) = self.system_lock_manager.lock() {
            println!("📋 [StrictModeOrchestrator] === Lock Manager Status ===");
            println!("  is_locked: {}", lock_manager.is_locked());
            println!(
                "  emergency_hotkey: {:?}",
                lock_manager.get_emergency_hotkey()
            );
        }

        println!("📋 [StrictModeOrchestrator] === End State ===");
    }
}

/// Events emitted by the StrictModeOrchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StrictModeEvent {
    MinimizeToMenuBar,
    ShowBreakTransition,
    ShowBreakOverlay,
    ReturnToMenuBar,
    EmergencyExit,
}

/// Diagnostic information for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrictModeDiagnostics {
    pub state: StrictModeState,
    pub config: StrictModeConfig,
    pub windows_status: StrictModeWindowsStatus,
    pub lock_manager_status: Option<StrictModeLockStatus>,
    pub state_validation: bool,
}

/// Status of strict mode windows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrictModeWindowsStatus {
    pub main_window_exists: bool,
    pub break_overlay_exists: bool,
    pub break_transition_exists: bool,
    pub menu_bar_popover_exists: bool,
}

/// Status of the lock manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrictModeLockStatus {
    pub is_locked: bool,
    pub emergency_hotkey: Option<String>,
}
