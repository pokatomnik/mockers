use crate::server::mockers_router::mockers_router;
use crate::server::signal::make_signal;

use super::params::ServerParams;

use super::listener::listener;
use std::sync::Arc;

use hyper::server::conn::http1;
use hyper::service::Service;
use hyper_util::rt::TokioIo;
use routerify_ng::RouterService;

pub async fn start_server(
    params: ServerParams,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let http = Arc::new(http1::Builder::new());
    let graceful = Arc::new(hyper_util::server::graceful::GracefulShutdown::new());
    let listener = listener(&params).await?;
    let mut shutdown_signal = make_signal();
    let router = mockers_router(&params)?;
    let router_service = Arc::new(RouterService::new(router)?);

    println!("Server has started at {}:{}", params.host, params.port);

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
