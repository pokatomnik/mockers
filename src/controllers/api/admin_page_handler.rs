use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{http, Request, Response, StatusCode};
use std::convert::Infallible;

pub async fn admin_page_handler(
    _req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    return Ok(Response::builder()
        .status(StatusCode::OK)
        // TODO send SPA HTML here
        .header(http::header::CONTENT_TYPE, "text/html")
        .body(Full::new(Bytes::from("Hello world!")))
        .unwrap());
}
