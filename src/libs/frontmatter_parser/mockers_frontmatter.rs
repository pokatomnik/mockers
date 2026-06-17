use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MockersPromptParams {
    #[serde(rename = "prompt")]
    prompt: Option<bool>,

    #[serde(rename = "api_endpoint")]
    api_endpoint: Option<String>,

    #[serde(rename = "env_key")]
    env_key: Option<String>,

    #[serde(rename = "model")]
    model: Option<String>,

    #[serde(rename = "proxy")]
    proxy: Option<String>,
}

impl MockersPromptParams {
    pub fn prompt(&self) -> Option<bool> {
        self.prompt
    }

    pub fn api_endpoint(&self) -> Option<&str> {
        self.api_endpoint.as_deref()
    }

    pub fn env_key(&self) -> Option<&str> {
        self.env_key.as_deref()
    }

    pub fn proxy(&self) -> Option<&str> {
        self.proxy.as_deref()
    }

    pub fn model(&self) -> Option<&str> {
        self.model.as_deref()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MockersFrontmatter {
    #[serde(rename = "$mockers")]
    mockers: Option<MockersPromptParams>,
}

impl MockersFrontmatter {
    pub fn mockers(&self) -> Option<&MockersPromptParams> {
        self.mockers.as_ref()
    }
}
