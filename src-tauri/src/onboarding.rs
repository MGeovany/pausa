use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod backup;
pub mod validation;

pub use backup::{
    create_post_onboarding_backup, create_pre_onboarding_backup, BackupManager, BackupType,
    ConfigurationBackup,
};
pub use validation::{validate_step_data, OnboardingValidator, ValidationError, ValidationResult};

// This module provides onboarding functionality for the Pausa application

/// Represents the different steps in the onboarding process
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OnboardingStep {
    Welcome,
    WorkSchedule,
    WorkHours,
    CycleConfig,
    StrictMode,
    Summary,
    Complete,
}

impl Default for OnboardingStep {
    fn default() -> Self {
        OnboardingStep::Welcome
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingManager {
    current_step: OnboardingStep,
    collected_data: HashMap<String, serde_json::Value>,
    is_complete: bool,
}

impl OnboardingManager {
    pub fn new() -> Self {
        Self {
            current_step: OnboardingStep::Welcome,
            collected_data: HashMap::new(),
            is_complete: false,
        }
    }

    pub fn get_current_step(&self) -> &OnboardingStep {
        &self.current_step
    }

    pub fn is_complete(&self) -> bool {
        self.is_complete
    }

    pub fn next_step(&mut self) -> Result<OnboardingStep, String> {
        if self.is_complete {
            return Err("Onboarding already completed".to_string());
        }

        match self.current_step {
            OnboardingStep::Welcome => {
                self.current_step = OnboardingStep::WorkSchedule;
                Ok(self.current_step.clone())
            }
            OnboardingStep::WorkSchedule => {
                // Check if user wants to use work schedule
                if let Some(data) = self.get_step_data(&OnboardingStep::WorkSchedule) {
                    if let Some(use_schedule) = data.get("useWorkSchedule") {
                        if use_schedule.as_bool().unwrap_or(false) {
                            self.current_step = OnboardingStep::WorkHours;
                            return Ok(self.current_step.clone());
                        }
                    }
                }
                // Go to CycleConfig if not using work schedule
                self.current_step = OnboardingStep::CycleConfig;
                Ok(self.current_step.clone())
            }
            OnboardingStep::WorkHours => {
                self.current_step = OnboardingStep::CycleConfig;
                Ok(self.current_step.clone())
            }
            OnboardingStep::CycleConfig => {
                self.current_step = OnboardingStep::StrictMode;
                Ok(self.current_step.clone())
            }
            OnboardingStep::StrictMode => {
                self.current_step = OnboardingStep::Summary;
                Ok(self.current_step.clone())
            }
            OnboardingStep::Summary => {
                self.current_step = OnboardingStep::Complete;
                self.is_complete = true;
                Ok(self.current_step.clone())
            }
            OnboardingStep::Complete => Err("Cannot proceed beyond completion step".to_string()),
        }
    }

    pub fn previous_step(&mut self) -> Result<OnboardingStep, String> {
        match self.current_step {
            OnboardingStep::Welcome => Err("Cannot go back from welcome step".to_string()),
            OnboardingStep::WorkSchedule => {
                self.current_step = OnboardingStep::Welcome;
                Ok(self.current_step.clone())
            }
            OnboardingStep::WorkHours => {
                self.current_step = OnboardingStep::WorkSchedule;
                Ok(self.current_step.clone())
            }
            OnboardingStep::CycleConfig => {
                // Check if we came from WorkHours or WorkSchedule
                if let Some(data) = self.get_step_data(&OnboardingStep::WorkSchedule) {
                    if let Some(use_schedule) = data.get("useWorkSchedule") {
                        if use_schedule.as_bool().unwrap_or(false) {
                            self.current_step = OnboardingStep::WorkHours;
                            return Ok(self.current_step.clone());
                        }
                    }
                }
                self.current_step = OnboardingStep::WorkSchedule;
                Ok(self.current_step.clone())
            }
            OnboardingStep::StrictMode => {
                self.current_step = OnboardingStep::CycleConfig;
                Ok(self.current_step.clone())
            }
            OnboardingStep::Summary => {
                self.current_step = OnboardingStep::StrictMode;
                Ok(self.current_step.clone())
            }
            OnboardingStep::Complete => {
                self.current_step = OnboardingStep::Summary;
                self.is_complete = false;
                Ok(self.current_step.clone())
            }
        }
    }

    pub fn set_step_data(
        &mut self,
        step: OnboardingStep,
        data: serde_json::Value,
    ) -> Result<(), String> {
        let step_key = format!("{:?}", step);
        self.collected_data.insert(step_key, data);
        Ok(())
    }

    pub fn get_step_data(&self, step: &OnboardingStep) -> Option<&serde_json::Value> {
        let step_key = format!("{:?}", step);
        self.collected_data.get(&step_key)
    }

    pub fn reset(&mut self) {
        self.current_step = OnboardingStep::Welcome;
        self.collected_data.clear();
        self.is_complete = false;
    }
}

impl Default for OnboardingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Public function to verify the onboarding module is properly linked
pub fn verify_onboarding_module() -> &'static str {
    "Onboarding module is properly linked"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_onboarding_manager_creation() {
        let manager = OnboardingManager::new();
        assert_eq!(manager.get_current_step(), &OnboardingStep::Welcome);
        assert!(!manager.is_complete());
    }

    #[test]
    fn test_onboarding_navigation() {
        let mut manager = OnboardingManager::new();

        // Test forward navigation
        assert_eq!(manager.next_step().unwrap(), OnboardingStep::WorkSchedule);
        assert_eq!(manager.get_current_step(), &OnboardingStep::WorkSchedule);

        // Set work schedule data to skip work hours
        let work_schedule_data = serde_json::json!({"useWorkSchedule": false});
        manager
            .set_step_data(OnboardingStep::WorkSchedule, work_schedule_data)
            .unwrap();

        // Should skip to CycleConfig
        assert_eq!(manager.next_step().unwrap(), OnboardingStep::CycleConfig);
        assert_eq!(manager.next_step().unwrap(), OnboardingStep::StrictMode);
        assert_eq!(manager.next_step().unwrap(), OnboardingStep::Summary);
        assert_eq!(manager.next_step().unwrap(), OnboardingStep::Complete);
        assert!(manager.is_complete());

        // Test cannot proceed beyond complete
        assert!(manager.next_step().is_err());
    }

    #[test]
    fn test_onboarding_backward_navigation() {
        let mut manager = OnboardingManager::new();

        // Move to WorkSchedule
        manager.next_step().unwrap();

        // Test backward navigation
        assert_eq!(manager.previous_step().unwrap(), OnboardingStep::Welcome);
        assert_eq!(manager.get_current_step(), &OnboardingStep::Welcome);

        // Test cannot go back from welcome
        assert!(manager.previous_step().is_err());
    }

    #[test]
    fn test_step_data_storage() {
        let mut manager = OnboardingManager::new();
        let test_data = serde_json::json!({"test": "value"});

        manager
            .set_step_data(OnboardingStep::Welcome, test_data.clone())
            .unwrap();

        let retrieved_data = manager.get_step_data(&OnboardingStep::Welcome);
        assert!(retrieved_data.is_some());
        assert_eq!(retrieved_data.unwrap(), &test_data);
    }

    #[test]
    fn test_reset_functionality() {
        let mut manager = OnboardingManager::new();

        // Progress through onboarding
        manager.next_step().unwrap();
        manager
            .set_step_data(
                OnboardingStep::WorkSchedule,
                serde_json::json!({"data": "test"}),
            )
            .unwrap();

        // Reset
        manager.reset();

        assert_eq!(manager.get_current_step(), &OnboardingStep::Welcome);
        assert!(!manager.is_complete());
        assert!(manager
            .get_step_data(&OnboardingStep::WorkSchedule)
            .is_none());
    }

    // Comprehensive validation and backup tests

    #[test]
    fn test_complete_onboarding_flow_validation() {
        use serde_json::json;

        // Test valid configuration
        let valid_config = json!({
            "workSchedule": {
                "useWorkSchedule": true,
                "workStartTime": "09:00",
                "workEndTime": "17:00"
            },
            "focusDuration": 25,
            "breakDuration": 5,
            "longBreakDuration": 15,
            "cyclesPerLongBreak": 4,
            "strictMode": true,
            "emergencyKey": "Cmd+Shift+P",
            "userName": "Test User"
        });

        let mut validator = OnboardingValidator::new();
        assert!(validator.validate_configuration(&valid_config).is_ok());

        // Test invalid configuration
        let invalid_config = json!({
            "focusDuration": 200, // Too high
            "breakDuration": 50,  // Too high
            "longBreakDuration": 5, // Less than break duration
            "cyclesPerLongBreak": 1, // Too low
            "strictMode": true,
            "emergencyKey": "Cmd+C" // Weak key
        });

        let mut validator = OnboardingValidator::new();
        let result = validator.validate_configuration(&invalid_config);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.len() > 0);
    }

