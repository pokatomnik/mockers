use crate::entities::cache_mode::CacheMode;
use crate::entities::mock_defaults::{DEFAULT_DELAY_MS, DEFAULT_STATUS_CODE};
use serde::{Deserialize, Serialize};
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
    /// Attempts to read and parse a JSON file into a map of MockConfig entries.
    ///
    /// # Arguments
    ///
    /// * `path` - A path to the JSON file containing the configuration.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `HashMap` where keys are entry names (Strings)
    /// and values are `MockConfig` objects. Returns an error if the file cannot
    /// be read or if the content is not valid JSON.
    pub async fn try_read_from_file(
        path: impl AsRef<Path>,
    ) -> anyhow::Result<HashMap<String, Self>> {
        let contents = tokio::fs::read_to_string(path).await?;
        let result = Self::try_from_str(contents)?;
        Ok(result)
    }

    pub fn try_from_str(source: impl Into<String>) -> anyhow::Result<HashMap<String, Self>> {
        serde_json::from_str::<HashMap<String, Self>>(&source.into()).map_err(anyhow::Error::from)
    }

    pub async fn try_remove_from_file(
        path: impl AsRef<Path>,
        entry_name: impl Into<String>,
    ) -> anyhow::Result<()> {
        let entry_name = entry_name.into();
        let mut config_map = Self::try_read_from_file(path.as_ref()).await?;
        config_map.remove(&entry_name);

        if config_map.is_empty() {
            tokio::fs::remove_file(path.as_ref()).await?;
            return Ok(());
        }

        let updated_json_str = serde_json::to_string_pretty(&config_map)?;

        tokio::fs::write(&path, updated_json_str).await?;

        return Ok(());
    }

    pub async fn try_write_to_file(
        &self,
        path: impl AsRef<Path>,
        entry_name: impl Into<String>,
    ) -> anyhow::Result<()> {
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

    pub fn new() -> Self {
        Self {
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
