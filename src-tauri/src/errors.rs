#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("lock: {0}")]
    Lock(String),
    #[error("http: {0}")]
    Http(String),
    #[error("json: {0}")]
    Json(String),
    #[error("config: {0}")]
    Config(String),
    #[error("auth: {0}")]
    Auth(String),
    #[error("io: {0}")]
    Io(String),
    #[error("database: {0}")]
    Database(String),
    #[error("validation: {0}")]
    Validation(String),
    #[error("cycle: {0}")]
    Cycle(String),
    #[error("onboarding: {0}")]
    Onboarding(String),
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::Http(e.to_string())
    }
}
impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Json(e.to_string())
    }
}
impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}

// User-friendly error messages for frontend
impl AppError {
    pub fn user_message(&self) -> String {
        match self {
            AppError::Lock(_) => {
                "The application is temporarily busy. Please try again.".to_string()
            }
            AppError::Http(_) => {
                "Network connection error. Please check your internet connection.".to_string()
            }
            AppError::Json(_) => "Data format error. Please restart the application.".to_string(),
            AppError::Config(_) => "Configuration error. Please check your settings.".to_string(),
            AppError::Auth(_) => "Authentication failed. Please sign in again.".to_string(),
            AppError::Io(_) => "File system error. Please check permissions.".to_string(),
            AppError::Database(_) => "Database error. Your data may need recovery.".to_string(),
            AppError::Validation(_) => "Invalid input. Please check your entries.".to_string(),
            AppError::Cycle(_) => {
                "Work cycle error. Please try restarting the session.".to_string()
            }
            AppError::Onboarding(_) => {
                "Setup error. Please restart the onboarding process.".to_string()
            }
        }
    }

    pub fn should_retry(&self) -> bool {
        matches!(self, AppError::Lock(_) | AppError::Http(_))
    }

    pub fn is_recoverable(&self) -> bool {
        !matches!(self, AppError::Database(_) | AppError::Io(_))
    }
}