    #[test]
    fn test_step_validation() {
        use serde_json::json;

        // Test WorkSchedule step
        let work_schedule_data = json!({
            "useWorkSchedule": true
        });
        assert!(validate_step_data("WorkSchedule", &work_schedule_data).is_ok());

        let invalid_work_schedule = json!({
            "useWorkSchedule": "invalid"
        });
        assert!(validate_step_data("WorkSchedule", &invalid_work_schedule).is_err());

        // Test WorkHours step
        let work_hours_data = json!({
            "startTime": "09:00",
            "endTime": "17:00"
        });
        assert!(validate_step_data("WorkHours", &work_hours_data).is_ok());

        let invalid_work_hours = json!({
            "startTime": "25:00", // Invalid time
            "endTime": "17:00"
        });
        assert!(validate_step_data("WorkHours", &invalid_work_hours).is_err());

        // Test CycleConfig step
        let cycle_config_data = json!({
            "focusDuration": 25,
            "breakDuration": 5,
            "longBreakDuration": 15,
            "cyclesPerLongBreak": 4
        });
        assert!(validate_step_data("CycleConfig", &cycle_config_data).is_ok());

        // Test StrictMode step
        let strict_mode_data = json!({
            "strictMode": true,
            "emergencyKey": "Cmd+Shift+P",
            "userName": "Test User"
        });
        assert!(validate_step_data("StrictMode", &strict_mode_data).is_ok());
    }

