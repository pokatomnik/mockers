use crate::libs::absolute_mocks_path::{AbsoluteMocksPath, WithMocks};
use crate::libs::create_params::DEFAULT_DELAY_MS;
use crate::libs::get_info_async::GetInfoAsync;
use crate::libs::global_config::{GlobalConfigAPI, WithGlobalConfigAPI};
use crate::libs::preflight_type::PreflightType;
use crate::libs::tls_acceptor_ext::TLSAcceptorLoader;
use crate::middlewares::logger::VerbosityLevel;
use crate::server::mockers_router::mockers_router;
use crate::server::signal::make_signal;
use clap::{ArgAction, Args};
use hyper::server::conn::http1;
use hyper::service::Service;
use hyper_util::rt::TokioIo;
use log::{error, info};
use path_absolutize::Absolutize;
use routerify_ng::RouterService;
use std::error::Error as StdError;
use std::io::Error as IoError;
use std::io::ErrorKind;
use std::net::ToSocketAddrs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::OnceCell;
use tokio_rustls::TlsAcceptor;

pub const DEFAULT_HOST: &'static str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 8080;
pub const DEFAULT_MOCKS_DIR_NAME: &'static str = "mocks";
pub const DEFAULT_MOCKS_RESPONSE_DELAY: u64 = 0;
pub const DEFAULT_CORS_ENABLED: bool = false;
pub const CONFIG_FILE_NAME: &'static str = "config.json";
// This variable should only be used when initializing the global configuration file.
pub const DEFAULT_ADMIN_BASE_URL: &'static str = "/__admin";
/// Maximum number of bytes allowed to fetch from remote server
pub const DEFAULT_PROXY_RESPONSE_BODY_BYTES: usize = 1024 * 1024 * 16;
/// Default maxumum number of bytes allowed to fetch from remote server
pub const HARD_MAX_PROXY_RESPONSE_BODY_BYTES: usize = DEFAULT_PROXY_RESPONSE_BODY_BYTES * 2;

static BANNER_MSG: &'static str = include_str!("./banner.txt");

#[derive(Args, Debug, Clone)]
#[clap(rename_all = "kebab-case")]
pub struct ServerParams {
    #[arg(long, help = "Host to listen on")]
    host: Option<String>,

    #[arg(long, short, help = "Port to listen on")]
    port: Option<u16>,

    #[arg(long, short, help = "Path to the directory containing mock files")]
    mocks: Option<String>,

    // CORS flag uses specific clap attributes to handle boolean values flexibly:
    // - `action = ArgAction::Set`: Allows explicit value setting via CLI argument.
    // - `num_args = 0..=1`: Makes the value optional, allowing the flag to be present without a value.
    // - `default_missing_value = "true"`: Sets the value to true if the flag is provided without a value (e.g., `--cors`).
    // - `require_equals = true`: Requires the usage of an equals sign for explicit values (e.g., `--cors=true` or `--cors=false`).
    // This combination enables three distinct usage patterns: omitting the flag (None),
    // using the flag alone (Some(true)), and setting it explicitly (Some(true/false)).
    #[arg(
        long,
        short,
        action = ArgAction::Set,
        num_args = 0..=1,
        default_missing_value = "true",
        require_equals = true,
        help = "Enable CORS headers")]
    cors: Option<bool>,

    #[arg(long, help = "Handle browser's preflight requests automatically")]
    preflight: Option<PreflightType>,

    #[arg(long, short, help = "Mocks response delay")]
    delay_ms: Option<u64>,

    #[arg(long, short, help = "Origin server where")]
    origin: Option<String>,

    #[arg(
        long,
        short,
        help = "Admin base URL. The entry point for all admin URLs. Disabled by default"
    )]
    admin_base_url: Option<String>,

    #[arg(long, short, help = "Request log level")]
    log_request: Option<VerbosityLevel>,

    #[arg(long, short, help = "Verbosity level")]
    verbosity: Option<VerbosityLevel>,

    #[arg(long, help = "Maximum response size in bytes")]
    proxy_body_max_bytes: Option<usize>,

    #[clap(skip)]
    global_config: OnceCell<GlobalConfigAPI>,
}

