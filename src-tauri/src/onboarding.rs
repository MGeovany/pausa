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
                self.current_step = OnboardingStep::WorkHours;
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
            OnboardingStep::WorkHours => {
                self.current_step = OnboardingStep::Welcome;
                Ok(self.current_step.clone())
            }
            OnboardingStep::CycleConfig => {
                self.current_step = OnboardingStep::WorkHours;
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
