use super::params::ServerParams;

use super::params::{DEFAULT_HOST, DEFAULT_PORT};
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;

use super::handler::mock_handler;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

fn get_default_socket_addr() -> SocketAddr {
    let as_str = format!("{}:{}", DEFAULT_HOST, DEFAULT_PORT);
    return as_str.to_socket_addrs().unwrap().next().unwrap();
}

pub async fn start_server(
    params: &ServerParams,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let socket_addr = format!("{}:{}", params.host, params.port)
        .to_socket_addrs()?
        .next()
        .unwrap_or_else(get_default_socket_addr);

    let listener = TcpListener::bind(socket_addr).await?;
    let http = http1::Builder::new();
    let graceful = hyper_util::server::graceful::GracefulShutdown::new();
    let mut signal = std::pin::pin!(shutdown_signal());

    println!("Server has started at {}:{}", params.host, params.port);

    let params = Arc::new(params.clone());

    loop {
        tokio::select! {
            Ok((stream, _addr)) = listener.accept() => {
                let io = TokioIo::new(stream);
                let params = params.clone();
                let conn = http.serve_connection(io, service_fn(move |req| mock_handler(req, params.clone())));
                let fut = graceful.watch(conn);
                tokio::spawn(async move {
                    if let Err(e) = fut.await {
                        eprintln!("Error serving connection: {:?}", e);
                    }
                });
            },

            _ = &mut signal => {
                drop(listener);
                eprintln!("graceful shutdown signal received");
                break;
            }
        }
    }
    Ok(())
}
