use serde::Deserialize;

use crate::libs::llm::request::Role;

#[derive(Deserialize)]
pub(crate) struct OpenAILikeProviderGenerateResponse {
    #[serde(rename = "choices")]
    choices: Vec<OpenAILikeProviderGenerateResponseChoice>,
}

impl OpenAILikeProviderGenerateResponse {
    pub fn get_choices(&self) -> &[OpenAILikeProviderGenerateResponseChoice] {
        &self.choices
    }
}

impl TryFrom<String> for OpenAILikeProviderGenerateResponse {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let result = serde_json::from_str::<OpenAILikeProviderGenerateResponse>(value.as_str())?;
        Ok(result)
    }
}

#[derive(Deserialize)]
pub(crate) struct OpenAILikeProviderGenerateResponseChoice {
    #[serde(rename = "finish_reason")]
    finish_reason: OpenAILikeProviderGenerateResponseFinishReason,

    #[serde(rename = "message")]
    message: OpenAILikeProviderGenerateResponseMessage,
}

impl OpenAILikeProviderGenerateResponseChoice {
    pub fn finish_reason(&self) -> &OpenAILikeProviderGenerateResponseFinishReason {
        &self.finish_reason
    }

    pub fn message(&self) -> &OpenAILikeProviderGenerateResponseMessage {
        &self.message
    }
}

#[derive(serde::Deserialize)]
pub(crate) struct OpenAILikeProviderGenerateResponseMessage {
    #[serde(rename = "role")]
    role: Role,

    #[serde(rename = "content")]
    content: Option<String>,
}

impl OpenAILikeProviderGenerateResponseMessage {
    pub fn role(&self) -> Role {
        self.role
    }

    pub fn content(&self) -> Option<&str> {
        self.content.as_deref()
    }
}

#[derive(serde::Deserialize, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub(crate) enum OpenAILikeProviderGenerateResponseFinishReason {
    #[serde(rename = "stop")]
    Stop,

    #[serde(rename = "length")]
    Length,

    #[serde(rename = "tool_calls")]
    ToolCalls,

    #[serde(rename = "content_filter")]
    ContentFilter,

    #[serde(rename = "error")]
    Error,
}
