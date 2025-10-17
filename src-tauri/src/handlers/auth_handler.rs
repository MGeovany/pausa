use tauri::{AppHandle, State, Emitter, Manager};
use crate::{state::AppState, domain::oauth::OAuthProvider, domain::oauth::OAuthCallback};

#[tauri::command]
pub async fn login_with_google(app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    println!("🔐 [Rust] login_with_google called");
    
    println!("🔒 [Rust] Acquiring OAuth service lock...");
    let mut svc = state.oauth_google.lock().await;
    
    println!("🚀 [Rust] Starting login...");
    let url = svc.start_login().await.map_err(|e| {
        println!("❌ [Rust] Error starting login: {}", e);
        format!("Failed to start login: {}", e)
    })?;
    
    println!("✅ [Rust] Login URL generated: {}", url);
    drop(svc); // Release lock before opening browser
    
    println!("🌐 [Rust] Opening browser...");
    // Use the system command to open the URL
    #[cfg(target_os = "macos")]
    std::process::Command::new("open")
        .arg(&url)
        .spawn()
        .map_err(|e| format!("Failed to open browser: {}", e))?;
    
    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd")
        .args(&["/C", "start", &url])
        .spawn()
        .map_err(|e| format!("Failed to open browser: {}", e))?;
    
    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open")
        .arg(&url)
        .spawn()
        .map_err(|e| format!("Failed to open browser: {}", e))?;
    
    println!("✅ [Rust] Browser opened successfully");
    
    // Start a simple HTTP server to receive the callback
    let app_handle = app.clone();
    
    tokio::spawn(async move {
        use warp::Filter;
        
        let app_handle_inner = app_handle.clone();
        
        let callback = warp::get()
            .and(warp::query::<std::collections::HashMap<String, String>>())
            .and_then(move |params: std::collections::HashMap<String, String>| {
                let app_handle = app_handle_inner.clone();
                
                async move {
                    println!("🔗 [Callback] Received callback with params: {:?}", params);
                    
                    if let (Some(code), Some(state_param)) = (params.get("code"), params.get("state")) {
                        println!("✅ [Callback] Got code and state: {}", code);
                        
                        // Get the state from the app handle
                        let state: tauri::State<AppState> = app_handle.state();
                        
                        // Process the callback
                        let mut svc = state.oauth_google.lock().await;
                        let callback = OAuthCallback { 
                            code: code.clone(), 
                            state: state_param.clone() 
                        };
                        
                        match svc.handle_callback(callback).await {
                            Ok(tokens) => {
                                println!("✅ [Callback] Tokens obtained successfully");
                                
                                // Save tokens
                                if let Err(e) = state.tokens_storage.save(&tokens) {
                                    println!("❌ [Callback] Error saving tokens: {}", e);
                                    let _ = app_handle.emit("auth:error", format!("Failed to save tokens: {}", e));
                                } else {
                                    println!("✅ [Callback] Tokens saved successfully");
                                    println!("📤 [Callback] Emitting auth:success event...");
                                    
                                    // Try to emit to the main window
                                    let window_label = "onboarding";
                                    match app_handle.emit_to(window_label, "auth:success", ()) {
                                        Ok(_) => println!("✅ [Callback] Event emitted successfully to window: {}", window_label),
                                        Err(e) => {
                                            println!("❌ [Callback] Error emitting to window {}: {}", window_label, e);
                                            // Fallback: try to emit to all windows
                                            let _ = app_handle.emit("auth:success", ());
                                            println!("⚠️ [Callback] Fallback: emitted to all windows");
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                println!("❌ [Callback] Error handling callback: {}", e);
                                let _ = app_handle.emit("auth:error", e.to_string());
                            }
                        }
                    }
                    
                    Ok::<_, warp::Rejection>(warp::reply::html(r#"
                        <!DOCTYPE html>
                        <html>
                        <head>
                            <title>Authentication Successful</title>
                            <style>
                                * {
                                    margin: 0;
                                    padding: 0;
                                    box-sizing: border-box;
                                }
                                body {
                                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                                    display: flex;
                                    justify-content: center;
                                    align-items: center;
                                    height: 100vh;
                                    background: #ffffff;
                                }
                                .container {
                                    text-align: center;
                                    padding: 48px;
                                    max-width: 400px;
                                }
                                .icon {
                                    width: 64px;
                                    height: 64px;
                                    background: #000000;
                                    border-radius: 12px;
                                    display: flex;
                                    align-items: center;
                                    justify-content: center;
                                    margin: 0 auto 24px;
                                    font-size: 32px;
                                }
                                h1 {
                                    color: #000000;
                                    font-size: 24px;
                                    font-weight: 600;
                                    margin-bottom: 8px;
                                }
                                p {
                                    color: #666666;
                                    font-size: 14px;
                                }
                            </style>
                        </head>
                        <body>
                            <div class="container">
                                <div class="icon">✓</div>
                                <h1>Authentication Successful</h1>
                                <p>You can close this window now.</p>
                            </div>
                        </body>
                        </html>
                    "#))
                }
            });
        
        let routes = callback;
        
        println!("🌐 [Callback] Starting server on port 8080...");
        warp::serve(routes)
            .run(([127, 0, 0, 1], 8080))
            .await;
    });
    
    Ok("browser_opened".into())
}

#[tauri::command]
pub async fn read_tokens(state: State<'_, AppState>) -> Result<Option<crate::domain::tokens::Tokens>, String> {
    state.tokens_storage.load()
}

#[tauri::command]
pub async fn get_tokens_path(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.tokens_storage.path_str())
}
