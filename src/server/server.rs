use crate::server::mockers_router::mockers_router;
use crate::server::signal::make_signal;

use super::params::ServerParams;

use super::listener::listener;
use std::sync::Arc;

use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;

pub async fn start_server(
    params: ServerParams,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let http = http1::Builder::new();
    let graceful = hyper_util::server::graceful::GracefulShutdown::new();
    let listener = listener(&params).await?;
    let mut shutdown_signal = make_signal();

    println!("Server has started at {}:{}", params.host, params.port);

    let router = Arc::new(mockers_router(&params));

    loop {
        tokio::select! {
            Ok((stream, _addr)) = listener.accept() => {
                let io = TokioIo::new(stream);
                let router = router.clone();
                let conn = http.serve_connection(io, service_fn(move |req| {
                    router.clone().handle(req)
                }));
                let fut = graceful.watch(conn);
                tokio::spawn(async move {
                    if let Err(e) = fut.await {
                        eprintln!("Error serving connection: {:?}", e);
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
