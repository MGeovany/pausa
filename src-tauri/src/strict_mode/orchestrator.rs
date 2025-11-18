use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};

use super::models::{StrictModeConfig, StrictModeState, StrictModeWindowType};
use super::system_lock_manager::SystemLockManager;
use crate::cycle_orchestrator::CycleEvent;
use crate::window_manager::WindowManager;

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

        let window_manager = self
            .window_manager
            .lock()
            .map_err(|e| format!("Failed to lock window manager: {}", e))?;

        window_manager
            .show_break_transition()
            .map_err(|e| format!("Failed to show break transition: {}", e))?;

        self.state.current_window_type = Some(StrictModeWindowType::BreakTransition);
        self.save_state_to_database()?;
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
        let window_manager = self
            .window_manager
            .lock()
            .map_err(|e| format!("Failed to lock window manager: {}", e))?;

        window_manager
            .show_break_overlay()
            .map_err(|e| format!("Failed to show break overlay: {}", e))?;

        // Get the break overlay window
        let window = self
            .app_handle
            .get_webview_window("break-overlay")
            .ok_or_else(|| "Break overlay window not found".to_string())?;

        // Lock the system
        let mut lock_manager = self
            .system_lock_manager
            .lock()
            .map_err(|e| format!("Failed to lock system lock manager: {}", e))?;

        lock_manager
            .lock_system(&window)
            .map_err(|e| format!("Failed to lock system: {}", e))?;

        self.state.current_window_type = Some(StrictModeWindowType::FullscreenBreakOverlay);
        self.state.is_locked = true;

        // Save state to database
        self.save_state_to_database()?;

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
            self.unlock_system()?;
        }

        // Hide all strict mode windows
        self.hide_all_strict_windows()?;

        // Restore main window
        let window_manager = self
            .window_manager
            .lock()
            .map_err(|e| format!("Failed to lock window manager: {}", e))?;
        window_manager
            .restore_from_menu_bar()
            .map_err(|e| format!("Failed to restore main window: {}", e))?;

        // Deactivate strict mode
        self.state.is_active = false;
        self.state.current_window_type = None;

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

        let mut lock_manager = self
            .system_lock_manager
            .lock()
            .map_err(|e| format!("Failed to lock system lock manager: {}", e))?;

        lock_manager
            .register_emergency_hotkey(combination.clone())
            .map_err(|e| format!("Failed to register emergency hotkey: {}", e))?;

        // Update config
        self.config.emergency_key_combination = Some(combination);

        println!("✅ [StrictModeOrchestrator] Emergency hotkey registered");
        Ok(())
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
