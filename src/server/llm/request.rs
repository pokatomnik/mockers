use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LLMProviderRequestBody {
    #[serde(rename = "model")]
    model: String,

    #[serde(rename = "messages")]
    messages: Vec<LLMProviderMessage>,

    /// Always false. No need to stream response.
    #[serde(rename = "stream")]
    stream: bool,

    /// Always 0. Deterministic responses required.
    #[serde(rename = "temperature")]
    temperature: f32,
}

impl LLMProviderRequestBody {
    fn get_system_prompt(suffix: &str) -> LLMProviderMessage {
        let content = include_str!("./system.md").to_string();
        let mut buf = String::with_capacity(content.len() + suffix.len() + 1);
        buf.push_str(content.as_str());
        buf.push('\n');
        buf.push_str(suffix);

        LLMProviderMessage {
            role: Role::System,
            content: buf,
        }
    }

    fn get_user_message(prompt: String) -> LLMProviderMessage {
        LLMProviderMessage {
            role: Role::User,
            content: prompt,
        }
    }

    fn get_messages(prompt: String, suffix: &str) -> Vec<LLMProviderMessage> {
        vec![
            Self::get_system_prompt(suffix),
            Self::get_user_message(prompt),
        ]
    }

    pub fn new(model: String, prompt: String, suffix: &str) -> Self {
        Self {
            model,
            messages: Self::get_messages(prompt, suffix),
            stream: false,
            temperature: 0f32,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LLMProviderMessage {
    #[serde(rename = "role")]
    role: Role,

    #[serde(rename = "content")]
    content: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub(crate) enum Role {
    #[serde(rename = "user")]
    User,

    #[serde(rename = "assistant")]
    Assistant,

    #[serde(rename = "system")]
    System,
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::User => f.write_str("user"),
            Role::Assistant => f.write_str("assistant"),
            Role::System => f.write_str("system"),
        }
    }
}
