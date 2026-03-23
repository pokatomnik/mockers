use crate::libs::cache_mode::CacheMode;
use crate::libs::create_params::{DEFAULT_DELAY_MS, DEFAULT_STATUS_CODE};
use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::{collections::HashMap, path::Path};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MockConfig {
    delay_ms: Option<u64>,
    status_code: Option<u16>,
    headers: Option<HashMap<String, String>>,
    cache_mode: Option<CacheMode>,
    disabled: Option<bool>,
}

impl Default for MockConfig {
    fn default() -> Self {
        MockConfig {
            delay_ms: Some(DEFAULT_DELAY_MS),
            headers: Some(HashMap::new()),
            cache_mode: Some(CacheMode::NoCache),
            status_code: Some(DEFAULT_STATUS_CODE),
            disabled: Some(false),
        }
    }
}

impl MockConfig {
    pub async fn try_read_from_file(
        path: impl AsRef<Path>,
    ) -> Result<HashMap<String, MockConfig>, Box<dyn StdError + Send + Sync>> {
        let contents = tokio::fs::read_to_string(path).await?;
        serde_json::from_str::<HashMap<String, MockConfig>>(&contents).map_err(Box::from)
    }

    pub async fn try_remove_from_file(
        path: impl AsRef<Path>,
        entry_name: impl Into<String>,
    ) -> Result<(), Box<dyn StdError + Send + Sync>> {
        let entry_name = entry_name.into();
        let mut config_map = Self::try_read_from_file(path.as_ref()).await?;
        config_map.remove(&entry_name);

        if config_map.is_empty() {
            return tokio::fs::remove_file(path.as_ref())
                .await
                .map_err(|e| -> Box<dyn StdError + Send + Sync> { e.into() });
        }

        let updated_json_str = serde_json::to_string_pretty(&config_map)?;

        tokio::fs::write(&path, updated_json_str).await?;

        return Ok(());
    }

    pub async fn try_write_to_file(
        &self,
        path: impl AsRef<Path>,
        entry_name: impl Into<String>,
    ) -> Result<(), Box<dyn StdError + Send + Sync>> {
        let json = match Self::try_read_from_file(&path).await {
            Ok(mut existing_config) => {
                existing_config.insert(entry_name.into(), self.clone());
                serde_json::to_string_pretty(&existing_config)?
            }
            Err(_) => {
                let mut new_config = HashMap::with_capacity(1);
                new_config.insert(entry_name.into(), self.clone());
                serde_json::to_string_pretty(&new_config)?
            }
        };
        tokio::fs::write(&path, json).await?;

        Ok(())
    }

    pub fn new() -> MockConfig {
        MockConfig {
            delay_ms: None,
            status_code: None,
            headers: None,
            cache_mode: None,
            disabled: None,
        }
    }

    pub fn with_delay_ms(mut self, delay_ms: u64) -> Self {
        self.delay_ms = Some(delay_ms);
        self
    }

    pub fn delay_ms(&self) -> Option<u64> {
        self.delay_ms
    }

    pub fn with_disabled_status(mut self, is_disabled: bool) -> Self {
        self.disabled = Some(is_disabled);
        self
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled.unwrap_or(false)
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