impl ServerParams {
    fn check_admin_base_url(&self) -> Result<(), Box<dyn StdError + Sync + Send>> {
        let path = &self.admin_base_url.as_ref().map(PathBuf::from);
        let Some(path) = path else {
            return Ok(());
        };
        if path.is_absolute() || path.starts_with("/") {
            return Ok(());
        }
        let message = format!(
            "Admin base URL should be absolute path: \"{}\"",
            path.to_string_lossy().to_string()
        );
        Err(message.into())
    }

    async fn expect_mocks_path_to_exist(&self) -> Result<(), Box<dyn StdError + Sync + Send>> {
        let Some(ref path) = self.get_absolute_mocks_path().await else {
            return Err("Mocks dir not set".into());
        };
        let metadata_result = tokio::fs::metadata(path).await;

        if let Ok(ref metadata) = metadata_result
            && metadata.is_dir()
        {
            return Ok(());
        }

        let wrong_target = metadata_result
            .map(|m| m.is_file() || m.is_symlink())
            .unwrap_or(false);
        if wrong_target {
            let err_msg = format!(
                "The specified path '{}' is not a directory",
                &path.display()
            );
            return Err(err_msg.into());
        }

        if path.is_absolute() {
            let absolute_path = path.absolutize()?;
            tokio::fs::create_dir_all(absolute_path).await?;
            return Ok(());
        }

        let cwd = std::env::current_dir()?;
        let absolute_path: PathBuf = cwd.join(&path).absolutize()?.into();
        tokio::fs::create_dir_all(absolute_path).await?;

        Ok(())
    }

    async fn get_host(&self) -> String {
        match self.host {
            Some(ref host) => Some(host.clone()),
            None => self.get_global_config().await.get_host().await,
        }
        .unwrap_or_else(|| DEFAULT_HOST.to_string())
    }

    async fn get_port(&self) -> u16 {
        match self.port {
            Some(port) => Some(port),
            None => self.get_global_config().await.get_port().await,
        }
        .unwrap_or(DEFAULT_PORT)
    }

    pub async fn admin_base_url(&self) -> Option<String> {
        match self.admin_base_url {
            Some(ref base_url) => Some(base_url.clone()),
            None => self.get_global_config().await.get_admin_base_url().await,
        }
    }

    pub async fn cors(&self) -> bool {
        match self.cors {
            Some(cors) => Some(cors),
            None => self.get_global_config().await.get_cors().await,
        }
        .unwrap_or(DEFAULT_CORS_ENABLED)
    }

    pub async fn preflight(&self) -> Option<PreflightType> {
        match self.preflight {
            Some(preflight) => Some(preflight),
            None => self.get_global_config().await.get_preflight().await,
        }
    }

    pub async fn delay_ms(&self) -> u64 {
        match self.delay_ms {
            Some(delay) => Some(delay),
            None => self.get_global_config().await.get_delay_ms().await,
        }
        .unwrap_or(DEFAULT_DELAY_MS)
    }

    pub async fn origin(&self) -> Option<String> {
        match self.origin {
            Some(ref origin) => Some(origin.clone()),
            None => self.get_global_config().await.get_origin().await,
        }
    }

    pub async fn log_request(&self) -> VerbosityLevel {
        match self.log_request {
            Some(log_request) => Some(log_request),
            None => self.get_global_config().await.get_log_request().await,
        }
        .unwrap_or(VerbosityLevel::Info)
    }

    pub async fn verbosity_level(&self) -> VerbosityLevel {
        match self.verbosity {
            Some(verbosity) => Some(verbosity),
            None => self.get_global_config().await.get_verbosity_level().await,
        }
        .unwrap_or(VerbosityLevel::Info)
    }

