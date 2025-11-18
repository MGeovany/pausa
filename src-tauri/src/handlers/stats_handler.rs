use tauri::State;

use crate::api_models::SessionStats;
use crate::state::AppState;

/// Fetch focus session statistics for the given horizon (in days).
#[tauri::command]
pub async fn get_session_stats(
    days: u32,
    state: State<'_, AppState>,
) -> Result<Vec<SessionStats>, String> {
    let stats = state
        .database
        .get_session_stats(days)
        .map_err(|error| format!("Failed to get session stats: {}", error))?;

    for stat in &stats {}

    Ok(stats.into_iter().map(SessionStats::from).collect())
}
