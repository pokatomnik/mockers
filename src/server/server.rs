use super::params::ServerParams;

use super::params::{DEFAULT_HOST, DEFAULT_PORT};
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;

use super::handler::hello;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

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
    println!("Server has started at {}:{}", params.host, params.port);

    let params = Arc::new(params.clone());

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let params = params.clone();

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(|req| hello(req, &params)))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
