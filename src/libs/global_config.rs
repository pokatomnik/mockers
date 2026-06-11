use crate::libs::get_info_async::GetInfoAsync;
use crate::libs::path_buf_ext::PathBufExt;
use crate::libs::preflight_type::PreflightType;
use crate::middlewares::logger::VerbosityLevel;
use reqwest::Proxy;
use serde::{Deserialize, Serialize};
use std::collections::LinkedList;
use std::env::current_dir;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

pub(crate) static GLOBAL_CONFIG_FILE_NAME: &'static str = ".mockers";

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GlobalConfig {
    host: Option<String>,
    port: Option<u16>,
    https_port: Option<u16>,
    mocks: Option<String>,
    cors: Option<bool>,
    preflight: Option<PreflightType>,
    delay_ms: Option<u64>,
    origin: Option<String>,
    admin_base_url: Option<String>,
    log_request: Option<VerbosityLevel>,
    verbosity: Option<VerbosityLevel>,
    proxy_body_max_bytes: Option<usize>,
    proxy: Option<String>,
}

pub(crate) trait GlobalConfigBuilder {
    fn with_host(self, host: String) -> Self;
    fn with_port(self, port: u16) -> Self;
    fn with_https_port(self, https_port: u16) -> Self;
    fn with_mocks(self, mocks: String) -> Self;
    fn with_cors(self, cors: bool) -> Self;
    fn with_preflight(self, preflight: PreflightType) -> Self;
    fn with_delay_ms(self, delay_ms: u64) -> Self;
    #[allow(unused)]
    fn with_origin(self, origin: String) -> Self;
    fn with_admin_base_url(self, admin_base_url: String) -> Self;
    fn with_log_request(self, log_request: VerbosityLevel) -> Self;
    fn with_verbosity(self, verbosity: VerbosityLevel) -> Self;
    fn with_proxy_body_max_bytes(self, proxy_body_max_bytes: usize) -> Self;
}

impl GlobalConfigBuilder for GlobalConfig {
    fn with_host(mut self, host: String) -> Self {
        self.host = Some(host);
        self
    }

    fn with_port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    fn with_https_port(mut self, https_port: u16) -> Self {
        self.https_port = Some(https_port);
        self
    }

    fn with_mocks(mut self, mocks: String) -> Self {
        self.mocks = Some(mocks);
        self
    }

    fn with_cors(mut self, cors: bool) -> Self {
        self.cors = Some(cors);
        self
    }

    fn with_preflight(mut self, preflight: PreflightType) -> Self {
        self.preflight = Some(preflight);
        self
    }

    fn with_delay_ms(mut self, delay_ms: u64) -> Self {
        self.delay_ms = Some(delay_ms);
        self
    }

    fn with_origin(mut self, origin: String) -> Self {
        self.origin = Some(origin);
        self
    }

    fn with_admin_base_url(mut self, admin_base_url: String) -> Self {
        self.admin_base_url = Some(admin_base_url);
        self
    }

    fn with_log_request(mut self, log_request: VerbosityLevel) -> Self {
        self.log_request = Some(log_request);
        self
    }

    fn with_verbosity(mut self, verbosity: VerbosityLevel) -> Self {
        self.verbosity = Some(verbosity);
        self
    }

    fn with_proxy_body_max_bytes(mut self, proxy_body_max_bytes: usize) -> Self {
        self.proxy_body_max_bytes = Some(proxy_body_max_bytes);
        self
    }
}

pub(crate) trait WithGlobalConfigAPI {
    async fn get_global_config(&self) -> &GlobalConfigAPI;
}

