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
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self { AppError::Http(e.to_string()) }
}
impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self { AppError::Json(e.to_string()) }
}
impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self { AppError::Io(e.to_string()) }
}
