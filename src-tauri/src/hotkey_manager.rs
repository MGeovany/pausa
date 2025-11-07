use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;
// use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use crate::api_models::BreakType;
use crate::state_manager::{AppState, StateEvent, StateManager};
use crate::window_manager::WindowManager;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HotkeyAction {
    ToggleCommandPalette,
    ToggleFocusSession,
    ImmediateLock,
}

impl HotkeyAction {
    pub fn default_shortcut(&self) -> Shortcut {
        match self {
            HotkeyAction::ToggleCommandPalette => {
                #[cfg(target_os = "macos")]
                return Shortcut::new(Some(Modifiers::META), Code::Space);
                #[cfg(not(target_os = "macos"))]
                return Shortcut::new(Some(Modifiers::CONTROL), Code::Space);
            }
            HotkeyAction::ToggleFocusSession => {
                #[cfg(target_os = "macos")]
                return Shortcut::new(Some(Modifiers::META | Modifiers::SHIFT), Code::KeyF);
                #[cfg(not(target_os = "macos"))]
                return Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyF);
            }
            HotkeyAction::ImmediateLock => {
                #[cfg(target_os = "macos")]
                return Shortcut::new(Some(Modifiers::META | Modifiers::SHIFT), Code::KeyL);
                #[cfg(not(target_os = "macos"))]
                return Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyL);
            }
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            HotkeyAction::ToggleCommandPalette => "Toggle Command Palette",
            HotkeyAction::ToggleFocusSession => "Toggle Focus Session",
            HotkeyAction::ImmediateLock => "Immediate Lock/Break",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub action: HotkeyAction,
    pub shortcut: Shortcut,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyEventResult {
    pub action: HotkeyAction,
    pub success: bool,
    pub message: String,
    pub state_changes: Vec<StateEvent>,
}

impl HotkeyConfig {
    pub fn new(action: HotkeyAction) -> Self {
        let shortcut = action.default_shortcut();
        Self {
            action,
            shortcut,
            enabled: true,
        }
    }

    pub fn with_shortcut(action: HotkeyAction, shortcut: Shortcut) -> Self {
        Self {
            action,
            shortcut,
            enabled: true,
        }
    }
}

pub struct HotkeyManager {
    app_handle: AppHandle,
    registered_hotkeys: Arc<Mutex<HashMap<HotkeyAction, HotkeyConfig>>>,
    state_manager: Arc<Mutex<StateManager>>,
    window_manager: Arc<Mutex<WindowManager>>,
}

impl HotkeyManager {
    pub fn new(
        app_handle: AppHandle,
        state_manager: Arc<Mutex<StateManager>>,
        window_manager: Arc<Mutex<WindowManager>>,
    ) -> Self {
        Self {
            app_handle,
            registered_hotkeys: Arc::new(Mutex::new(HashMap::new())),
            state_manager,
            window_manager,
        }
    }

    /// Initialize the hotkey manager with default hotkeys
    pub fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        let default_actions = vec![
            HotkeyAction::ToggleCommandPalette,
            HotkeyAction::ToggleFocusSession,
            HotkeyAction::ImmediateLock,
        ];

        for action in default_actions {
            let config = HotkeyConfig::new(action.clone());
            self.register_hotkey(config)?;
        }

        println!("Hotkey manager initialized with default hotkeys");
        Ok(())
    }

