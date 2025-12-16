use crate::server::mockers_context::MockersContext;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response, StatusCode};
use routerify_ng::ext::RequestExt;
use std::convert::Infallible;
use std::sync::Arc;

pub async fn admin_page_handler(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    return Ok(Response::builder()
        .status(StatusCode::OK)
        // TODO send SPA HTML here
        .body(Full::new(Bytes::from("Hello world!")))
        .unwrap());
}
