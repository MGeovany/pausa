use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// This module provides onboarding functionality for the Pausa application

/// Represents the different steps in the onboarding process
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OnboardingStep {
    Welcome,
    WorkSchedule,
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
            OnboardingStep::Complete => {
                self.current_step = OnboardingStep::WorkSchedule;
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

        assert_eq!(manager.next_step().unwrap(), OnboardingStep::Complete);
        assert_eq!(manager.get_current_step(), &OnboardingStep::Complete);
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
}
