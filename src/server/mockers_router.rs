use std::sync::Arc;

use http_body_util::Full;
use hyper::{Response, StatusCode, body::Bytes};

use crate::{libs::router::Router, server::params::ServerParams};

pub fn mockers_router(params: &ServerParams) -> Router {
    return Router::new(params.clone()).route(
        "/health".to_string(),
        Arc::new(|_, params| {
            Box::pin(async move {
                let mut builder = Response::builder().status(StatusCode::OK);

                if params.cors {
                    builder = builder
                        .header("Access-Control-Allow-Origin", "*")
                        .header("Access-Control-Allow-Methods", "*");
                }
                let response = builder.body(Full::new(Bytes::from("OK"))).unwrap();
                Ok(response)
            })
        }),
    );
}
