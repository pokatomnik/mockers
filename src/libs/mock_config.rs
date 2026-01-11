use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};
use tokio::fs::read_to_string;

use crate::libs::cache_mode::CacheMode;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MockConfig {
    pub delay_ms: Option<u64>,
    pub status_code: Option<u16>,
    pub headers: Option<HashMap<String, String>>,
    pub cache_mode: Option<CacheMode>,
}

pub async fn read_config(path: impl AsRef<Path>) -> Option<HashMap<String, MockConfig>> {
    read_to_string(path)
        .await
        .ok()
        .and_then(|contents| serde_json::from_str::<HashMap<String, MockConfig>>(&contents).ok())
}
