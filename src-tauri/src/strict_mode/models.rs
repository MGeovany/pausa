use serde::{Deserialize, Serialize};

/// Configuration for strict mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrictModeConfig {
    /// Whether strict mode is enabled
    pub enabled: bool,
    /// Emergency key combination to exit strict mode (e.g., "Cmd+Shift+E")
    pub emergency_key_combination: Option<String>,
    /// Countdown duration before break starts (in seconds)
    pub transition_countdown_seconds: u32,
}

impl Default for StrictModeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            emergency_key_combination: None,
            transition_countdown_seconds: 10,
        }
    }
}

/// Current state of strict mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrictModeState {
    /// Whether strict mode is currently active
    pub is_active: bool,
    /// Whether the system is currently locked (during break)
    pub is_locked: bool,
    /// Current window type being displayed
    pub current_window_type: Option<StrictModeWindowType>,
}

impl Default for StrictModeState {
    fn default() -> Self {
        Self {
            is_active: false,
            is_locked: false,
            current_window_type: None,
        }
    }
}

/// Types of windows used in strict mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrictModeWindowType {
    /// Menu bar icon (minimized state)
    MenuBarIcon,
    /// Popover shown when clicking menu bar icon
    MenuBarPopover,
    /// Transition window before break starts
    BreakTransition,
    /// Fullscreen overlay during break (locked)
    FullscreenBreakOverlay,
}

impl std::fmt::Display for StrictModeWindowType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StrictModeWindowType::MenuBarIcon => write!(f, "menu_bar_icon"),
            StrictModeWindowType::MenuBarPopover => write!(f, "menu_bar_popover"),
            StrictModeWindowType::BreakTransition => write!(f, "break_transition"),
            StrictModeWindowType::FullscreenBreakOverlay => {
                write!(f, "fullscreen_break_overlay")
            }
        }
    }
}
