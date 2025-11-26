use std::sync::Arc;

use reqwest::Client;

use crate::server::params::ServerParams;

pub struct MockersContext {
    pub server_params: ServerParams,
    pub client: Arc<Client>,
}
