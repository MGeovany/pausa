use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{
    AppHandle, LogicalPosition, Manager, Position, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};
// use tauri_plugin_positioner::{Position as PositionerPosition, WindowExt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WindowType {
    CommandPalette,
    FocusWidget,
    BreakOverlay,
    Settings,
}

impl WindowType {
    pub fn label(&self) -> &'static str {
        match self {
            WindowType::CommandPalette => "command-palette",
            WindowType::FocusWidget => "focus-widget",
            WindowType::BreakOverlay => "break-overlay",
            WindowType::Settings => "settings",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowPosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub position: Option<WindowPosition>,
    pub is_visible: bool,
    pub monitor_index: Option<usize>,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            position: None,
            is_visible: false,
            monitor_index: None,
        }
    }
}

pub struct WindowManager {
    app_handle: AppHandle,
    window_states: Arc<Mutex<HashMap<WindowType, WindowState>>>,
}

impl WindowManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            window_states: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Show the command palette window
    pub fn show_command_palette(&self) -> Result<(), Box<dyn std::error::Error>> {
        let window = self.get_or_create_window(WindowType::CommandPalette)?;

        // Position in center of current monitor
        window.move_window(PositionerPosition::Center)?;
        window.show()?;
        window.set_focus()?;

        self.update_window_state(WindowType::CommandPalette, |state| {
            state.is_visible = true;
        });

        Ok(())
    }

    /// Hide the command palette window
    pub fn hide_command_palette(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(window) = self
            .app_handle
            .get_webview_window(WindowType::CommandPalette.label())
        {
            window.hide()?;
            self.update_window_state(WindowType::CommandPalette, |state| {
                state.is_visible = false;
            });
        }
        Ok(())
    }

    /// Toggle the command palette visibility
    pub fn toggle_command_palette(&self) -> Result<(), Box<dyn std::error::Error>> {
        let is_visible = self.is_window_visible(WindowType::CommandPalette);

        if is_visible {
            self.hide_command_palette()?;
        } else {
            self.show_command_palette()?;
        }

        Ok(())
    }

    /// Show the focus widget window
    pub fn show_focus_widget(&self) -> Result<(), Box<dyn std::error::Error>> {
        let window = self.get_or_create_window(WindowType::FocusWidget)?;

        // Position the widget based on saved position or default to top-right
        if let Some(saved_position) = self.get_saved_position(WindowType::FocusWidget) {
            window.set_position(Position::Logical(LogicalPosition {
                x: saved_position.x,
                y: saved_position.y,
            }))?;
        } else {
            // Default to top-right corner with some margin
            window.move_window(PositionerPosition::TopRight)?;
            if let Ok(position) = window.outer_position() {
                let adjusted_position = LogicalPosition {
                    x: position.x as f64 - 20.0, // 20px margin from right edge
                    y: position.y as f64 + 60.0, // 60px margin from top edge
                };
                window.set_position(Position::Logical(adjusted_position))?;
            }
        }

        window.show()?;

        self.update_window_state(WindowType::FocusWidget, |state| {
            state.is_visible = true;
        });

        Ok(())
    }

    /// Hide the focus widget window
    pub fn hide_focus_widget(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(window) = self
            .app_handle
            .get_webview_window(WindowType::FocusWidget.label())
        {
            // Save current position before hiding
            if let Ok(position) = window.outer_position() {
                self.save_window_position(
                    WindowType::FocusWidget,
                    WindowPosition {
                        x: position.x as f64,
                        y: position.y as f64,
                    },
                );
            }

            window.hide()?;
            self.update_window_state(WindowType::FocusWidget, |state| {
                state.is_visible = false;
            });
        }
        Ok(())
    }

    /// Show the break overlay on all monitors
    pub fn show_break_overlay(&self) -> Result<(), Box<dyn std::error::Error>> {
        let window = self.get_or_create_window(WindowType::BreakOverlay)?;

        // Set fullscreen on current monitor
        window.set_fullscreen(true)?;
        window.show()?;
        window.set_focus()?;

        self.update_window_state(WindowType::BreakOverlay, |state| {
            state.is_visible = true;
        });

        Ok(())
    }

    /// Hide the break overlay
    pub fn hide_break_overlay(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(window) = self
            .app_handle
            .get_webview_window(WindowType::BreakOverlay.label())
        {
            window.set_fullscreen(false)?;
            window.hide()?;
            self.update_window_state(WindowType::BreakOverlay, |state| {
                state.is_visible = false;
            });
        }
        Ok(())
    }

    /// Show the settings window
    pub fn show_settings(&self) -> Result<(), Box<dyn std::error::Error>> {
        let window = self.get_or_create_window(WindowType::Settings)?;

        // Center the settings window
        window.move_window(PositionerPosition::Center)?;
        window.show()?;
        window.set_focus()?;

        self.update_window_state(WindowType::Settings, |state| {
            state.is_visible = true;
        });

        Ok(())
    }

    /// Hide the settings window
    pub fn hide_settings(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(window) = self
            .app_handle
            .get_webview_window(WindowType::Settings.label())
        {
            window.hide()?;
            self.update_window_state(WindowType::Settings, |state| {
                state.is_visible = false;
            });
        }
        Ok(())
    }

    /// Hide a specific window type
    pub fn hide_window(&self, window_type: WindowType) -> Result<(), Box<dyn std::error::Error>> {
        match window_type {
            WindowType::CommandPalette => self.hide_command_palette(),
            WindowType::FocusWidget => self.hide_focus_widget(),
            WindowType::BreakOverlay => self.hide_break_overlay(),
            WindowType::Settings => self.hide_settings(),
        }
    }

    /// Check if a window is currently visible
    pub fn is_window_visible(&self, window_type: WindowType) -> bool {
        if let Ok(states) = self.window_states.lock() {
            states
                .get(&window_type)
                .map(|state| state.is_visible)
                .unwrap_or(false)
        } else {
            false
        }
    }

    /// Get or create a window of the specified type
    fn get_or_create_window(
        &self,
        window_type: WindowType,
    ) -> Result<WebviewWindow, Box<dyn std::error::Error>> {
        let label = window_type.label();

        // Try to get existing window
        if let Some(window) = self.app_handle.get_webview_window(label) {
            return Ok(window);
        }

        // Create new window with appropriate configuration
        let window = match window_type {
            WindowType::CommandPalette => WebviewWindowBuilder::new(
                &self.app_handle,
                label,
                WebviewUrl::App("index.html".into()),
            )
            .title("Pausa Command Palette")
            .inner_size(600.0, 400.0)
            .resizable(false)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .center()
            .shadow(false)
            .focused(true)
            .visible(false)
            .build()?,
            WindowType::FocusWidget => WebviewWindowBuilder::new(
                &self.app_handle,
                label,
                WebviewUrl::App("index.html".into()),
            )
            .title("Pausa Focus Widget")
            .inner_size(280.0, 80.0)
            .resizable(false)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .shadow(false)
            .visible(false)
            .build()?,
            WindowType::BreakOverlay => WebviewWindowBuilder::new(
                &self.app_handle,
                label,
                WebviewUrl::App("index.html".into()),
            )
            .title("Pausa Break")
            .resizable(false)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .fullscreen(true)
            .shadow(false)
            .focused(true)
            .visible(false)
            .build()?,
            WindowType::Settings => WebviewWindowBuilder::new(
                &self.app_handle,
                label,
                WebviewUrl::App("index.html".into()),
            )
            .title("Pausa Settings")
            .inner_size(800.0, 600.0)
            .resizable(true)
            .decorations(true)
            .center()
            .skip_taskbar(false)
            .shadow(true)
            .visible(false)
            .build()?,
        };
        Ok(window)
    }

    /// Update window state
    fn update_window_state<F>(&self, window_type: WindowType, updater: F)
    where
        F: FnOnce(&mut WindowState),
    {
        if let Ok(mut states) = self.window_states.lock() {
            let state = states.entry(window_type).or_default();
            updater(state);
        }
    }

    /// Save window position
    fn save_window_position(&self, window_type: WindowType, position: WindowPosition) {
        self.update_window_state(window_type, |state| {
            state.position = Some(position);
        });
    }

    /// Get saved window position
    fn get_saved_position(&self, window_type: WindowType) -> Option<WindowPosition> {
        if let Ok(states) = self.window_states.lock() {
            states
                .get(&window_type)
                .and_then(|state| state.position.clone())
        } else {
            None
        }
    }

    /// Get all window states (for persistence)
    pub fn get_all_window_states(&self) -> HashMap<WindowType, WindowState> {
        if let Ok(states) = self.window_states.lock() {
            states.clone()
        } else {
            HashMap::new()
        }
    }

    /// Restore window states (from persistence)
    pub fn restore_window_states(&self, states: HashMap<WindowType, WindowState>) {
        if let Ok(mut current_states) = self.window_states.lock() {
            *current_states = states;
        }
    }

    /// Handle window drag events for the focus widget
    pub fn handle_focus_widget_drag(
        &self,
        x: f64,
        y: f64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(window) = self
            .app_handle
            .get_webview_window(WindowType::FocusWidget.label())
        {
            window.set_position(Position::Logical(LogicalPosition { x, y }))?;
            self.save_window_position(WindowType::FocusWidget, WindowPosition { x, y });
        }
        Ok(())
    }

    /// Close all windows
    pub fn close_all_windows(&self) -> Result<(), Box<dyn std::error::Error>> {
        for window_type in [
            WindowType::CommandPalette,
            WindowType::FocusWidget,
            WindowType::BreakOverlay,
            WindowType::Settings,
        ] {
            self.hide_window(window_type)?;
        }
        Ok(())
    }
}

