use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::Utc;

#[derive(Debug, Clone, Serialize)]
struct PostHogEvent {
    #[serde(rename = "api_key")]
    api_key: String,
    event: String,
    #[serde(rename = "distinct_id")]
    distinct_id: String,
    properties: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    pub error_type: String,
    pub message: String,
    pub context: Option<String>,
    pub stack: Option<String>,
    pub user_action: Option<String>,
    pub recoverable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricEvent {
    pub metric_name: String,
    pub value: f64,
    pub tags: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginEvent {
    pub event: String, // "login_started", "login_success", "login_failed"
    pub provider: String,
    pub error: Option<String>,
}

pub struct TelemetryService {
    api_key: Option<String>,
    enabled: bool,
    queue: Arc<Mutex<Vec<PostHogEvent>>>,
    user_id: Arc<Mutex<Option<String>>>,
}

impl TelemetryService {
    pub fn new() -> Self {
        // Try PostHog API key from env, or use default
        let api_key = std::env::var("POSTHOG_API_KEY")
            .ok()
            .or_else(|| {
                // Default API key provided by user
                Some("phc_wfSFStKUOOz5DEdaLzZZJlWA0Rbd8hOz3TAp58qHstl".to_string())
            });
        
        let enabled = api_key.is_some();
        
        Self {
            api_key,
            enabled,
            queue: Arc::new(Mutex::new(Vec::new())),
            user_id: Arc::new(Mutex::new(None)),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub async fn set_user_id(&self, user_id: Option<String>) {
        let mut uid = self.user_id.lock().await;
        *uid = user_id;
    }

    pub async fn log_error(&self, error: ErrorEvent) {
        if !self.enabled {
            return;
        }

        let api_key = match &self.api_key {
            Some(key) => key.clone(),
            None => return,
        };

        let user_id = self.user_id.lock().await.clone();
        let distinct_id = user_id.unwrap_or_else(|| "anonymous".to_string());
        
        let event = PostHogEvent {
            api_key: api_key.clone(),
            event: "app_error".to_string(),
            distinct_id: distinct_id.clone(),
            timestamp: Some(Utc::now().to_rfc3339()),
            properties: serde_json::json!({
                "error_type": error.error_type,
                "message": error.message,
                "context": error.context,
                "stack": error.stack,
                "user_action": error.user_action,
                "recoverable": error.recoverable,
                "$lib": "pausa-app",
                "$lib_version": env!("CARGO_PKG_VERSION"),
            }),
        };

        self.queue_event(event).await;
    }

    pub async fn log_login(&self, login_event: LoginEvent) {
        if !self.enabled {
            return;
        }

        let api_key = match &self.api_key {
            Some(key) => key.clone(),
            None => return,
        };

        let user_id = self.user_id.lock().await.clone();
        let distinct_id = user_id.unwrap_or_else(|| "anonymous".to_string());
        
        let event = PostHogEvent {
            api_key: api_key.clone(),
            event: format!("login_{}", login_event.event),
            distinct_id: distinct_id.clone(),
            timestamp: Some(Utc::now().to_rfc3339()),
            properties: serde_json::json!({
                "provider": login_event.provider,
                "error": login_event.error,
                "$lib": "pausa-app",
                "$lib_version": env!("CARGO_PKG_VERSION"),
            }),
        };

        self.queue_event(event).await;
    }

    pub async fn log_metric(&self, metric: MetricEvent) {
        if !self.enabled {
            return;
        }

        let api_key = match &self.api_key {
            Some(key) => key.clone(),
            None => return,
        };

        let user_id = self.user_id.lock().await.clone();
        let distinct_id = user_id.unwrap_or_else(|| "anonymous".to_string());
        
        let mut properties = serde_json::json!({
            "value": metric.value,
            "$lib": "pausa-app",
            "$lib_version": env!("CARGO_PKG_VERSION"),
        });

        // Merge tags into properties
        if let Some(tags) = metric.tags {
            if let serde_json::Value::Object(mut props_obj) = properties {
                if let serde_json::Value::Object(tags_obj) = tags {
                    for (k, v) in tags_obj {
                        props_obj.insert(k, v);
                    }
                }
                properties = serde_json::Value::Object(props_obj);
            }
        }

        let event = PostHogEvent {
            api_key: api_key.clone(),
            event: metric.metric_name,
            distinct_id: distinct_id.clone(),
            timestamp: Some(Utc::now().to_rfc3339()),
            properties,
        };

        self.queue_event(event).await;
    }

    async fn queue_event(&self, event: PostHogEvent) {
        let mut queue = self.queue.lock().await;
        queue.push(event);

        // Auto-flush if queue gets too large
        if queue.len() >= 10 {
            drop(queue);
            let _ = self.flush().await;
        }
    }

    pub async fn flush(&self) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }

        let api_key = match &self.api_key {
            Some(key) => key.clone(),
            None => return Ok(()),
        };

        let mut queue = self.queue.lock().await;
        if queue.is_empty() {
            return Ok(());
        }

        let events: Vec<PostHogEvent> = queue.drain(..).collect();
        drop(queue);

        let client = reqwest::Client::new();
        
        // PostHog batch API endpoint
        // Convert events to the format PostHog expects
        let batch: Vec<serde_json::Value> = events.iter().map(|e| {
            serde_json::json!({
                "event": e.event,
                "distinct_id": e.distinct_id,
                "properties": e.properties,
                "timestamp": e.timestamp,
            })
        }).collect();
        
        let response = client
            .post("https://app.posthog.com/batch/")
            .json(&serde_json::json!({
                "api_key": api_key,
                "batch": batch
            }))
            .send()
            .await
            .map_err(|e| format!("Failed to send telemetry to PostHog: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!(
                "PostHog API returned error {}: {}",
                status, body
            ));
        }

        Ok(())
    }

    pub async fn log_session_completed(&self, session_data: serde_json::Value) {
        if !self.enabled {
            return;
        }

        let api_key = match &self.api_key {
            Some(key) => key.clone(),
            None => return,
        };

        let user_id = self.user_id.lock().await.clone();
        let distinct_id = user_id.unwrap_or_else(|| "anonymous".to_string());
        
        let mut properties = session_data;
        if let serde_json::Value::Object(ref mut props_obj) = properties {
            props_obj.insert("$lib".to_string(), serde_json::Value::String("pausa-app".to_string()));
            props_obj.insert("$lib_version".to_string(), serde_json::Value::String(env!("CARGO_PKG_VERSION").to_string()));
        }
        
        let event = PostHogEvent {
            api_key: api_key.clone(),
            event: "session_completed".to_string(),
            distinct_id: distinct_id.clone(),
            timestamp: Some(Utc::now().to_rfc3339()),
            properties,
        };

        self.queue_event(event).await;
    }
}

impl Default for TelemetryService {
    fn default() -> Self {
        Self::new()
    }
}
