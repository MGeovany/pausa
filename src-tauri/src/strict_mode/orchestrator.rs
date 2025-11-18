use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::AppHandle;

use super::models::{StrictModeConfig, StrictModeState, StrictModeWindowType};
use crate::cycle_orchestrator::CycleEvent;
use crate::window_manager::WindowManager;

/// Orchestrates strict mode functionality, managing window transitions and system locks
pub struct StrictModeOrchestrator {
    config: StrictModeConfig,
    state: StrictModeState,
    app_handle: AppHandle,
    window_manager: Arc<Mutex<WindowManager>>,
}

impl StrictModeOrchestrator {
    /// Create a new StrictModeOrchestrator
    pub fn new(
        config: StrictModeConfig,
        app_handle: AppHandle,
        window_manager: Arc<Mutex<WindowManager>>,
    ) -> Self {
        Self {
            config,
            state: StrictModeState::default(),
            app_handle,
            window_manager,
        }
    }

    /// Activate strict mode
    pub fn activate(&mut self) -> Result<(), String> {
        if self.state.is_active {
            return Err("Strict mode is already active".to_string());
        }

        self.state.is_active = true;

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

    /// Unlock the system (placeholder for future implementation)
    fn unlock_system(&mut self) -> Result<(), String> {
        self.state.is_locked = false;
        println!("🔓 [StrictModeOrchestrator] System unlocked");
        Ok(())
    }

    /// Hide all strict mode windows (placeholder for future implementation)
    fn hide_all_strict_windows(&self) -> Result<(), String> {
        println!("🪟 [StrictModeOrchestrator] Hiding all strict mode windows");
        Ok(())
    }

    /// Handle cycle events and manage window transitions
    pub fn handle_cycle_event(
        &mut self,
        event: &CycleEvent,
    ) -> Result<Vec<StrictModeEvent>, String> {
        if !self.state.is_active {
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

                        let window_manager = self
                            .window_manager
                            .lock()
                            .map_err(|e| format!("Failed to lock window manager: {}", e))?;

                        window_manager
                            .minimize_to_menu_bar()
                            .map_err(|e| format!("Failed to minimize to menu bar: {}", e))?;

                        events.push(StrictModeEvent::MinimizeToMenuBar);
                        self.state.current_window_type = Some(StrictModeWindowType::MenuBarIcon);
                    }
                    crate::cycle_orchestrator::CyclePhase::ShortBreak
                    | crate::cycle_orchestrator::CyclePhase::LongBreak => {
                        // When break starts, show transition window
                        println!("☕ [StrictModeOrchestrator] Break starting - should show transition window");
                        events.push(StrictModeEvent::ShowBreakTransition);
                        self.state.current_window_type =
                            Some(StrictModeWindowType::BreakTransition);
                    }
                    _ => {}
                }
            }
            CycleEvent::PhaseEnded { phase, completed } => {
                println!(
                    "🏁 [StrictModeOrchestrator] Phase ended: {:?}, completed: {}",
                    phase, completed
                );

                match phase {
                    crate::cycle_orchestrator::CyclePhase::Focus => {
                        // Focus ended, prepare for break transition
                        println!("📍 [StrictModeOrchestrator] Focus ended - preparing for break");
                    }
                    crate::cycle_orchestrator::CyclePhase::ShortBreak
                    | crate::cycle_orchestrator::CyclePhase::LongBreak => {
                        // Break ended, return to menu bar
                        println!("☕ [StrictModeOrchestrator] Break ended - returning to menu bar");
                        events.push(StrictModeEvent::ReturnToMenuBar);
                        self.state.current_window_type = Some(StrictModeWindowType::MenuBarIcon);
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
        self.state.current_window_type = Some(StrictModeWindowType::BreakTransition);
        // Window creation will be handled by WindowManager in future tasks
        Ok(())
    }

    /// Start break from transition (after countdown or manual trigger)
    pub fn start_break_from_transition(&mut self) -> Result<(), String> {
        println!("🪟 [StrictModeOrchestrator] Starting break from transition");
        self.show_fullscreen_break_overlay()?;
        Ok(())
    }

    /// Show the fullscreen break overlay with system lock
    pub fn show_fullscreen_break_overlay(&mut self) -> Result<(), String> {
        println!("🪟 [StrictModeOrchestrator] Showing fullscreen break overlay");
        self.state.current_window_type = Some(StrictModeWindowType::FullscreenBreakOverlay);
        self.state.is_locked = true;
        // Window creation and system lock will be handled in future tasks
        Ok(())
    }

    /// Hide the fullscreen break overlay
    pub fn hide_fullscreen_break_overlay(&mut self) -> Result<(), String> {
        println!("🪟 [StrictModeOrchestrator] Hiding fullscreen break overlay");
        self.state.is_locked = false;
        self.state.current_window_type = Some(StrictModeWindowType::MenuBarIcon);
        // Window hiding will be handled by WindowManager in future tasks
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
