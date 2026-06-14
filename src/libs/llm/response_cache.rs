use std::{num::NonZeroUsize, sync::Arc, time::Duration};

use lru::LruCache;
use tokio::sync::Mutex;
use xxhash_rust::xxh3::xxh3_128_with_seed;

use crate::libs::llm::llm_response::LLMResponse;

static SEED: u64 = 42;

#[derive(Debug)]
pub(crate) struct LLMProfileResponseCache {
    ttl_ms: Duration,
    cache: Arc<Mutex<LruCache<u128, LLMResponse>>>,
}

impl LLMProfileResponseCache {
    pub fn new(ttl_ms: Duration, max_size: NonZeroUsize) -> Self {
        let lru = LruCache::new(max_size);
        Self {
            ttl_ms,
            cache: Arc::new(Mutex::new(lru)),
        }
    }

    #[inline]
    fn hash(&self, bytes: &[u8]) -> u128 {
        xxh3_128_with_seed(bytes, SEED)
    }

    pub async fn set(&self, prompt: impl AsRef<str>, response: impl AsRef<str>) {
        let prompt = prompt.as_ref();
        let hash = self.hash(prompt.as_bytes());

        let response = response.as_ref();

        let mut cache = self.cache.lock().await;
        let llm_response = LLMResponse::from_now(self.ttl_ms.clone(), response.to_string());
        cache.put(hash, llm_response);
    }

    pub async fn get(&self, prompt: impl AsRef<str>) -> Option<String> {
        let prompt = prompt.as_ref();
        let hash = self.hash(prompt.as_bytes());
        let mut cache = self.cache.lock().await;

        let Some(response) = cache.get(&hash) else {
            return None;
        };

        if response.is_expired() {
            return None;
        }

        Some(response.get_response().to_string())
    }
}
