use std::{num::NonZeroUsize, sync::Arc, time::Duration};

use anyhow::Context;
use hyper::Method;
use reqwest::{Client, Proxy};
use tokio::sync::Semaphore;

use crate::libs::frontmatter_parser::mockers_frontmatter::MockersPromptParams;
use crate::libs::llm::request::{self, Role};
use crate::libs::llm::response::OpenAILikeProviderGenerateResponse;
use crate::libs::llm::response::OpenAILikeProviderGenerateResponseFinishReason;
use crate::libs::llm::response_cache::LLMProfileResponseCache;

const DEFAULT_CACHE_TTL_DURATION: Duration = Duration::from_millis(120_000);
const DEFAULT_CACHE_MAX_SIZE: NonZeroUsize = NonZeroUsize::new(42).unwrap();
const SIMULTANEOUS_REQUESTS: usize = 1;

#[derive(Clone, Debug)]
pub(crate) struct LLMClient {
    semaphore: Arc<Semaphore>,
    cache: Arc<LLMProfileResponseCache>,
}

impl LLMClient {
    pub fn new() -> Self {
        let semaphore = Arc::new(Semaphore::new(SIMULTANEOUS_REQUESTS));
        let cache = Arc::new(LLMProfileResponseCache::new(
            DEFAULT_CACHE_TTL_DURATION,
            DEFAULT_CACHE_MAX_SIZE,
        ));
        Self { semaphore, cache }
    }

    fn client(proxy: Option<&str>) -> anyhow::Result<Client> {
        let mut builder = Client::builder().no_proxy();
        let proxy = proxy.and_then(|s| Proxy::all(s).ok());

        if let Some(proxy) = proxy {
            builder = builder.proxy(proxy);
        }

        builder
            .build()
            .with_context(|| "Failed to build HTTP client")
    }

    fn auth_token(maybe_proxy_env_key: Option<&str>) -> Option<String> {
        let Some(proxy_env_key) = maybe_proxy_env_key else {
            return None;
        };

        std::env::var(proxy_env_key).ok()
    }

    async fn ask_internal(
        &self,
        frontmatter: &MockersPromptParams,
        prompt: &str,
    ) -> anyhow::Result<String> {
        let Some(api_url) = frontmatter.api_endpoint() else {
            anyhow::bail!("API endpoint not specified");
        };
        let Some(model) = frontmatter.model() else {
            anyhow::bail!("Model not defined");
        };
        let request_body =
            request::LLMProviderRequestBody::new(model.to_string(), prompt.to_string());

        let mut request = Self::client(frontmatter.proxy())?.request(Method::POST, api_url);
        let request_body_json = serde_json::to_string_pretty(&request_body)?;

        if let Some(token) = Self::auth_token(frontmatter.env_key()) {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        request = request.header("Content-Type", "application/json");

        let response = request.body(request_body_json).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to ask LLM: {}", response.status()));
        }

        let response_text = response.text().await?;

        let response: OpenAILikeProviderGenerateResponse = response_text.try_into()?;

        let Some(llm_response_choice) = response.get_choices().get(0) else {
            return Ok(String::default());
        };

        if llm_response_choice.finish_reason()
            != &OpenAILikeProviderGenerateResponseFinishReason::Stop
        {
            anyhow::bail!(
                "LLM response finish reason is not stop: {:?}",
                llm_response_choice.finish_reason()
            )
        }

        let message = llm_response_choice.message();

        if message.role() != Role::Assistant {
            anyhow::bail!("LLM response role is not assistant: {:?}", message.role())
        }

        let content = message
            .content()
            .map(ToString::to_string)
            .unwrap_or_default();

        Ok(content)
    }

    pub async fn ask(
        &self,
        frontmatter: &MockersPromptParams,
        prompt: &str,
    ) -> anyhow::Result<String> {
        let _permit = self.semaphore.acquire().await?;
        if let Some(cached_response) = self.cache.get(prompt).await {
            return Ok(cached_response);
        }
        let response = self.ask_internal(frontmatter, prompt).await?;
        self.cache.set(prompt, response.clone()).await;
        Ok(response)
    }
}