    /// Register a hotkey with the system
    pub fn register_hotkey(&self, config: HotkeyConfig) -> Result<(), Box<dyn std::error::Error>> {
        if !config.enabled {
            return Ok(());
        }

        // Unregister existing hotkey if it exists
        if let Ok(hotkeys) = self.registered_hotkeys.lock() {
            if let Some(existing_config) = hotkeys.get(&config.action) {
                if let Err(e) = self
                    .app_handle
                    .global_shortcut()
                    .unregister(existing_config.shortcut.clone())
                {
                    eprintln!("Warning: Failed to unregister existing hotkey: {}", e);
                }
            }
        }

        // Create closure for hotkey handler
        let action = config.action.clone();
        let state_manager = Arc::clone(&self.state_manager);
        let window_manager = Arc::clone(&self.window_manager);

        // Register the new hotkey
        self.app_handle.global_shortcut().on_shortcut(
            config.shortcut.clone(),
            move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    if let Err(e) = Self::handle_hotkey_event(
                        action.clone(),
                        Arc::clone(&state_manager),
                        Arc::clone(&window_manager),
                    ) {
                        eprintln!("Error handling hotkey event: {}", e);
                    }
                }
            },
        )?;

        // Store the configuration
        if let Ok(mut hotkeys) = self.registered_hotkeys.lock() {
            hotkeys.insert(config.action.clone(), config.clone());
        }

        println!("Registered hotkey for action: {:?}", config.action);
        Ok(())
    }

    /// Unregister a hotkey
    pub fn unregister_hotkey(
        &self,
        action: &HotkeyAction,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(mut hotkeys) = self.registered_hotkeys.lock() {
            if let Some(config) = hotkeys.remove(action) {
                self.app_handle
                    .global_shortcut()
                    .unregister(config.shortcut.clone())?;
                println!("Unregistered hotkey for action: {:?}", action);
            }
        }
        Ok(())
    }

    /// Update a hotkey configuration
    pub fn update_hotkey(&self, config: HotkeyConfig) -> Result<(), Box<dyn std::error::Error>> {
        // Unregister the old hotkey
        self.unregister_hotkey(&config.action)?;

        // Register the new hotkey
        self.register_hotkey(config)?;

        Ok(())
    }

    /// Enable or disable a hotkey
    pub fn set_hotkey_enabled(
        &self,
        action: &HotkeyAction,
        enabled: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(mut hotkeys) = self.registered_hotkeys.lock() {
            if let Some(config) = hotkeys.get_mut(action) {
                if config.enabled == enabled {
                    return Ok(()); // No change needed
                }

                config.enabled = enabled;

                if enabled {
                    // Re-register the hotkey
                    let config_clone = config.clone();
                    drop(hotkeys); // Release the lock before calling register_hotkey
                    self.register_hotkey(config_clone)?;
                } else {
                    // Unregister the hotkey
                    let shortcut = config.shortcut.clone();
                    drop(hotkeys); // Release the lock before calling unregister
                    self.app_handle.global_shortcut().unregister(shortcut)?;
                }
            }
        }
        Ok(())
    }

    /// Get all registered hotkey configurations
    pub fn get_hotkey_configs(&self) -> HashMap<HotkeyAction, HotkeyConfig> {
        if let Ok(hotkeys) = self.registered_hotkeys.lock() {
            hotkeys.clone()
        } else {
            HashMap::new()
        }
    }

    /// Check if hotkeys should be enabled based on app state
    pub fn update_hotkey_state_based_on_app_state(&self) -> Result<(), Box<dyn std::error::Error>> {
        let app_state = if let Ok(state_manager) = self.state_manager.lock() {
            state_manager.get_state()
        } else {
            return Err("Failed to lock state manager".into());
        };

        // Determine which hotkeys should be enabled based on app state
        let command_palette_enabled = true; // Always enabled
        let focus_session_enabled = matches!(
            app_state,
            AppState::Idle | AppState::FocusRunning | AppState::FocusPaused
        );
        let immediate_lock_enabled = !matches!(
            app_state,
            AppState::BreakRunning | AppState::LongBreakRunning
        );

        // Update hotkey states
        self.set_hotkey_enabled(&HotkeyAction::ToggleCommandPalette, command_palette_enabled)?;
        self.set_hotkey_enabled(&HotkeyAction::ToggleFocusSession, focus_session_enabled)?;
        self.set_hotkey_enabled(&HotkeyAction::ImmediateLock, immediate_lock_enabled)?;

        println!("Updated hotkey states based on app state: {:?}", app_state);
        Ok(())
    }

    /// Route hotkey events to appropriate handlers based on current context
    pub fn route_hotkey_event(
        &self,
        action: HotkeyAction,
    ) -> Result<Vec<HotkeyEventResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();

        match action {
            HotkeyAction::ToggleCommandPalette => {
                results.push(self.handle_command_palette_toggle()?);
            }
            HotkeyAction::ToggleFocusSession => {
                results.extend(self.handle_focus_session_toggle()?);
            }
            HotkeyAction::ImmediateLock => {
                results.extend(self.handle_immediate_lock()?);
            }
        }

        Ok(results)
    }

    /// Handle command palette toggle with context awareness
    fn handle_command_palette_toggle(
        &self,
    ) -> Result<HotkeyEventResult, Box<dyn std::error::Error>> {
        if let Ok(window_manager) = self.window_manager.lock() {
            let was_visible =
                window_manager.is_window_visible(crate::window_manager::WindowType::CommandPalette);

            window_manager.toggle_command_palette()?;

            Ok(HotkeyEventResult {
                action: HotkeyAction::ToggleCommandPalette,
                success: true,
                message: if was_visible {
                    "Command palette hidden".to_string()
                } else {
                    "Command palette shown".to_string()
                },
                state_changes: vec![],
            })
        } else {
            Err("Failed to access window manager".into())
        }
    }

    /// Handle focus session toggle with state-aware logic
    fn handle_focus_session_toggle(
        &self,
    ) -> Result<Vec<HotkeyEventResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();

        if let Ok(mut state_manager) = self.state_manager.lock() {
            let current_state = state_manager.get_state();

            match current_state {
                AppState::Idle => {
                    // Start a new focus session
                    let events = state_manager.start_focus_session(false)?;

                    // Show focus widget
                    if let Ok(window_manager) = self.window_manager.lock() {
                        window_manager.show_focus_widget()?;
                    }

                    results.push(HotkeyEventResult {
                        action: HotkeyAction::ToggleFocusSession,
                        success: true,
                        message: "Focus session started".to_string(),
                        state_changes: events,
                    });
                }
                AppState::FocusRunning => {
                    // Pause the current session
                    let events = state_manager.pause_session()?;

                    results.push(HotkeyEventResult {
                        action: HotkeyAction::ToggleFocusSession,
                        success: true,
                        message: "Focus session paused".to_string(),
                        state_changes: events,
                    });
                }
                AppState::FocusPaused => {
                    // Resume the current session
                    let events = state_manager.resume_session()?;

                    results.push(HotkeyEventResult {
                        action: HotkeyAction::ToggleFocusSession,
                        success: true,
                        message: "Focus session resumed".to_string(),
                        state_changes: events,
                    });
                }
                _ => {
                    results.push(HotkeyEventResult {
                        action: HotkeyAction::ToggleFocusSession,
                        success: false,
                        message: format!(
                            "Cannot toggle focus session in state: {:?}",
                            current_state
                        ),
                        state_changes: vec![],
                    });
                }
            }
        } else {
            return Err("Failed to access state manager".into());
        }

        Ok(results)
    }

    /// Handle immediate lock with context-aware break type selection
    fn handle_immediate_lock(&self) -> Result<Vec<HotkeyEventResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();

        if let Ok(mut state_manager) = self.state_manager.lock() {
            let current_state = state_manager.get_state();

            match current_state {
                AppState::FocusRunning | AppState::FocusPaused => {
                    // End current focus session and start break
                    let end_events = state_manager.end_session()?;

                    // Show break overlay
                    if let Ok(window_manager) = self.window_manager.lock() {
                        window_manager.show_break_overlay()?;
                    }

                    results.push(HotkeyEventResult {
                        action: HotkeyAction::ImmediateLock,
                        success: true,
                        message: "Focus session ended, break started".to_string(),
                        state_changes: end_events,
                    });
                }
                AppState::Idle => {
                    // Start immediate short break
                    let events = state_manager.start_break(BreakType::Short)?;

                    // Show break overlay
                    if let Ok(window_manager) = self.window_manager.lock() {
                        window_manager.show_break_overlay()?;
                    }

                    results.push(HotkeyEventResult {
                        action: HotkeyAction::ImmediateLock,
                        success: true,
                        message: "Immediate break started".to_string(),
                        state_changes: events,
                    });
                }
                _ => {
                    results.push(HotkeyEventResult {
                        action: HotkeyAction::ImmediateLock,
                        success: false,
                        message: format!(
                            "Cannot start immediate lock in state: {:?}",
                            current_state
                        ),
                        state_changes: vec![],
                    });
                }
            }
        } else {
            return Err("Failed to access state manager".into());
        }

        Ok(results)
    }

    /// Handle hotkey events (simplified version for callback)
    fn handle_hotkey_event(
        action: HotkeyAction,
        state_manager: Arc<Mutex<StateManager>>,
        window_manager: Arc<Mutex<WindowManager>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match action {
            HotkeyAction::ToggleCommandPalette => {
                if let Ok(manager) = window_manager.lock() {
                    manager.toggle_command_palette()?;
                }
            }
            HotkeyAction::ToggleFocusSession => {
                if let Ok(mut state_mgr) = state_manager.lock() {
                    let current_state = state_mgr.get_state();
                    match current_state {
                        AppState::Idle => {
                            let _events = state_mgr.start_focus_session(false)?;
                            if let Ok(window_mgr) = window_manager.lock() {
                                window_mgr.show_focus_widget()?;
                            }
                        }
                        AppState::FocusRunning => {
                            let _events = state_mgr.pause_session()?;
                        }
                        AppState::FocusPaused => {
                            let _events = state_mgr.resume_session()?;
                        }
                        _ => {
                            println!("Cannot toggle focus session in state: {:?}", current_state);
                        }
                    }
                }
            }
            HotkeyAction::ImmediateLock => {
                if let Ok(mut state_mgr) = state_manager.lock() {
                    let current_state = state_mgr.get_state();
                    match current_state {
                        AppState::FocusRunning | AppState::FocusPaused => {
                            let _events = state_mgr.end_session()?;
                            if let Ok(window_mgr) = window_manager.lock() {
                                window_mgr.show_break_overlay()?;
                            }
                        }
                        AppState::Idle => {
                            let _events = state_mgr.start_break(BreakType::Short)?;
                            if let Ok(window_mgr) = window_manager.lock() {
                                window_mgr.show_break_overlay()?;
                            }
                        }
                        _ => {
                            println!("Cannot start immediate lock in state: {:?}", current_state);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Load custom hotkey configurations from settings
    pub fn load_custom_hotkeys(&self) -> Result<(), Box<dyn std::error::Error>> {
        // In a real implementation, this would load from a settings file or database
        // For now, we'll use default configurations
        println!("Loading custom hotkey configurations (using defaults for now)");
        Ok(())
    }

    /// Save current hotkey configurations to settings
    pub fn save_hotkey_configurations(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(hotkeys) = self.registered_hotkeys.lock() {
            // In a real implementation, this would save to a settings file or database
            println!("Saving {} hotkey configurations", hotkeys.len());
            for (action, config) in hotkeys.iter() {
                println!(
                    "  {:?}: enabled={}, shortcut={:?}",
                    action, config.enabled, config.shortcut
                );
            }
        }
        Ok(())
    }

    /// Reset all hotkeys to default configurations
    pub fn reset_to_defaults(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Unregister all current hotkeys
        self.unregister_all()?;

        // Clear current configurations
        if let Ok(mut hotkeys) = self.registered_hotkeys.lock() {
            hotkeys.clear();
        }

        // Re-initialize with defaults
        self.initialize()?;

        println!("Reset all hotkeys to default configurations");
        Ok(())
    }

    /// Check for hotkey conflicts with system or other applications
    pub fn check_for_conflicts(&self) -> Vec<HotkeyAction> {
        let mut conflicts = Vec::new();

        if let Ok(hotkeys) = self.registered_hotkeys.lock() {
            for (action, config) in hotkeys.iter() {
                // In a real implementation, this would check against system hotkeys
                // For now, we'll just check if the hotkey is enabled
                if !config.enabled {
                    conflicts.push(action.clone());
                }
            }
        }

        conflicts
    }

    /// Get available modifier combinations for hotkey customization
    pub fn get_available_modifiers() -> Vec<(String, Modifiers)> {
        vec![
            #[cfg(target_os = "macos")]
            ("Cmd".to_string(), Modifiers::META),
            #[cfg(not(target_os = "macos"))]
            ("Ctrl".to_string(), Modifiers::CONTROL),
            ("Alt".to_string(), Modifiers::ALT),
            ("Shift".to_string(), Modifiers::SHIFT),
            #[cfg(target_os = "macos")]
            ("Cmd+Shift".to_string(), Modifiers::META | Modifiers::SHIFT),
            #[cfg(not(target_os = "macos"))]
            (
                "Ctrl+Shift".to_string(),
                Modifiers::CONTROL | Modifiers::SHIFT,
            ),
            #[cfg(target_os = "macos")]
            ("Cmd+Alt".to_string(), Modifiers::META | Modifiers::ALT),
            #[cfg(not(target_os = "macos"))]
            ("Ctrl+Alt".to_string(), Modifiers::CONTROL | Modifiers::ALT),
        ]
    }

    /// Unregister all hotkeys (cleanup)
    pub fn unregister_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(hotkeys) = self.registered_hotkeys.lock() {
            for config in hotkeys.values() {
                if let Err(e) = self
                    .app_handle
                    .global_shortcut()
                    .unregister(config.shortcut.clone())
                {
                    eprintln!("Warning: Failed to unregister hotkey: {}", e);
                }
            }
        }
        Ok(())
    }
}

// Tauri commands for hotkey management

#[tauri::command]
pub async fn get_hotkey_configs(
    hotkey_manager: tauri::State<'_, Arc<Mutex<HotkeyManager>>>,
) -> Result<HashMap<HotkeyAction, HotkeyConfig>, String> {
    let manager = hotkey_manager
        .lock()
        .map_err(|e| format!("Failed to lock hotkey manager: {}", e))?;
    Ok(manager.get_hotkey_configs())
}

#[tauri::command]
pub async fn update_hotkey_config(
    config: HotkeyConfig,
    hotkey_manager: tauri::State<'_, Arc<Mutex<HotkeyManager>>>,
) -> Result<(), String> {
    let manager = hotkey_manager
        .lock()
        .map_err(|e| format!("Failed to lock hotkey manager: {}", e))?;
    manager
        .update_hotkey(config)
        .map_err(|e| format!("Failed to update hotkey: {}", e))
}

#[tauri::command]
pub async fn set_hotkey_enabled(
    action: HotkeyAction,
    enabled: bool,
    hotkey_manager: tauri::State<'_, Arc<Mutex<HotkeyManager>>>,
) -> Result<(), String> {
    let manager = hotkey_manager
        .lock()
        .map_err(|e| format!("Failed to lock hotkey manager: {}", e))?;
    manager
        .set_hotkey_enabled(&action, enabled)
        .map_err(|e| format!("Failed to set hotkey enabled state: {}", e))
}

#[tauri::command]
pub async fn refresh_hotkey_state(
    hotkey_manager: tauri::State<'_, Arc<Mutex<HotkeyManager>>>,
) -> Result<(), String> {
    let manager = hotkey_manager
        .lock()
        .map_err(|e| format!("Failed to lock hotkey manager: {}", e))?;
    manager
        .update_hotkey_state_based_on_app_state()
        .map_err(|e| format!("Failed to refresh hotkey state: {}", e))
}

#[tauri::command]
pub async fn route_hotkey_event(
    action: HotkeyAction,
    hotkey_manager: tauri::State<'_, Arc<Mutex<HotkeyManager>>>,
) -> Result<Vec<HotkeyEventResult>, String> {
    let manager = hotkey_manager
        .lock()
        .map_err(|e| format!("Failed to lock hotkey manager: {}", e))?;
    manager
        .route_hotkey_event(action)
        .map_err(|e| format!("Failed to route hotkey event: {}", e))
}

#[tauri::command]
pub async fn save_hotkey_configurations(
    hotkey_manager: tauri::State<'_, Arc<Mutex<HotkeyManager>>>,
) -> Result<(), String> {
    let manager = hotkey_manager
        .lock()
        .map_err(|e| format!("Failed to lock hotkey manager: {}", e))?;
    manager
        .save_hotkey_configurations()
        .map_err(|e| format!("Failed to save hotkey configurations: {}", e))
}

#[tauri::command]
pub async fn load_custom_hotkeys(
    hotkey_manager: tauri::State<'_, Arc<Mutex<HotkeyManager>>>,
) -> Result<(), String> {
    let manager = hotkey_manager
        .lock()
        .map_err(|e| format!("Failed to lock hotkey manager: {}", e))?;
    manager
        .load_custom_hotkeys()
        .map_err(|e| format!("Failed to load custom hotkeys: {}", e))
}

#[tauri::command]
pub async fn reset_hotkeys_to_defaults(
    hotkey_manager: tauri::State<'_, Arc<Mutex<HotkeyManager>>>,
) -> Result<(), String> {
    let manager = hotkey_manager
        .lock()
        .map_err(|e| format!("Failed to lock hotkey manager: {}", e))?;
    manager
        .reset_to_defaults()
        .map_err(|e| format!("Failed to reset hotkeys to defaults: {}", e))
}

#[tauri::command]
pub async fn check_hotkey_conflicts(
    hotkey_manager: tauri::State<'_, Arc<Mutex<HotkeyManager>>>,
) -> Result<Vec<HotkeyAction>, String> {
    let manager = hotkey_manager
        .lock()
        .map_err(|e| format!("Failed to lock hotkey manager: {}", e))?;
    Ok(manager.check_for_conflicts())
}

#[tauri::command]
pub async fn get_available_modifiers() -> Result<Vec<(String, String)>, String> {
    // Convert Modifiers to string representation for frontend
    let modifiers = HotkeyManager::get_available_modifiers();
    let string_modifiers: Vec<(String, String)> = modifiers
        .into_iter()
        .map(|(name, _)| (name.clone(), name)) // In a real implementation, convert Modifiers to string
        .collect();
    Ok(string_modifiers)
}

