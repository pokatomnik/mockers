use std::sync::Arc;

use crate::server::params::ServerParams;
use reqwest::Client;

pub struct MockersContext {
    pub server_params: ServerParams,
    pub client: Arc<Client>,
}
