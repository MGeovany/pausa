use tauri::State;

use crate::api_models::SessionStats;
use crate::state::AppState;

/// Fetch focus session statistics for the given horizon (in days).
#[tauri::command]
pub async fn get_session_stats(
    days: u32,
    state: State<'_, AppState>,
) -> Result<Vec<SessionStats>, String> {
    println!("📊 [StatsHandler] get_session_stats called for {} days", days);
    
    let stats = state
        .database
        .get_session_stats(days)
        .map_err(|error| {
            eprintln!("❌ [StatsHandler] Failed to get session stats: {}", error);
            format!("Failed to get session stats: {}", error)
        })?;

    println!("📊 [StatsHandler] Retrieved {} stat entries", stats.len());
    for stat in &stats {
        println!("📊 [StatsHandler] Stat: date={}, focus_minutes={}, breaks={}, sessions={}", 
            stat.date, stat.focus_minutes, stat.breaks_completed, stat.sessions_completed);
    }

    Ok(stats.into_iter().map(SessionStats::from).collect())
}

