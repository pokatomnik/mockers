use std::io::ErrorKind;
use std::net::ToSocketAddrs;
use tokio::net::TcpListener;

use super::params::{DEFAULT_HOST, DEFAULT_PORT};

use crate::server::params::ServerParams;

pub async fn listener(params: &ServerParams) -> Result<TcpListener, std::io::Error> {
    let socket_addr = format!("{}:{}", params.host, params.port)
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

    Err(std::io::Error::new(
        ErrorKind::AddrNotAvailable,
        "Incorrect default host and/or port",
    ))
}
