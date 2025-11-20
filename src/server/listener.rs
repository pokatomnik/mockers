use std::net::{SocketAddr, ToSocketAddrs};
use tokio::net::TcpListener;

use super::params::{DEFAULT_HOST, DEFAULT_PORT};

use crate::server::params::ServerParams;

pub async fn listener(params: &ServerParams) -> Result<TcpListener, std::io::Error> {
    let socket_addr = format!("{}:{}", params.host, params.port)
        .to_socket_addrs()?
        .next()
        .unwrap_or_else(get_default_socket_addr);

    return TcpListener::bind(socket_addr).await;
}

fn get_default_socket_addr() -> SocketAddr {
    let as_str = format!("{}:{}", DEFAULT_HOST, DEFAULT_PORT);
    return as_str.to_socket_addrs().unwrap().next().unwrap();
}
