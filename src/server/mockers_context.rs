use std::sync::Arc;

use crate::libs::response_cache::InMemoryMocks;
use crate::server::params::ServerParams;
use reqwest::Client;

pub struct MockersContext {
    pub server_params: ServerParams,
    pub client: Arc<Client>,
    pub response_cache: Arc<InMemoryMocks>,
}