    /// Returns the maximum response size in bytes for proxy requests.
    ///
    /// This method determines the value by checking the following sources in order:
    /// 1. `self.proxy_body_max_bytes`: The value passed via the command line argument `--proxy-body-max-bytes`.
    /// 2. `get_proxy_body_max_bytes()`: The value defined in the global configuration file.
    /// 3. `DEFAULT_PROXY_RESPONSE_BODY_BYTES`: The default constant if neither source provides a value.
    ///
    /// The final value is capped by `HARD_MAX_PROXY_RESPONSE_BODY_BYTES` to ensure it does not exceed the hard limit.
    pub async fn proxy_body_max_bytes(&self) -> usize {
        let proxy_body_max_bytes = match self.proxy_body_max_bytes {
            Some(proxy_body_max_bytes) => Some(proxy_body_max_bytes),
            None => {
                self.get_global_config()
                    .await
                    .get_proxy_body_max_bytes()
                    .await
            }
        }
        .unwrap_or(DEFAULT_PROXY_RESPONSE_BODY_BYTES);

        std::cmp::min(proxy_body_max_bytes, HARD_MAX_PROXY_RESPONSE_BODY_BYTES)
    }

    pub async fn test(&self) -> Result<(), Box<dyn StdError + Sync + Send>> {
        if let Err(e) = self.expect_mocks_path_to_exist().await {
            return Err(e);
        }

        if let Err(e) = self.check_admin_base_url() {
            return Err(e);
        }

        Ok(())
    }

    async fn listener(&self) -> Result<TcpListener, IoError> {
        let socket_addr = format!("{}:{}", self.get_host().await, self.get_port().await)
            .to_socket_addrs()?
            .next();
        if let Some(socket_addr) = socket_addr {
            return TcpListener::bind(socket_addr).await;
        }

        let fallback_addr = format!("{}:{}", DEFAULT_HOST, DEFAULT_PORT)
            .to_socket_addrs()?
            .next();
        if let Some(fallback_addr) = fallback_addr {
            return TcpListener::bind(fallback_addr).await;
        }

        Err(IoError::new(
            ErrorKind::AddrNotAvailable,
            "Incorrect default host and/or port",
        ))
    }

