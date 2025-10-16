use async_trait::async_trait;
use crate::errors::AppError;
use crate::domain::tokens::Tokens;

pub struct OAuthCallback { pub code: String, pub state: String }

#[async_trait]
pub trait OAuthProvider {
    async fn start_login(&mut self) -> Result<String, AppError>; 
    async fn handle_callback(&mut self, cb: OAuthCallback) -> Result<Tokens, AppError>;
}
