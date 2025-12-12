use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;
use std::sync::Arc;

use crate::config::{tokens_path, AppConfig};
use crate::cycle_orchestrator::CycleOrchestrator;
use crate::database::DatabaseManager;
use crate::domain::tokens::TokenStorage;
use crate::notification_service::NotificationService;
use crate::services::{google_oauth::GoogleOAuthService, telemetry::TelemetryService};
use crate::strict_mode::StrictModeOrchestrator;

pub struct AppState {
    pub oauth_google: Mutex<GoogleOAuthService>,
    pub tokens_storage: TokenStorage,
    pub database: DatabaseManager,
    pub app_handle: AppHandle,
    pub cycle_orchestrator: Mutex<Option<CycleOrchestrator>>,
    pub notification_service: Mutex<NotificationService>,
    pub strict_mode_orchestrator: Mutex<Option<StrictModeOrchestrator>>,
    pub telemetry_service: Arc<TelemetryService>,
}

impl AppState {
    pub fn init(app: &AppHandle, cfg: AppConfig) -> Result<Self, String> {
        let tokens_path = tokens_path(app)?;
        let storage = TokenStorage::new(tokens_path);
        let svc = GoogleOAuthService::new(cfg);

        // Initialize database
        let app_data_dir = app
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data directory: {}", e))?;
        let db_path = app_data_dir.join("pausa.db");
        let database = DatabaseManager::new(db_path)
            .map_err(|e| format!("Failed to initialize database: {}", e))?;

        // Initialize notification service
        let notification_service = NotificationService::new();
        
        // Initialize telemetry service
        let telemetry_service = Arc::new(TelemetryService::new());

        Ok(Self {
            oauth_google: Mutex::new(svc),
            tokens_storage: storage,
            database,
            app_handle: app.clone(),
            cycle_orchestrator: Mutex::new(None),
            notification_service: Mutex::new(notification_service),
            strict_mode_orchestrator: Mutex::new(None),
            telemetry_service,
        })
    }
}
