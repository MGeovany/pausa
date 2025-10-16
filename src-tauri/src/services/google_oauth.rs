use url::Url;
use chrono::Utc;
use async_trait::async_trait;

use crate::{config::AppConfig, errors::AppError, infrastructure::http::http_client};
use crate::domain::{oauth::{OAuthProvider, OAuthCallback}, tokens::Tokens};
use super::pkce::{generate, PkcePair};

pub struct GoogleOAuthService {
    cfg: AppConfig,
    pkce: Option<PkcePair>,
    expected_state: Option<String>,
}

impl GoogleOAuthService {
    pub fn new(cfg: AppConfig) -> Self {
        Self { cfg, pkce: None, expected_state: None }
    }
}

#[async_trait]
impl OAuthProvider for GoogleOAuthService {
    async fn start_login(&mut self) -> Result<String, AppError> {
        let pkce = generate();
        let state = rand::random::<u64>().to_string();
        self.pkce = Some(pkce.clone());
        self.expected_state = Some(state.clone());

        let mut url = Url::parse("https://accounts.google.com/o/oauth2/v2/auth").unwrap();
        url.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", &self.cfg.client_id)
            .append_pair("redirect_uri", &self.cfg.redirect_uri)
            .append_pair("scope", "openid email profile")
            .append_pair("state", &state)
            .append_pair("code_challenge", &pkce.challenge)
            .append_pair("code_challenge_method", "S256")
            .append_pair("access_type", "offline")
            .append_pair("prompt", "consent");
        Ok(url.into_string())
    }

    async fn handle_callback(&mut self, cb: OAuthCallback) -> Result<Tokens, AppError> {
        // CSRF check
        if self.expected_state.as_deref() != Some(&cb.state) {
            return Err(AppError::Auth("state mismatch".into()));
        }
        let code_verifier = self.pkce.as_ref().ok_or(AppError::Auth("missing pkce".into()))?.verifier.clone();

        #[derive(serde::Serialize)]
        struct TokenReq<'a> {
            client_id: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            client_secret: Option<&'a str>,
            code: &'a str,
            code_verifier: &'a str,
            grant_type: &'a str,
            redirect_uri: &'a str,
        }
        #[derive(serde::Deserialize)]
        struct TokenRes {
            access_token: String,
            expires_in: i64,
            scope: String,
            token_type: String,
            #[serde(default)] refresh_token: Option<String>,
            #[serde(default)] id_token: Option<String>,
        }

        let body = TokenReq {
            client_id: &self.cfg.client_id,
            client_secret: self.cfg.client_secret.as_deref(),
            code: &cb.code,
            code_verifier: &code_verifier,
            grant_type: "authorization_code",
            redirect_uri: &self.cfg.redirect_uri,
        };

        let res = http_client()
            .post("https://oauth2.googleapis.com/token")
            .form(&body)
            .send().await?;
        if !res.status().is_success() {
            let text = res.text().await.unwrap_or_default();
            return Err(AppError::Auth(format!("token error ({}): {}", res.status(), text)));
        }
        let tr: TokenRes = res.json().await?;

        Ok(Tokens {
            access_token: tr.access_token,
            expires_in: tr.expires_in,
            scope: tr.scope,
            token_type: tr.token_type,
            refresh_token: tr.refresh_token,
            id_token: tr.id_token,
            obtained_at: Utc::now().timestamp(),
        })
    }
}
