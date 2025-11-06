use crate::onboarding::{OnboardingManager, OnboardingStep};
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub async fn start_onboarding(
    state: State<'_, Mutex<OnboardingManager>>,
) -> Result<OnboardingStep, String> {
    println!("🚀 [Rust] start_onboarding called");

    let manager: std::sync::MutexGuard<'_, OnboardingManager> = state.lock().map_err(|e| {
        println!("❌ [Rust] Error acquiring onboarding manager lock: {}", e);
        format!("Failed to acquire onboarding manager lock: {}", e)
    })?;

    let current_step = manager.get_current_step().clone();
    println!("✅ [Rust] Current onboarding step: {:?}", current_step);

    Ok(current_step)
}

#[tauri::command]
pub async fn next_onboarding_step(
    step_data: Option<serde_json::Value>,
    state: State<'_, Mutex<OnboardingManager>>,
) -> Result<OnboardingStep, String> {
    println!(
        "🔄 [Rust] next_onboarding_step called with data: {:?}",
        step_data
    );

    let mut manager = state.lock().map_err(|e| {
        let error_msg = format!("Failed to acquire onboarding manager lock: {}", e);
        println!("❌ [Rust] {}", error_msg);
        error_msg
    })?;

    let current_step = manager.get_current_step().clone();
    println!(
        "📍 [Rust] Current step before navigation: {:?}",
        current_step
    );

    // Validate step data if provided
    if let Some(data) = step_data {
        if data.is_null() {
            println!("⚠️ [Rust] Received null step data, skipping storage");
        } else {
            println!(
                "💾 [Rust] Storing step data for {:?}: {:?}",
                current_step, data
            );
            manager.set_step_data(current_step, data).map_err(|e| {
                let error_msg = format!("Failed to store step data: {}", e);
                println!("❌ [Rust] {}", error_msg);
                error_msg
            })?;
        }
    }

    // Attempt navigation
    let next_step = manager.next_step().map_err(|e| {
        let error_msg = format!("Navigation failed: {}", e);
        println!("❌ [Rust] {}", error_msg);
        error_msg
    })?;

    println!(
        "✅ [Rust] Successfully advanced to next step: {:?}",
        next_step
    );
    Ok(next_step)
}

#[tauri::command]
pub async fn previous_onboarding_step(
    state: State<'_, Mutex<OnboardingManager>>,
) -> Result<OnboardingStep, String> {
    println!("🔙 [Rust] previous_onboarding_step called");

    let mut manager = state.lock().map_err(|e| {
        let error_msg = format!("Failed to acquire onboarding manager lock: {}", e);
        println!("❌ [Rust] {}", error_msg);
        error_msg
    })?;

    let current_step = manager.get_current_step().clone();
    println!(
        "📍 [Rust] Current step before backward navigation: {:?}",
        current_step
    );

    // Attempt backward navigation
    let previous_step = manager.previous_step().map_err(|e| {
        let error_msg = format!("Backward navigation failed: {}", e);
        println!("❌ [Rust] {}", error_msg);
        error_msg
    })?;

    println!(
        "✅ [Rust] Successfully moved to previous step: {:?}",
        previous_step
    );
    Ok(previous_step)
}

#[tauri::command]
pub async fn complete_onboarding(
    final_config: serde_json::Value,
    onboarding_state: State<'_, Mutex<OnboardingManager>>,
    app_state: State<'_, crate::state::AppState>,
) -> Result<(), String> {
    println!(
        "🎉 [Rust] complete_onboarding called with config: {:?}",
        final_config
    );

    let mut manager = onboarding_state.lock().map_err(|e| {
        println!("❌ [Rust] Error acquiring onboarding manager lock: {}", e);
        format!("Failed to acquire onboarding manager lock: {}", e)
    })?;

    // Store the final configuration
    manager.set_step_data(OnboardingStep::Complete, final_config.clone())?;

    // Mark as complete
    if !manager.is_complete() {
        manager.next_step()?; // This should set is_complete to true
    }

    // Save onboarding completion to database
    let config_json = serde_json::to_string(&final_config).map_err(|e| {
        let error_msg = format!("Failed to serialize config: {}", e);
        println!("❌ [Rust] {}", error_msg);
        error_msg
    })?;

    app_state
        .database
        .save_onboarding_completion("1.0", Some(&config_json))
        .map_err(|e| {
            let error_msg = format!("Failed to save onboarding completion: {}", e);
            println!("❌ [Rust] {}", error_msg);
            error_msg
        })?;

    println!("✅ [Rust] Onboarding completed and saved to database successfully");

    Ok(())
}

#[tauri::command]
pub async fn get_onboarding_status(
    state: State<'_, Mutex<OnboardingManager>>,
) -> Result<bool, String> {
    println!("📊 [Rust] get_onboarding_status called");

    let manager = state.lock().map_err(|e| {
        println!("❌ [Rust] Error acquiring onboarding manager lock: {}", e);
        format!("Failed to acquire onboarding manager lock: {}", e)
    })?;

    let is_complete = manager.is_complete();
    println!(
        "✅ [Rust] Onboarding status: {}",
        if is_complete {
            "complete"
        } else {
            "incomplete"
        }
    );

    Ok(is_complete)
}
