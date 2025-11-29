use tauri::{AppHandle, Emitter, Manager};
use url::Url;

use crate::{state::AppState, domain::oauth::{OAuthCallback, OAuthProvider}};

pub fn handle_deep_link(app: &AppHandle, raw: String) -> Result<(), String> {
    let url = Url::parse(&raw).map_err(|e| e.to_string())?;
    if url.path() != "/auth/callback" { return Ok(()); }

    let mut code = None;
    let mut state = None;
    for (k, v) in url.query_pairs() {
        if k == "code" { code = Some(v.to_string()); }
        if k == "state" { state = Some(v.to_string()); }
    }

    let (code, state) = match (code, state) {
        (Some(c), Some(s)) => (c, s),
        _ => return Err("callback sin code/state".into()),
    };

    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let shared: tauri::State<AppState> = app_handle.state::<AppState>();
        
        // Clone the callback data before locking
        let callback = OAuthCallback { code, state };
        
        // Get the result without holding the lock across await
        let result = {
            let mut svc = shared.oauth_google.lock().await;
            svc.handle_callback(callback).await
        };

        match result {
            Ok(tokens) => {
                if let Err(e) = shared.tokens_storage.save(&tokens) {
                    let _ = app_handle.emit("auth:error", format!("save tokens: {}", e));
                    return;
                }
                let _ = app_handle.emit("auth:success", ());
            }
            Err(e) => { let _ = app_handle.emit("auth:error", e.to_string()); }
        }
    });

    Ok(())
}
