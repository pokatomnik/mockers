use std::sync::Arc;

use crate::{libs::llm::client::LLMClient, server::params::ServerParams};
use reqwest::Client;

pub struct MockersContext {
    pub server_params: ServerParams,
    pub client: Arc<Client>,
    pub llm_client: Arc<LLMClient>,
}