// Tauri commands for window management

#[tauri::command]
pub async fn show_command_palette(
    window_manager: tauri::State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<(), String> {
    let manager = window_manager
        .lock()
        .map_err(|e| format!("Failed to lock window manager: {}", e))?;
    manager
        .show_command_palette()
        .map_err(|e| format!("Failed to show command palette: {}", e))
}

#[tauri::command]
pub async fn hide_command_palette(
    window_manager: tauri::State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<(), String> {
    let manager = window_manager
        .lock()
        .map_err(|e| format!("Failed to lock window manager: {}", e))?;
    manager
        .hide_command_palette()
        .map_err(|e| format!("Failed to hide command palette: {}", e))
}

#[tauri::command]
pub async fn toggle_command_palette(
    window_manager: tauri::State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<(), String> {
    let manager = window_manager
        .lock()
        .map_err(|e| format!("Failed to lock window manager: {}", e))?;
    manager
        .toggle_command_palette()
        .map_err(|e| format!("Failed to toggle command palette: {}", e))
}

#[tauri::command]
pub async fn show_focus_widget(
    window_manager: tauri::State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<(), String> {
    let manager = window_manager
        .lock()
        .map_err(|e| format!("Failed to lock window manager: {}", e))?;
    manager
        .show_focus_widget()
        .map_err(|e| format!("Failed to show focus widget: {}", e))
}

#[tauri::command]
pub async fn hide_focus_widget(
    window_manager: tauri::State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<(), String> {
    let manager = window_manager
        .lock()
        .map_err(|e| format!("Failed to lock window manager: {}", e))?;
    manager
        .hide_focus_widget()
        .map_err(|e| format!("Failed to hide focus widget: {}", e))
}

#[tauri::command]
pub async fn show_break_overlay(
    window_manager: tauri::State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<(), String> {
    let manager = window_manager
        .lock()
        .map_err(|e| format!("Failed to lock window manager: {}", e))?;
    manager
        .show_break_overlay()
        .map_err(|e| format!("Failed to show break overlay: {}", e))
}

#[tauri::command]
pub async fn hide_break_overlay(
    window_manager: tauri::State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<(), String> {
    let manager = window_manager
        .lock()
        .map_err(|e| format!("Failed to lock window manager: {}", e))?;
    manager
        .hide_break_overlay()
        .map_err(|e| format!("Failed to hide break overlay: {}", e))
}

#[tauri::command]
pub async fn show_settings(
    window_manager: tauri::State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<(), String> {
    let manager = window_manager
        .lock()
        .map_err(|e| format!("Failed to lock window manager: {}", e))?;
    manager
        .show_settings()
        .map_err(|e| format!("Failed to show settings: {}", e))
}

#[tauri::command]
pub async fn hide_settings(
    window_manager: tauri::State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<(), String> {
    let manager = window_manager
        .lock()
        .map_err(|e| format!("Failed to lock window manager: {}", e))?;
    manager
        .hide_settings()
        .map_err(|e| format!("Failed to hide settings: {}", e))
}

#[tauri::command]
pub async fn handle_focus_widget_drag(
    x: f64,
    y: f64,
    window_manager: tauri::State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<(), String> {
    let manager = window_manager
        .lock()
        .map_err(|e| format!("Failed to lock window manager: {}", e))?;
    manager
        .handle_focus_widget_drag(x, y)
        .map_err(|e| format!("Failed to handle drag: {}", e))
}

#[tauri::command]
pub async fn is_window_visible(
    window_type: String,
    window_manager: tauri::State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<bool, String> {
    let manager = window_manager
        .lock()
        .map_err(|e| format!("Failed to lock window manager: {}", e))?;

    let window_type = match window_type.as_str() {
        "command-palette" => WindowType::CommandPalette,
        "focus-widget" => WindowType::FocusWidget,
        "break-overlay" => WindowType::BreakOverlay,
        "settings" => WindowType::Settings,
        _ => return Err("Invalid window type".to_string()),
    };

    Ok(manager.is_window_visible(window_type))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_type_labels() {
        assert_eq!(WindowType::CommandPalette.label(), "command-palette");
        assert_eq!(WindowType::FocusWidget.label(), "focus-widget");
        assert_eq!(WindowType::BreakOverlay.label(), "break-overlay");
        assert_eq!(WindowType::Settings.label(), "settings");
    }

    #[test]
    fn test_window_state_default() {
        let state = WindowState::default();
        assert!(state.position.is_none());
        assert!(!state.is_visible);
        assert!(state.monitor_index.is_none());
    }

    #[test]
    fn test_window_position() {
        let position = WindowPosition { x: 100.0, y: 200.0 };
        assert_eq!(position.x, 100.0);
        assert_eq!(position.y, 200.0);
    }
}
