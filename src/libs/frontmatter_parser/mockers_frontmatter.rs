use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MockersPromptParams {
    #[serde(rename = "prompt")]
    prompt: Option<bool>,
    api_endpoint: Option<String>,
    env_key: Option<String>,
    model: Option<String>,
    proxy: Option<String>,
}

impl MockersPromptParams {
    fn new(
        prompt: Option<bool>,
        model: Option<String>,
        api_endpoint: Option<String>,
        env_key: Option<String>,
        proxy: Option<String>,
    ) -> Self {
        Self {
            prompt,
            model,
            api_endpoint,
            env_key,
            proxy,
        }
    }

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
    pub fn new(mockers: Option<MockersPromptParams>) -> Self {
        Self { mockers }
    }

    pub fn mockers(&self) -> Option<&MockersPromptParams> {
        self.mockers.as_ref()
    }
}
