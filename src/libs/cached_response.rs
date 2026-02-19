use crate::libs::cache_mode::CacheMode;
use crate::libs::get_mime::get_mime;
use crate::libs::mock_config::MockConfig;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error as StdError;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CachedResponse {
    pub status_code: u16,
    pub delay_ms: u64,
    pub headers: HashMap<String, String>,
    pub body: String,
    #[serde(skip)]
    mime: Arc<RwLock<Option<String>>>,
}

impl From<&CachedResponse> for MockConfig {
    fn from(value: &CachedResponse) -> Self {
        MockConfig::new()
            .with_headers(value.headers.clone())
            .with_delay_ms(value.delay_ms)
            .with_status_code(value.status_code)
            .with_cache_mode(CacheMode::NoCache)
            .with_disabled_status(false)
    }
}

impl CachedResponse {
    pub fn new() -> CachedResponse {
        CachedResponse {
            status_code: StatusCode::OK.as_u16(),
            delay_ms: 0,
            headers: HashMap::new(),
            body: String::new(),
            mime: Arc::new(RwLock::new(None)),
        }
    }

    pub fn with_status_code(mut self, status_code: impl Into<u16>) -> Self {
        self.status_code = status_code.into();
        self
    }

    pub fn with_delay_ms(mut self, millis: impl Into<u64>) -> Self {
        self.delay_ms = millis.into();
        self
    }

    pub fn with_headers(mut self, headers: impl Into<HashMap<String, String>>) -> Self {
        self.headers = headers.into();
        self
    }

    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = body.into();
        self
    }

    pub async fn get_mime(&self) -> String {
        let read_guard = self.mime.read().await;
        if let Some(mime) = read_guard.as_ref() {
            return mime.clone();
        }
        drop(read_guard);
        let mime = get_mime(&Vec::from(self.body.as_bytes())).await;
        let mut write_guard = self.mime.write().await;
        *write_guard = Some(mime.to_owned());

        mime.to_owned()
    }

    pub async fn dump_response(
        &self,
        to: impl Into<PathBuf>,
    ) -> Result<(), Box<dyn StdError + Send + Sync>> {
        let absolute_response_path = to.into();
        tokio::fs::write(&absolute_response_path, &self.body).await?;

        Ok(())
    }

    pub async fn dump_config(
        &self,
        entry_name: &str,
        to: impl Into<PathBuf>,
    ) -> Result<(), Box<dyn StdError + Send + Sync>> {
        let mock_config = MockConfig::from(self);
        mock_config.try_write_to_file(to.into(), entry_name).await
    }
}
