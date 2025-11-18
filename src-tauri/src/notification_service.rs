use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

/// Simple notification service that uses native OS notifications
pub struct NotificationService {
    user_name: Option<String>,
}

impl NotificationService {
    pub fn new() -> Self {
        Self { user_name: None }
    }

    pub fn set_user_name(&mut self, name: Option<String>) {
        self.user_name = name;
    }

    pub fn get_user_name(&self) -> Option<&str> {
        self.user_name.as_deref()
    }

    /// Send a focus start notification
    pub fn notify_focus_start(&self, app: &AppHandle) {
        let title = "Focus mode started";
        let body = if let Some(name) = &self.user_name {
            format!("{}, time to concentrate and do great work 🧠", name)
        } else {
            "Time to concentrate and do great work 🧠".to_string()
        };

        let _ = app.notification().builder().title(title).body(&body).show();
    }

    /// Send a focus warning notification (2 minutes before end)
    pub fn notify_focus_warning(&self, app: &AppHandle, minutes_left: u32) {
        let title = "Focus session ending soon";
        let body = format!("{} minutes left. Time to wrap up ⏳", minutes_left);

        let _ = app.notification().builder().title(title).body(&body).show();
    }

    /// Send a focus end notification
    pub fn notify_focus_end(&self, app: &AppHandle) {
        let title = "Great work!";
        let body = if let Some(name) = &self.user_name {
            format!("{}, time to take a break ✨", name)
        } else {
            "Time to take a break ✨".to_string()
        };

        let _ = app.notification().builder().title(title).body(&body).show();
    }

    /// Send a break start notification
    pub fn notify_break_start(&self, app: &AppHandle) {
        let title = "Active break";
        let body = "Move. Stretch. Drink water ☕";

        let _ = app.notification().builder().title(title).body(body).show();
    }

    /// Send a long break start notification
    pub fn notify_long_break_start(&self, app: &AppHandle) {
        let title = "Excellent progress!";
        let body = if let Some(name) = &self.user_name {
            format!("{}, take a long break. You've earned it 🌟", name)
        } else {
            "Take a long break. You've earned it 🌟".to_string()
        };

        let _ = app.notification().builder().title(title).body(&body).show();
    }

    /// Send a break end notification
    pub fn notify_break_end(&self, app: &AppHandle) {
        let title = "Ready";
        let body = "Shall we start another block? 💪";

        let _ = app.notification().builder().title(title).body(body).show();
    }

    /// Send a cycle complete notification
    pub fn notify_cycle_complete(&self, app: &AppHandle, cycle_count: u32) {
        let title = "Cycle completed!";
        let body = format!("You've completed {} cycles. Keep it up! 🎉", cycle_count);

        let _ = app.notification().builder().title(title).body(&body).show();
    }
}

impl Default for NotificationService {
    fn default() -> Self {
        Self::new()
    }
}
