use std::time::{Duration, Instant};

pub(crate) struct LLMResponse {
    ttl_ms: Duration,
    response: String,
    created_at: Instant,
}

impl LLMResponse {
    pub(crate) fn from_now(ttl_ms: Duration, response: String) -> Self {
        Self {
            ttl_ms,
            response,
            created_at: Instant::now(),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl_ms
    }

    pub fn get_response(&self) -> &str {
        self.response.as_str()
    }
}
