use tauri::{AppHandle, WebviewWindow};

/// Manages system-level input locking during strict mode breaks
pub struct SystemLockManager {
    /// Whether the system is currently locked
    is_locked: bool,
    /// Reference to the app handle for window management
    app_handle: AppHandle,
    /// Emergency hotkey combination (e.g., "Cmd+Shift+E")
    emergency_hotkey: Option<String>,
}

impl SystemLockManager {
    /// Create a new SystemLockManager
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            is_locked: false,
            app_handle,
            emergency_hotkey: None,
        }
    }

    /// Lock the system by blocking keyboard and mouse inputs
    /// This is called when a break starts in strict mode
    pub fn lock_system(&mut self, window: &WebviewWindow) -> Result<(), String> {
        if self.is_locked {
            eprintln!("⚠️ [SystemLockManager] System is already locked");
            return Err("System is already locked".to_string());
        }

        println!("🔒 [SystemLockManager] Locking system inputs");

        // Set window properties to prevent interactions
        if let Err(e) = window.set_always_on_top(true) {
            eprintln!("❌ [SystemLockManager] Failed to set always on top: {}", e);
            return Err(format!("Failed to set always on top: {}", e));
        }

        if let Err(e) = window.set_fullscreen(true) {
            eprintln!("❌ [SystemLockManager] Failed to set fullscreen: {}", e);
            return Err(format!("Failed to set fullscreen: {}", e));
        }

        if let Err(e) = window.set_focus() {
            eprintln!("❌ [SystemLockManager] Failed to set focus: {}", e);
            // Don't fail on focus error, just log it
            println!("⚠️ [SystemLockManager] Continuing despite focus error");
        }

        // Note: Actual keyboard/mouse blocking will be implemented via JavaScript
        // event listeners in the frontend (subtasks 5.2 and 5.3)

        self.is_locked = true;
        println!("✅ [SystemLockManager] System locked successfully");

        Ok(())
    }

    /// Unlock the system by restoring normal input functionality
    /// This is called when a break ends or emergency exit is triggered
    pub fn unlock_system(&mut self, window: Option<&WebviewWindow>) -> Result<(), String> {
        if !self.is_locked {
            println!("ℹ️ [SystemLockManager] System is not locked, nothing to unlock");
            return Err("System is not locked".to_string());
        }

        println!("🔓 [SystemLockManager] Unlocking system inputs");

        // Restore window properties if window is provided
        if let Some(win) = window {
            if let Err(e) = win.set_fullscreen(false) {
                eprintln!("⚠️ [SystemLockManager] Failed to exit fullscreen: {}", e);
                // Continue anyway - we still want to mark as unlocked
                println!("⚠️ [SystemLockManager] Continuing despite fullscreen exit error");
            }
        }

        // Note: Frontend will handle removing event listeners

        self.is_locked = false;
        println!("✅ [SystemLockManager] System unlocked successfully");

        Ok(())
    }

    /// Check if the system is currently locked
    pub fn is_locked(&self) -> bool {
        self.is_locked
    }

    /// Register an emergency hotkey combination
    pub fn register_emergency_hotkey(&mut self, combination: String) -> Result<(), String> {
        // Validate the combination format
        if combination.is_empty() {
            return Err("Emergency hotkey combination cannot be empty".to_string());
        }

        // Basic validation: should contain at least one modifier
        let has_modifier = combination.contains("Cmd")
            || combination.contains("Ctrl")
            || combination.contains("Alt")
            || combination.contains("Shift");

        if !has_modifier {
            return Err(
                "Emergency hotkey must include at least one modifier key (Cmd, Ctrl, Alt, Shift)"
                    .to_string(),
            );
        }

        // Prevent common system shortcuts
        let forbidden_combinations = vec![
            "Cmd+Q",
            "Cmd+W",
            "Cmd+M",
            "Cmd+H",
            "Cmd+Tab",
            "Alt+F4",
            "Ctrl+Alt+Delete",
        ];

        if forbidden_combinations
            .iter()
            .any(|&forbidden| combination == forbidden)
        {
            return Err(format!(
                "Cannot use system shortcut '{}' as emergency key",
                combination
            ));
        }

        println!(
            "🔑 [SystemLockManager] Registering emergency hotkey: {}",
            combination
        );

        self.emergency_hotkey = Some(combination);

        Ok(())
    }

    /// Unregister the emergency hotkey
    pub fn unregister_emergency_hotkey(&mut self) -> Result<(), String> {
        if self.emergency_hotkey.is_none() {
            return Err("No emergency hotkey is registered".to_string());
        }

        println!("🔑 [SystemLockManager] Unregistering emergency hotkey");

        self.emergency_hotkey = None;

        println!("✅ [SystemLockManager] Emergency hotkey unregistered successfully");

        Ok(())
    }

    /// Get the currently registered emergency hotkey
    pub fn get_emergency_hotkey(&self) -> Option<String> {
        self.emergency_hotkey.clone()
    }

    /// Force unlock the system (used in emergency situations)
    /// This bypasses normal checks and ensures the system is unlocked
    pub fn force_unlock(&mut self) -> Result<(), String> {
        println!("🚨 [SystemLockManager] Force unlocking system");

        if !self.is_locked {
            println!(
                "ℹ️ [SystemLockManager] System was not locked, but marking as unlocked anyway"
            );
        }

        self.is_locked = false;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // Note: These tests are limited because SystemLockManager requires an AppHandle
    // which is only available in a running Tauri application context

    #[test]
    fn test_emergency_hotkey_validation() {
        // Test empty combination
        let result = validate_emergency_hotkey("");
        assert!(result.is_err());

        // Test without modifier
        let result = validate_emergency_hotkey("A");
        assert!(result.is_err());

        // Test with modifier
        let result = validate_emergency_hotkey("Cmd+Shift+E");
        assert!(result.is_ok());

        // Test forbidden combination
        let result = validate_emergency_hotkey("Cmd+Q");
        assert!(result.is_err());
    }

    // Helper function for testing validation logic
    fn validate_emergency_hotkey(combination: &str) -> Result<(), String> {
        if combination.is_empty() {
            return Err("Emergency hotkey combination cannot be empty".to_string());
        }

        let has_modifier = combination.contains("Cmd")
            || combination.contains("Ctrl")
            || combination.contains("Alt")
            || combination.contains("Shift");

        if !has_modifier {
            return Err("Emergency hotkey must include at least one modifier key".to_string());
        }

        let forbidden_combinations = vec![
            "Cmd+Q",
            "Cmd+W",
            "Cmd+M",
            "Cmd+H",
            "Cmd+Tab",
            "Alt+F4",
            "Ctrl+Alt+Delete",
        ];

        if forbidden_combinations
            .iter()
            .any(|&forbidden| combination == forbidden)
        {
            return Err(format!(
                "Cannot use system shortcut '{}' as emergency key",
                combination
            ));
        }

        Ok(())
    }
}
