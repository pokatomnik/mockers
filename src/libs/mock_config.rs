use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

use crate::libs::cache_mode::CacheMode;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MockConfig {
    delay_ms: Option<u64>,
    status_code: Option<u16>,
    headers: Option<HashMap<String, String>>,
    cache_mode: Option<CacheMode>,
}

impl MockConfig {
    pub fn new() -> MockConfig {
        MockConfig {
            delay_ms: None,
            status_code: None,
            headers: None,
            cache_mode: None,
        }
    }

    pub fn with_delay_ms(mut self, delay_ms: u64) -> Self {
        self.delay_ms = Some(delay_ms);
        self
    }

    pub fn delay_ms(&self) -> Option<u64> {
        self.delay_ms
    }

    pub fn with_status_code(mut self, status_code: u16) -> Self {
        self.status_code = Some(status_code);
        self
    }

    pub fn status_code(&self) -> Option<u16> {
        self.status_code
    }

    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = Some(headers);
        self
    }

    pub fn headers(&self) -> Option<&HashMap<String, String>> {
        if let Some(ref headers) = self.headers {
            return Some(headers);
        }
        None
    }

    pub fn with_cache_mode(mut self, cache_mode: CacheMode) -> Self {
        self.cache_mode = Some(cache_mode);
        self
    }

    pub fn cache_mode(&self) -> Option<CacheMode> {
        self.cache_mode.clone()
    }
}

pub async fn read_config(path: impl AsRef<Path>) -> Option<HashMap<String, MockConfig>> {
    tokio::fs::read_to_string(path)
        .await
        .ok()
        .and_then(|contents| serde_json::from_str::<HashMap<String, MockConfig>>(&contents).ok())
}