    pub async fn start_server(&self) -> Result<(), Box<dyn StdError + Send + Sync>> {
        let http = Arc::new(http1::Builder::new());
        let graceful = Arc::new(hyper_util::server::graceful::GracefulShutdown::new());
        let listener = self.listener().await?;
        let mut shutdown_signal = make_signal();
        let router = mockers_router(&self).await?;
        let router_service = Arc::new(RouterService::new(router)?);

        let tls_acceptor = match self.get_absolute_mocks_path().await {
            Some(amp) => TlsAcceptor::load(amp).await.map(Arc::new),
            None => None,
        };

        println!("{}", BANNER_MSG);
        match self.verbosity_level().await {
            VerbosityLevel::Debug | VerbosityLevel::Trace => {
                info!("{}", self.get_help("Start parameters").await)
            }
            VerbosityLevel::Info => {}
        };
        info!(
            "Server has started at {}:{}",
            self.get_host().await,
            self.get_port().await
        );

        loop {
            tokio::select! {
                Ok((stream, _)) = listener.accept() => {
                    let router_service = Arc::clone(&router_service);
                    let graceful = graceful.clone();
                    let http = http.clone();
                    let tls_acceptor = tls_acceptor.clone();

                    if let Some(tls_acceptor) = tls_acceptor {
                        tokio::spawn(async move {
                            let stream = match tls_acceptor.accept(stream).await {
                                Ok(stream) => stream,
                                Err(e) => {
                                    error!("TLS handshake failed: {:?}", e);
                                    return;
                                }
                            };

                            let io = TokioIo::new(stream);
                            let tcp_stream = io.inner().get_ref().0;

                            match router_service.call(tcp_stream).await {
                                Ok(request_service) => {
                                    let conn = http.serve_connection(io, request_service);
                                    let fut = graceful.watch(conn);

                                    if let Err(e) = fut.await {
                                        error!("Error serving HTTPS connection: {:?}", e);
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to create HTTPS request service: {:?}", e);
                                }
                            }
                        });
                    } else {
                        tokio::spawn(async move {
                            match router_service.call(&stream).await {
                                Ok(request_service) => {
                                    let io = TokioIo::new(stream);
                                    let conn = http.serve_connection(io, request_service);
                                    let fut = graceful.watch(conn);

                                    if let Err(e) = fut.await {
                                        error!("Error serving HTTP connection: {:?}", e);
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to create HTTP request service: {:?}", e);
                                }
                            }
                        });
                    }
                },

                _ = &mut shutdown_signal => {
                    drop(listener);
                    error!("graceful shutdown signal received");
                    break;
                }
            }
        }
        Ok(())
    }
}

impl WithMocks for ServerParams {
    fn get_mocks(&self) -> Option<&str> {
        self.mocks.as_ref().map(|x| x.as_str())
    }
}

impl WithGlobalConfigAPI for ServerParams {
    async fn get_global_config(&self) -> &GlobalConfigAPI {
        self.global_config.get_or_init(GlobalConfigAPI::new).await
    }
}

impl GetInfoAsync for ServerParams {
    async fn get_help(&self, title: &str) -> String {
        let mut buf = String::from(format!("{}:{}", title, Self::EOL));
        buf.push_str(&format!("================={}", Self::EOL));
        let host_info = format!(
            "Host:{}{}{}",
            Self::TAB.repeat(3),
            self.get_host().await,
            Self::EOL
        );
        let port_info = format!(
            "Port:{}{}{}",
            Self::TAB.repeat(3),
            self.get_port().await,
            Self::EOL
        );
        let mocks_path_info = format!(
            "Mocks path:{}{}{}",
            Self::TAB.repeat(2),
            self.get_absolute_mocks_path()
                .await
                .map(|v| v.to_string_lossy().to_string())
                .unwrap_or_else(|| Self::UNSET.to_string()),
            Self::EOL,
        );
        let cors_info = format!(
            "Cors:{}{}{}",
            Self::TAB.repeat(3),
            self.cors().await,
            Self::EOL
        );
        let preflight_info = format!(
            "Preflight:{}{}{}",
            Self::TAB.repeat(2),
            self.preflight()
                .await
                .map(|pt| pt.to_string())
                .unwrap_or_else(|| Self::UNSET.to_string()),
            Self::EOL
        );
        let delay_ms_info = format!(
            "Delay in milliseconds:{}{}{}",
            Self::TAB,
            self.delay_ms().await,
            Self::EOL
        );
        let origin_info = format!(
            "Origin:{}{}{}",
            Self::TAB.repeat(3),
            self.origin()
                .await
                .unwrap_or_else(|| Self::UNSET.to_string()),
            Self::EOL,
        );
        let admin_base_url_info = format!(
            "Admin base URL:{}{}{}",
            Self::TAB.repeat(2),
            self.admin_base_url()
                .await
                .unwrap_or_else(|| Self::UNSET.to_string()),
            Self::EOL,
        );
        let log_request_info = format!(
            "Requests log level:{}{}{}",
            Self::TAB,
            self.log_request().await,
            Self::EOL,
        );
        let verbosity_level_info = format!(
            "Verbosity level:{}{}{}",
            Self::TAB,
            self.verbosity_level().await,
            Self::EOL
        );

        buf.push_str(&host_info);
        buf.push_str(&port_info);
        buf.push_str(&mocks_path_info);
        buf.push_str(&cors_info);
        buf.push_str(&preflight_info);
        buf.push_str(&delay_ms_info);
        buf.push_str(&origin_info);
        buf.push_str(&admin_base_url_info);
        buf.push_str(&log_request_info);
        buf.push_str(&verbosity_level_info);

        buf
    }
}
