use crate::{controllers::entities::mock_info::MockInfo, libs::mock_config::MockConfig};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MockCreateParams {
    /// Path and HTTP method of the mock.
    mock_key: MockInfo,
    /// Configuration for the mock.
    mock_config: MockConfig,
    /// Body of the mock response.
    body: String,
}

impl MockCreateParams {
    pub fn mock_key(&self) -> &MockInfo {
        &self.mock_key
    }

    pub fn mock_config(&self) -> &MockConfig {
        &self.mock_config
    }

    pub fn body(&self) -> &str {
        &self.body
    }
}
