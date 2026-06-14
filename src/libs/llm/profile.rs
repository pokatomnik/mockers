use std::{
    num::NonZeroUsize,
    sync::{Arc, OnceLock},
    time::Duration,
};

use hyper::Method;
use reqwest::{Client, Proxy};
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;

use crate::libs::llm::{
    request::{self, Role},
    response::{
        OpenAILikeProviderGenerateResponse, OpenAILikeProviderGenerateResponseFinishReason,
    },
    response_cache::LLMProfileResponseCache,
};

const DEFAULT_CACHE_TTL_DURATION: Duration = Duration::from_millis(120_000);
const DEFAULT_CACHE_MAX_SIZE: NonZeroUsize = NonZeroUsize::new(42).unwrap();
const SIMULTANEOUS_REQUESTS: usize = 1;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct LLMProfile {
    #[serde(rename = "endpointURL")]
    endpoint_url: String,

    #[serde(rename = "authTokenEnvKey")]
    auth_token_env_key: Option<String>,

    #[serde(rename = "model")]
    model: String,

    #[serde(rename = "proxy")]
    proxy: Option<String>,

    #[serde(skip)]
    auth_token: OnceLock<Option<String>>,

    #[serde(skip)]
    client: OnceLock<Client>,

    #[serde(skip)]
    semaphore: OnceLock<Arc<Semaphore>>,

    #[serde(skip)]
    cache: OnceLock<Arc<LLMProfileResponseCache>>,
}

impl LLMProfile {
    fn auth_token(&self) -> Option<String> {
        let token = self.auth_token.get_or_init(|| {
            let Some(ref token_env_key) = self.auth_token_env_key else {
                return None;
            };
            std::env::var(token_env_key.as_str()).ok()
        });

        token.clone()
    }

    fn client(&self) -> &Client {
        self.client.get_or_init(|| {
            let mut builder = Client::builder().no_proxy();
            let proxy = self.proxy.as_ref().and_then(|s| Proxy::all(s).ok());

            if let Some(proxy) = proxy {
                builder = builder.proxy(proxy);
            }

            builder.build().expect("Cannot initalize HTTP client")
        })
    }

    fn get_semaphore(&self) -> Arc<Semaphore> {
        self.semaphore
            .get_or_init(|| Arc::new(Semaphore::new(SIMULTANEOUS_REQUESTS)))
            .clone()
    }

    fn get_cache(&self) -> Arc<LLMProfileResponseCache> {
        self.cache
            .get_or_init(|| {
                Arc::new(LLMProfileResponseCache::new(
                    DEFAULT_CACHE_TTL_DURATION,
                    DEFAULT_CACHE_MAX_SIZE,
                ))
            })
            .clone()
    }

    async fn ask_internal(&self, prompt: impl AsRef<str>) -> anyhow::Result<String> {
        let request_body = request::LLMProviderRequestBody::new(
            self.model.to_string(),
            prompt.as_ref().to_string(),
        );
        let api_url = format!(
            "{}/chat/completions",
            self.endpoint_url.trim().trim_matches('/')
        );

        let mut request = self.client().request(Method::POST, api_url);
        let request_body_json = serde_json::to_string_pretty(&request_body)?;

        if let Some(token) = self.auth_token() {
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

    pub async fn ask(&self, prompt: impl AsRef<str>) -> anyhow::Result<String> {
        let semaphore = self.get_semaphore();
        let _permit = semaphore.acquire().await?;
        let cache = self.get_cache();
        if let Some(cached_response) = cache.get(prompt.as_ref()).await {
            return Ok(cached_response);
        }
        let response = self.ask_internal(prompt.as_ref()).await?;
        cache.set(prompt.as_ref(), response.clone()).await;
        Ok(response)
    }
}
