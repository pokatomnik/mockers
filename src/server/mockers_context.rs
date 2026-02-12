use std::sync::Arc;

use crate::libs::in_memory_mocks::InMemoryMocks;
use crate::server::params::ServerParams;
use reqwest::Client;

pub struct MockersContext {
    pub server_params: ServerParams,
    pub client: Arc<Client>,
    pub response_cache: Arc<InMemoryMocks>,
}