    #[test]
    fn test_emergency_key_validation() {
        let validator = OnboardingValidator::new();

        // Test valid emergency keys
        assert!(validator.is_valid_emergency_key("Cmd+Shift+P"));
        assert!(validator.is_valid_emergency_key("Ctrl+Alt+F12"));
        assert!(validator.is_valid_emergency_key("⌘+⇧+P"));

        // Test invalid emergency keys
        assert!(!validator.is_valid_emergency_key(""));
        assert!(!validator.is_valid_emergency_key("P")); // No modifier
        assert!(!validator.is_valid_emergency_key("Cmd+C")); // Common shortcut
        assert!(!validator.is_valid_emergency_key("Ctrl+V")); // Common shortcut
    }

    #[test]
    fn test_time_validation() {
        let validator = OnboardingValidator::new();

        // Test valid time formats
        assert!(validator.is_valid_time_format("09:00"));
        assert!(validator.is_valid_time_format("23:59"));
        assert!(validator.is_valid_time_format("00:00"));
        assert!(validator.is_valid_time_format("9:00")); // Single digit hours are valid

        // Test invalid time formats
        assert!(!validator.is_valid_time_format("25:00")); // Invalid hour
        assert!(!validator.is_valid_time_format("12:60")); // Invalid minute
        assert!(!validator.is_valid_time_format("invalid"));
        assert!(!validator.is_valid_time_format("12")); // Missing minutes
        assert!(!validator.is_valid_time_format("12:5:30")); // Too many parts

        // Test time range validation
        assert!(validator.is_valid_time_range("09:00", "17:00"));
        assert!(!validator.is_valid_time_range("17:00", "09:00")); // End before start
        assert!(!validator.is_valid_time_range("12:00", "12:00")); // Same time
    }

    #[test]
    fn test_configuration_consistency_validation() {
        use serde_json::json;

        let mut validator = OnboardingValidator::new();

        // Test configuration where break is too long compared to focus
        let inconsistent_config = json!({
            "focusDuration": 5,
            "breakDuration": 10, // Longer than focus
            "longBreakDuration": 15,
            "cyclesPerLongBreak": 4
        });

        let result = validator.validate_configuration(&inconsistent_config);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::ConfigurationConflict { .. })));

        // Test configuration where long break is not longer than regular break
        let inconsistent_config2 = json!({
            "focusDuration": 25,
            "breakDuration": 15,
            "longBreakDuration": 10, // Shorter than regular break
            "cyclesPerLongBreak": 4
        });

        let mut validator2 = OnboardingValidator::new();
        let result2 = validator2.validate_configuration(&inconsistent_config2);
        assert!(result2.is_err());
        let errors2 = result2.unwrap_err();
        assert!(errors2
            .iter()
            .any(|e| matches!(e, ValidationError::ConfigurationConflict { .. })));
    }
}