impl GlobalConfig {
    pub fn merge(&mut self, other: Option<&Self>) -> Self {
        let host = other
            .map(|o| o.host.clone())
            .flatten()
            .or_else(|| self.host.clone());
        let port = other.map(|o| o.port).flatten().or(self.port);
        let https_port = other.map(|o| o.https_port).flatten().or(self.https_port);
        let mocks = other
            .map(|o| o.mocks.clone())
            .flatten()
            .or_else(|| self.mocks.clone());
        let cors = other.map(|o| o.cors).flatten().or(self.cors);
        let preflight = other.map(|o| o.preflight).flatten().or(self.preflight);
        let delay_ms = other.map(|o| o.delay_ms).flatten().or(self.delay_ms);
        let origin = other
            .map(|o| o.origin.clone())
            .flatten()
            .or_else(|| self.origin.clone());
        let admin_base_url = other
            .map(|o| o.admin_base_url.clone())
            .flatten()
            .or_else(|| self.admin_base_url.clone());
        let log_request = other.map(|o| o.log_request).flatten().or(self.log_request);
        let verbosity = other.map(|o| o.verbosity).flatten().or(self.verbosity);
        let proxy_body_max_bytes = other
            .map(|o| o.proxy_body_max_bytes)
            .flatten()
            .or(self.proxy_body_max_bytes);
        let proxy = other
            .map(|o| o.proxy.clone())
            .flatten()
            .or_else(|| self.proxy.clone());
        GlobalConfig {
            host,
            port,
            https_port,
            mocks,
            cors,
            preflight,
            delay_ms,
            origin,
            admin_base_url,
            log_request,
            verbosity,
            proxy_body_max_bytes,
            proxy,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct GlobalConfigAPI {
    config: Arc<RwLock<Option<GlobalConfig>>>,
}

impl GlobalConfigAPI {
    fn get_all_paths(start: impl AsRef<Path>) -> LinkedList<PathBuf> {
        let start = start.as_ref().to_path_buf();
        let mut result = LinkedList::new();
        result.push_front(start.clone());

        let mut current = start;
        loop {
            let parent = current.with_last_removed();
            if parent == current {
                break;
            }
            result.push_front(parent.clone());

            current = parent;
        }

        result
    }

    async fn read_config_by_path(path: impl AsRef<Path>) -> Option<GlobalConfig> {
        let contents = tokio::fs::read_to_string(path).await.ok()?;
        let result = serde_json::from_str::<GlobalConfig>(&contents).ok();
        return result;
    }

    async fn get_merged_config() -> GlobalConfig {
        let mut result = GlobalConfig::default();
        let all_dirs = current_dir().map(|dir| Self::get_all_paths(dir));
        let Ok(all_dirs) = all_dirs else {
            return result;
        };

        let all_global_config_paths: LinkedList<PathBuf> = all_dirs
            .iter()
            .map(|dir| dir.join(GLOBAL_CONFIG_FILE_NAME))
            .collect();

        let mut all_configs = LinkedList::new();

        for config in all_global_config_paths {
            let config = Self::read_config_by_path(config).await;
            all_configs.push_front(config);
        }

        while let Some(config_optional) = all_configs.pop_front() {
            result = result.merge(config_optional.as_ref());
        }

        result
    }

    async fn get_cached_config(&self) -> Option<GlobalConfig> {
        let config = self.config.read().await;
        config.clone()
    }

    async fn set_cached_config(&self, config: GlobalConfig) {
        let mut c = self.config.write().await;
        *c = Some(config)
    }

    pub async fn new() -> Self {
        GlobalConfigAPI {
            config: Arc::new(RwLock::new(None)),
        }
    }

    async fn get_config(&self) -> GlobalConfig {
        let config = self.get_cached_config().await;
        if let Some(config) = config {
            return config;
        }

        let new_config = Self::get_merged_config().await;
        self.set_cached_config(new_config.clone()).await;

        new_config
    }

    pub async fn get_host(&self) -> Option<String> {
        self.get_config().await.host
    }

    pub async fn get_port(&self) -> Option<u16> {
        self.get_config().await.port
    }

    pub async fn get_https_port(&self) -> Option<u16> {
        self.get_config().await.https_port
    }

    pub async fn get_mocks(&self) -> Option<String> {
        self.get_config().await.mocks
    }

    pub async fn get_cors(&self) -> Option<bool> {
        self.get_config().await.cors
    }

    pub async fn get_preflight(&self) -> Option<PreflightType> {
        self.get_config().await.preflight
    }

    pub async fn get_delay_ms(&self) -> Option<u64> {
        self.get_config().await.delay_ms
    }

    pub async fn get_origin(&self) -> Option<String> {
        self.get_config().await.origin
    }

    pub async fn get_admin_base_url(&self) -> Option<String> {
        self.get_config().await.admin_base_url
    }

    pub async fn get_log_request(&self) -> Option<VerbosityLevel> {
        self.get_config().await.log_request
    }

    pub async fn get_verbosity_level(&self) -> Option<VerbosityLevel> {
        self.get_config().await.verbosity
    }

    pub async fn get_proxy_body_max_bytes(&self) -> Option<usize> {
        self.get_config().await.proxy_body_max_bytes
    }

    pub async fn get_proxy(&self) -> Option<Proxy> {
        match self.get_config().await.proxy {
            Some(p) => Proxy::all(p).ok(),
            None => None,
        }
    }
}

impl GetInfoAsync for GlobalConfigAPI {
    async fn get_help(&self, title: &str) -> String {
        let mut buf = String::from(format!("{}:{}", title, Self::EOL));
        buf.push_str(&format!("================={}", Self::EOL));
        let host_info = format!(
            "Host:{}{}{}",
            Self::TAB.repeat(3),
            self.get_host()
                .await
                .unwrap_or_else(|| Self::UNSET.to_string()),
            Self::EOL
        );
        let port_info = format!(
            "Port:{}{}{}",
            Self::TAB.repeat(3),
            self.get_port()
                .await
                .map(|v| v.to_string())
                .unwrap_or_else(|| Self::UNSET.to_string()),
            Self::EOL
        );
        let https_port_info = format!(
            "HTTPS Port:{}{}{}",
            Self::TAB.repeat(2),
            self.get_https_port()
                .await
                .map(|v| v.to_string())
                .unwrap_or_else(|| Self::UNSET.to_string()),
            Self::EOL
        );
        let mocks_path_info = format!(
            "Mocks path:{}{}{}",
            Self::TAB.repeat(2),
            self.get_mocks()
                .await
                .unwrap_or_else(|| Self::UNSET.to_string()),
            Self::EOL,
        );
        let cors_info = format!(
            "Cors:{}{}{}",
            Self::TAB.repeat(3),
            self.get_cors()
                .await
                .map(|v| v.to_string())
                .unwrap_or_else(|| Self::UNSET.to_string()),
            Self::EOL
        );
        let preflight_info = format!(
            "Preflight:{}{}{}",
            Self::TAB.repeat(2),
            self.get_preflight()
                .await
                .map(|pt| pt.to_string())
                .unwrap_or_else(|| Self::UNSET.to_string()),
            Self::EOL
        );
        let delay_ms_info = format!(
            "Delay in milliseconds:{}{}{}",
            Self::TAB,
            self.get_delay_ms()
                .await
                .map(|v| v.to_string())
                .unwrap_or_else(|| Self::UNSET.to_string()),
            Self::EOL
        );
        let origin_info = format!(
            "Origin:{}{}{}",
            Self::TAB.repeat(3),
            self.get_origin()
                .await
                .unwrap_or_else(|| Self::UNSET.to_string()),
            Self::EOL,
        );
        let admin_base_url_info = format!(
            "Admin base URL:{}{}{}",
            Self::TAB.repeat(2),
            self.get_admin_base_url()
                .await
                .unwrap_or_else(|| Self::UNSET.to_string()),
            Self::EOL,
        );
        let log_request_info = format!(
            "Requests log level:{}{}{}",
            Self::TAB,
            self.get_log_request()
                .await
                .map(|v| v.to_string())
                .unwrap_or_else(|| Self::UNSET.to_string()),
            Self::EOL,
        );
        let verbosity_level_info = format!(
            "Verbosity level:{}{}{}",
            Self::TAB,
            self.get_verbosity_level()
                .await
                .map(|v| v.to_string())
                .unwrap_or_else(|| Self::UNSET.to_string()),
            Self::EOL
        );
        let proxy_body_max_bytes = format!(
            "Proxy body max bytes:{}{}{}",
            Self::TAB,
            self.get_proxy_body_max_bytes()
                .await
                .map(|v| v.to_string())
                .unwrap_or_else(|| Self::UNSET.to_string()),
            Self::EOL,
        );

        let proxy_text = match self.get_proxy().await {
            Some(_) => "Proxy IS set",
            None => "Proxy is NOT set",
        };
        let proxy = format!("Proxy:{}{}{}", Self::TAB.repeat(3), proxy_text, Self::EOL);

        buf.push_str(&host_info);
        buf.push_str(&port_info);
        buf.push_str(&https_port_info);
        buf.push_str(&mocks_path_info);
        buf.push_str(&cors_info);
        buf.push_str(&preflight_info);
        buf.push_str(&delay_ms_info);
        buf.push_str(&origin_info);
        buf.push_str(&admin_base_url_info);
        buf.push_str(&log_request_info);
        buf.push_str(&verbosity_level_info);
        buf.push_str(&proxy_body_max_bytes);
        buf.push_str(&proxy);

        buf
    }
}
