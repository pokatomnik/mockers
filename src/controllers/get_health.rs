use std::{convert::Infallible, sync::Arc};

use http_body_util::Full;
use hyper::{Request, Response, StatusCode, body::Bytes};
use routerify_ng::ext::RequestExt;

use crate::server::params::ServerParams;

pub async fn health_handler(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let cors = req
        .data::<Arc<ServerParams>>()
        .map(|params| params.cors)
        .unwrap_or(false);

    let mut builder = Response::builder().status(StatusCode::OK);
    if cors {
        builder = builder
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "*");
    }
    let response = builder.body(Full::new(Bytes::from("OK"))).unwrap();

    Ok(response)
}
