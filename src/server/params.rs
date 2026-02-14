use crate::libs::absolute_mocks_path::{generic_get_absolute_mocks_path, AbsoluteMocksPath};
use crate::server::mockers_router::mockers_router;
use crate::server::signal::make_signal;
use clap::Args;
use hyper::server::conn::http1;
use hyper::service::Service;
use hyper_util::rt::TokioIo;
use path_absolutize::Absolutize;
use routerify_ng::RouterService;
use std::error::Error as StdError;
use std::io::Error as IoError;
use std::io::ErrorKind;
use std::net::ToSocketAddrs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::net::TcpListener;

pub const DEFAULT_HOST: &'static str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 8080;
pub const DEFAULT_MOCKS_DIR_NAME: &'static str = "mocks";
pub const DEFAULT_MOCKS_RESPONSE_DELAY: u64 = 0;
pub const DEFAULT_CORS_ENABLED: bool = false;
pub const CONFIG_FILE_NAME: &'static str = "config.json";

#[derive(Args, Debug, Clone)]
pub struct ServerParams {
    #[arg(long, default_value = DEFAULT_HOST, help = "Host to listen on")]
    host: String,

    #[arg(long, short, default_value_t = DEFAULT_PORT, help = "Port to listen on")]
    port: u16,

    #[arg(long, short, default_value = DEFAULT_MOCKS_DIR_NAME, help = "Path to the directory containing mock files")]
    mocks: String,

    #[arg(long, short, default_value_t = false, help = "Enable CORS headers")]
    cors: bool,

    #[arg(long, short, default_value_t = 0, help = "Mocks response delay")]
    delay_ms: u64,

    #[arg(long, short, help = "Origin server where")]
    origin: Option<String>,

    #[arg(
        long,
        short,
        help = "Admin base URL. The entry point for all admin URLs. Disabled by default"
    )]
    admin_base_url: Option<String>,
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
        let path = Path::new(&self.mocks);
        let metadata_result = tokio::fs::metadata(path).await;

        if let Ok(ref metadata) = metadata_result
            && metadata.is_dir()
        {
            return Ok(());
        }

        if let Ok(ref metadata) = metadata_result
            && (metadata.is_file() || metadata.is_symlink())
        {
            let error = IoError::new(
                ErrorKind::NotADirectory,
                format!("The specified path '{}' is not a directory", &self.mocks),
            );
            return Err(error.into());
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

    pub fn admin_base_url(&self) -> Option<&str> {
        self.admin_base_url.as_deref()
    }
    
    pub fn cors(&self) -> bool {
        self.cors
    }
    
    pub fn delay_ms(&self) -> u64 {
        self.delay_ms
    }
    
    pub fn origin(&self) -> Option<&str> {
        self.origin.as_deref()
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
        let socket_addr = format!("{}:{}", self.host, self.port)
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
        let router = mockers_router(&self)?;
        let router_service = Arc::new(RouterService::new(router)?);

        println!("Server has started at {}:{}", self.host, self.port);

        loop {
            tokio::select! {
                Ok((stream, _)) = listener.accept() => {
                    let router_service = Arc::clone(&router_service);
                    let graceful = graceful.clone();
                    let http = http.clone();

                    tokio::spawn(async move {
                        match router_service.call(&stream).await {
                            Ok(request_service) => {
                                let io = TokioIo::new(stream);

                                let conn = http.serve_connection(io, request_service);
                                let fut = graceful.watch(conn);
                                if let Err(e) = fut.await {
                                    eprintln!("Error serving connection: {:?}", e);
                                }
                            }
                            Err(_) => {}
                        }
                    });
                },

                _ = &mut shutdown_signal => {
                    drop(listener);
                    eprintln!("graceful shutdown signal received");
                    break;
                }
            }
        }
        Ok(())
    }
}

impl AbsoluteMocksPath for ServerParams {
    fn get_absolute_mocks_path(&self) -> Result<PathBuf, Box<dyn StdError + Sync + Send>> {
        generic_get_absolute_mocks_path(&self.mocks, || std::env::current_dir().map_err(Box::from))
    }
}
