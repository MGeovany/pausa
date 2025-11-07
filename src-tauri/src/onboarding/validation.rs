use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Validation errors for onboarding configuration
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum ValidationError {
    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("Invalid value for {field}: {reason}")]
    InvalidValue { field: String, reason: String },

    #[error("Value out of range for {field}: expected {min}-{max}, got {value}")]
    OutOfRange {
        field: String,
        min: i32,
        max: i32,
        value: i32,
    },

    #[error("Invalid time format for {field}: expected HH:MM, got {value}")]
    InvalidTimeFormat { field: String, value: String },

    #[error("Work end time must be after start time")]
    InvalidTimeRange,

    #[error("Emergency key combination is too simple or common")]
    WeakEmergencyKey,

    #[error("Configuration conflict: {reason}")]
    ConfigurationConflict { reason: String },
}

/// Validation result type
pub type ValidationResult<T> = Result<T, Vec<ValidationError>>;

/// Comprehensive onboarding configuration validator
#[derive(Debug, Clone)]
pub struct OnboardingValidator {
    errors: Vec<ValidationError>,
}

impl OnboardingValidator {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Validate complete onboarding configuration
    pub fn validate_configuration(&mut self, config: &serde_json::Value) -> ValidationResult<()> {
        self.errors.clear();

        // Validate work schedule configuration
        if let Some(work_schedule) = config.get("workSchedule") {
            self.validate_work_schedule(work_schedule);
        }

        // Validate cycle configuration
        self.validate_cycle_configuration(config);

        // Validate strict mode configuration
        self.validate_strict_mode_configuration(config);

        // Validate user name if provided
        if let Some(user_name) = config.get("userName") {
            self.validate_user_name(user_name);
        }

        // Check for configuration conflicts
        self.validate_configuration_consistency(config);

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    /// Validate work schedule configuration
    fn validate_work_schedule(&mut self, work_schedule: &serde_json::Value) {
        let use_work_schedule = work_schedule
            .get("useWorkSchedule")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if use_work_schedule {
            // Validate work start time
            if let Some(start_time) = work_schedule.get("workStartTime") {
                if let Some(start_str) = start_time.as_str() {
                    if !self.is_valid_time_format(start_str) {
                        self.errors.push(ValidationError::InvalidTimeFormat {
                            field: "workStartTime".to_string(),
                            value: start_str.to_string(),
                        });
                    }
                } else {
                    self.errors.push(ValidationError::MissingField {
                        field: "workStartTime".to_string(),
                    });
                }
            } else {
                self.errors.push(ValidationError::MissingField {
                    field: "workStartTime".to_string(),
                });
            }

            // Validate work end time
            if let Some(end_time) = work_schedule.get("workEndTime") {
                if let Some(end_str) = end_time.as_str() {
                    if !self.is_valid_time_format(end_str) {
                        self.errors.push(ValidationError::InvalidTimeFormat {
                            field: "workEndTime".to_string(),
                            value: end_str.to_string(),
                        });
                    }
                } else {
                    self.errors.push(ValidationError::MissingField {
                        field: "workEndTime".to_string(),
                    });
                }
            } else {
                self.errors.push(ValidationError::MissingField {
                    field: "workEndTime".to_string(),
                });
            }

            // Validate time range consistency
            if let (Some(start_str), Some(end_str)) = (
                work_schedule.get("workStartTime").and_then(|v| v.as_str()),
                work_schedule.get("workEndTime").and_then(|v| v.as_str()),
            ) {
                if self.is_valid_time_format(start_str) && self.is_valid_time_format(end_str) {
                    if !self.is_valid_time_range(start_str, end_str) {
                        self.errors.push(ValidationError::InvalidTimeRange);
                    }
                }
            }
        }
    }

    /// Validate cycle configuration
    fn validate_cycle_configuration(&mut self, config: &serde_json::Value) {
        // Validate focus duration
        if let Some(focus_duration) = config.get("focusDuration") {
            if let Some(duration) = focus_duration.as_u64() {
                let duration = duration as i32;
                if duration < 5 || duration > 120 {
                    self.errors.push(ValidationError::OutOfRange {
                        field: "focusDuration".to_string(),
                        min: 5,
                        max: 120,
                        value: duration,
                    });
                }
            } else {
                self.errors.push(ValidationError::InvalidValue {
                    field: "focusDuration".to_string(),
                    reason: "must be a positive number".to_string(),
                });
            }
        } else {
            self.errors.push(ValidationError::MissingField {
                field: "focusDuration".to_string(),
            });
        }

        // Validate break duration
        if let Some(break_duration) = config.get("breakDuration") {
            if let Some(duration) = break_duration.as_u64() {
                let duration = duration as i32;
                if duration < 1 || duration > 30 {
                    self.errors.push(ValidationError::OutOfRange {
                        field: "breakDuration".to_string(),
                        min: 1,
                        max: 30,
                        value: duration,
                    });
                }
            } else {
                self.errors.push(ValidationError::InvalidValue {
                    field: "breakDuration".to_string(),
                    reason: "must be a positive number".to_string(),
                });
            }
        } else {
            self.errors.push(ValidationError::MissingField {
                field: "breakDuration".to_string(),
            });
        }

        // Validate long break duration
        if let Some(long_break_duration) = config.get("longBreakDuration") {
            if let Some(duration) = long_break_duration.as_u64() {
                let duration = duration as i32;
                if duration < 5 || duration > 60 {
                    self.errors.push(ValidationError::OutOfRange {
                        field: "longBreakDuration".to_string(),
                        min: 5,
                        max: 60,
                        value: duration,
                    });
                }
            } else {
                self.errors.push(ValidationError::InvalidValue {
                    field: "longBreakDuration".to_string(),
                    reason: "must be a positive number".to_string(),
                });
            }
        } else {
            self.errors.push(ValidationError::MissingField {
                field: "longBreakDuration".to_string(),
            });
        }

        // Validate cycles per long break
        if let Some(cycles) = config.get("cyclesPerLongBreak") {
            if let Some(cycles_num) = cycles.as_u64() {
                let cycles_num = cycles_num as i32;
                if cycles_num < 2 || cycles_num > 10 {
                    self.errors.push(ValidationError::OutOfRange {
                        field: "cyclesPerLongBreak".to_string(),
                        min: 2,
                        max: 10,
                        value: cycles_num,
                    });
                }
            } else {
                self.errors.push(ValidationError::InvalidValue {
                    field: "cyclesPerLongBreak".to_string(),
                    reason: "must be a positive number".to_string(),
                });
            }
        } else {
            self.errors.push(ValidationError::MissingField {
                field: "cyclesPerLongBreak".to_string(),
            });
        }
    }

    /// Validate strict mode configuration
    fn validate_strict_mode_configuration(&mut self, config: &serde_json::Value) {
        let strict_mode = config
            .get("strictMode")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if strict_mode {
            // Emergency key is required for strict mode
            if let Some(emergency_key) = config.get("emergencyKey") {
                if let Some(key_str) = emergency_key.as_str() {
                    if !self.is_valid_emergency_key(key_str) {
                        self.errors.push(ValidationError::WeakEmergencyKey);
                    }
                } else if emergency_key.is_null() {
                    self.errors.push(ValidationError::MissingField {
                        field: "emergencyKey".to_string(),
                    });
                }
            } else {
                self.errors.push(ValidationError::MissingField {
                    field: "emergencyKey".to_string(),
                });
            }
        }
    }

    /// Validate user name
    fn validate_user_name(&mut self, user_name: &serde_json::Value) {
        if let Some(name_str) = user_name.as_str() {
            let name_str = name_str.trim();
            if name_str.is_empty() {
                self.errors.push(ValidationError::InvalidValue {
                    field: "userName".to_string(),
                    reason: "cannot be empty".to_string(),
                });
            } else if name_str.len() > 50 {
                self.errors.push(ValidationError::InvalidValue {
                    field: "userName".to_string(),
                    reason: "cannot exceed 50 characters".to_string(),
                });
            }
        }
    }

    /// Validate configuration consistency
    fn validate_configuration_consistency(&mut self, config: &serde_json::Value) {
        // Check if break duration is reasonable compared to focus duration
        if let (Some(focus), Some(break_dur)) = (
            config.get("focusDuration").and_then(|v| v.as_u64()),
            config.get("breakDuration").and_then(|v| v.as_u64()),
        ) {
            let ratio = focus as f64 / break_dur as f64;
            if ratio < 2.0 {
                self.errors.push(ValidationError::ConfigurationConflict {
                    reason: "Break duration should be significantly shorter than focus duration for optimal productivity".to_string(),
                });
            }
        }

        // Check if long break is longer than regular break
        if let (Some(break_dur), Some(long_break)) = (
            config.get("breakDuration").and_then(|v| v.as_u64()),
            config.get("longBreakDuration").and_then(|v| v.as_u64()),
        ) {
            if long_break <= break_dur {
                self.errors.push(ValidationError::ConfigurationConflict {
                    reason: "Long break duration should be longer than regular break duration"
                        .to_string(),
                });
            }
        }
    }

    /// Check if time format is valid (HH:MM)
    pub fn is_valid_time_format(&self, time_str: &str) -> bool {
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() != 2 {
            return false;
        }

        if let (Ok(hours), Ok(minutes)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
            hours < 24 && minutes < 60
        } else {
            false
        }
    }

    /// Check if time range is valid (end > start)
    pub fn is_valid_time_range(&self, start_time: &str, end_time: &str) -> bool {
        let start_minutes = self.time_to_minutes(start_time);
        let end_minutes = self.time_to_minutes(end_time);

        match (start_minutes, end_minutes) {
            (Some(start), Some(end)) => end > start,
            _ => false,
        }
    }

    /// Convert time string to minutes since midnight
    fn time_to_minutes(&self, time_str: &str) -> Option<u32> {
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() != 2 {
            return None;
        }

        if let (Ok(hours), Ok(minutes)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
            if hours < 24 && minutes < 60 {
                Some(hours * 60 + minutes)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Check if emergency key combination is secure enough
    pub fn is_valid_emergency_key(&self, key_combination: &str) -> bool {
        let key_combination = key_combination.trim();

        // Must not be empty
        if key_combination.is_empty() {
            return false;
        }

        // Must contain at least one modifier key
        let has_modifier = key_combination.contains("Cmd")
            || key_combination.contains("Ctrl")
            || key_combination.contains("Alt")
            || key_combination.contains("Shift")
            || key_combination.contains("⌘")
            || key_combination.contains("⌃")
            || key_combination.contains("⌥")
            || key_combination.contains("⇧");

        if !has_modifier {
            return false;
        }

        // Check against common/weak combinations
        let weak_combinations = [
            "Cmd+Q", "⌘+Q", "Cmd+W", "⌘+W", "Cmd+C", "⌘+C", "Cmd+V", "⌘+V", "Cmd+X", "⌘+X",
            "Cmd+Z", "⌘+Z", "Cmd+A", "⌘+A", "Cmd+S", "⌘+S", "Alt+F4", "Ctrl+C", "Ctrl+V", "Ctrl+X",
            "Ctrl+Z", "Ctrl+A", "Ctrl+S",
        ];

        !weak_combinations
            .iter()
            .any(|&weak| key_combination.eq_ignore_ascii_case(weak))
    }

    /// Get all validation errors
    pub fn get_errors(&self) -> &[ValidationError] {
        &self.errors
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl Default for OnboardingValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate step-specific data
pub fn validate_step_data(step: &str, data: &serde_json::Value) -> ValidationResult<()> {
    let mut validator = OnboardingValidator::new();

    match step {
        "WorkSchedule" => {
            if let Some(use_schedule) = data.get("useWorkSchedule") {
                if !use_schedule.is_boolean() {
                    validator.errors.push(ValidationError::InvalidValue {
                        field: "useWorkSchedule".to_string(),
                        reason: "must be true or false".to_string(),
                    });
                }
            } else {
                validator.errors.push(ValidationError::MissingField {
                    field: "useWorkSchedule".to_string(),
                });
            }
        }
        "WorkHours" => {
            let work_schedule = serde_json::json!({
                "useWorkSchedule": true,
                "workStartTime": data.get("startTime"),
                "workEndTime": data.get("endTime")
            });
            validator.validate_work_schedule(&work_schedule);
        }
        "CycleConfig" => {
            validator.validate_cycle_configuration(data);
        }
        "StrictMode" => {
            validator.validate_strict_mode_configuration(data);
            if let Some(user_name) = data.get("userName") {
                validator.validate_user_name(user_name);
            }
        }
        _ => {
            // No specific validation for other steps
        }
    }

    if validator.has_errors() {
        Err(validator.errors)
    } else {
        Ok(())
    }
}

