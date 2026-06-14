use std::{collections::HashMap, sync::Arc};

use crate::{libs::llm::profile::LLMProfile, server::params::ServerParams};
use reqwest::Client;

pub struct MockersContext {
    pub server_params: ServerParams,
    pub client: Arc<Client>,
    pub llm_profiles: Arc<HashMap<String, LLMProfile>>,
}
