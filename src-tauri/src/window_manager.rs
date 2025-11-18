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
    MenuBarPopover,
    BreakTransition,
}

impl WindowType {
    pub fn label(&self) -> &'static str {
        match self {
            WindowType::CommandPalette => "command-palette",
            WindowType::FocusWidget => "focus-widget",
            WindowType::BreakOverlay => "break-overlay",
            WindowType::Settings => "settings",
            WindowType::MenuBarPopover => "menu-bar-popover",
            WindowType::BreakTransition => "break-transition",
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
        self.center_window(&window)?;
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
            self.position_top_right(&window)?;
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
        self.center_window(&window)?;
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

    /// Minimize the main window to menu bar (hide it)
    pub fn minimize_to_menu_bar(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📍 [WindowManager] Minimizing main window to menu bar");

        if let Some(window) = self.app_handle.get_webview_window("main") {
            window.hide()?;
            println!("✅ [WindowManager] Main window minimized to menu bar");
        } else {
            println!("⚠️ [WindowManager] Main window not found");
        }
        Ok(())
    }

    /// Restore the main window from menu bar
    pub fn restore_from_menu_bar(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📍 [WindowManager] Restoring main window from menu bar");

        if let Some(window) = self.app_handle.get_webview_window("main") {
            window.show()?;
            window.set_focus()?;
            println!("✅ [WindowManager] Main window restored from menu bar");
        } else {
            println!("⚠️ [WindowManager] Main window not found");
        }
        Ok(())
    }

    /// Show the menu bar popover
    pub fn show_menu_bar_popover(&self) -> Result<(), Box<dyn std::error::Error>> {
        let window = self.get_or_create_window(WindowType::MenuBarPopover)?;

        // Position near the menu bar icon (top-right area)
        self.position_near_menu_bar(&window)?;
        window.show()?;
        window.set_focus()?;

        self.update_window_state(WindowType::MenuBarPopover, |state| {
            state.is_visible = true;
        });

        Ok(())
    }

    /// Hide the menu bar popover
    pub fn hide_menu_bar_popover(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(window) = self
            .app_handle
            .get_webview_window(WindowType::MenuBarPopover.label())
        {
            window.hide()?;
            self.update_window_state(WindowType::MenuBarPopover, |state| {
                state.is_visible = false;
            });
        }
        Ok(())
    }

    /// Show the break transition window
    pub fn show_break_transition(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🪟 [WindowManager] Showing break transition window");

        let window = self.get_or_create_window(WindowType::BreakTransition)?;
        println!("🪟 [WindowManager] Break transition window created/retrieved");

        // Center the window on the current monitor
        self.center_window(&window)?;
        println!("🪟 [WindowManager] Break transition window centered");

        window.show()?;
        println!("🪟 [WindowManager] Break transition window shown");

        window.set_focus()?;
        println!("🪟 [WindowManager] Break transition window focused");

        self.update_window_state(WindowType::BreakTransition, |state| {
            state.is_visible = true;
        });

        println!("✅ [WindowManager] Break transition window fully displayed");
        Ok(())
    }

    /// Hide the break transition window
    pub fn hide_break_transition(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🪟 [WindowManager] Hiding break transition window");

        if let Some(window) = self
            .app_handle
            .get_webview_window(WindowType::BreakTransition.label())
        {
            window.hide()?;
            self.update_window_state(WindowType::BreakTransition, |state| {
                state.is_visible = false;
            });
            println!("✅ [WindowManager] Break transition window hidden");
        } else {
            println!("ℹ️ [WindowManager] Break transition window not found, nothing to hide");
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
            WindowType::MenuBarPopover => self.hide_menu_bar_popover(),
            WindowType::BreakTransition => self.hide_break_transition(),
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
            WindowType::MenuBarPopover => WebviewWindowBuilder::new(
                &self.app_handle,
                label,
                WebviewUrl::App("index.html".into()),
            )
            .title("Pausa Menu Bar")
            .inner_size(280.0, 200.0)
            .resizable(false)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .shadow(true)
            .focused(true)
            .visible(false)
            .build()?,
            WindowType::BreakTransition => WebviewWindowBuilder::new(
                &self.app_handle,
                label,
                WebviewUrl::App("index.html".into()),
            )
            .title("Pausa Break Transition")
            .inner_size(400.0, 300.0)
            .resizable(false)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .center()
            .shadow(true)
            .focused(true)
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
            WindowType::MenuBarPopover,
            WindowType::BreakTransition,
        ] {
            self.hide_window(window_type)?;
        }
        Ok(())
    }

    /// Center a window on the current monitor
    fn center_window(&self, window: &WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(monitor) = window.current_monitor()? {
            let monitor_size = monitor.size();
            let window_size = window.outer_size()?;

            let x = (monitor_size.width as i32 - window_size.width as i32) / 2;
            let y = (monitor_size.height as i32 - window_size.height as i32) / 2;

            window.set_position(Position::Logical(LogicalPosition {
                x: x as f64,
                y: y as f64,
            }))?;
        }
        Ok(())
    }

    /// Position a window at the top-right corner with margin
    fn position_top_right(&self, window: &WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(monitor) = window.current_monitor()? {
            let monitor_size = monitor.size();
            let window_size = window.outer_size()?;

            let x = monitor_size.width as i32 - window_size.width as i32 - 20; // 20px margin from right
            let y = 60; // 60px margin from top

            window.set_position(Position::Logical(LogicalPosition {
                x: x as f64,
                y: y as f64,
            }))?;
        }
        Ok(())
    }

    /// Position a window near the menu bar icon (top-right area)
    fn position_near_menu_bar(
        &self,
        window: &WebviewWindow,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(monitor) = window.current_monitor()? {
            let monitor_size = monitor.size();
            let window_size = window.outer_size()?;

            // Position in top-right corner, below the menu bar
            let x = monitor_size.width as i32 - window_size.width as i32 - 10; // 10px margin from right
            let y = 30; // 30px from top (below menu bar)

            window.set_position(Position::Logical(LogicalPosition {
                x: x as f64,
                y: y as f64,
            }))?;
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
        "menu-bar-popover" => WindowType::MenuBarPopover,
        "break-transition" => WindowType::BreakTransition,
        _ => return Err("Invalid window type".to_string()),
    };

    Ok(manager.is_window_visible(window_type))
}

#[tauri::command]
pub async fn minimize_to_menu_bar(
    window_manager: tauri::State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<(), String> {
    let manager = window_manager
        .lock()
        .map_err(|e| format!("Failed to lock window manager: {}", e))?;
    manager
        .minimize_to_menu_bar()
        .map_err(|e| format!("Failed to minimize to menu bar: {}", e))
}

#[tauri::command]
pub async fn restore_from_menu_bar(
    window_manager: tauri::State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<(), String> {
    let manager = window_manager
        .lock()
        .map_err(|e| format!("Failed to lock window manager: {}", e))?;
    manager
        .restore_from_menu_bar()
        .map_err(|e| format!("Failed to restore from menu bar: {}", e))
}

#[tauri::command]
pub async fn show_break_transition(
    window_manager: tauri::State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<(), String> {
    let manager = window_manager
        .lock()
        .map_err(|e| format!("Failed to lock window manager: {}", e))?;
    manager
        .show_break_transition()
        .map_err(|e| format!("Failed to show break transition: {}", e))
}

#[tauri::command]
pub async fn hide_break_transition(
    window_manager: tauri::State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<(), String> {
    let manager = window_manager
        .lock()
        .map_err(|e| format!("Failed to lock window manager: {}", e))?;
    manager
        .hide_break_transition()
        .map_err(|e| format!("Failed to hide break transition: {}", e))
}
