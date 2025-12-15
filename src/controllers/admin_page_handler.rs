use std::convert::Infallible;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response, StatusCode};

pub async fn admin_page_handler(_req: Request<Full<Bytes>>) -> Result<Response<Full<Bytes>>, Infallible> {
    return Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Full::new(Bytes::from("Hello world!")))
        .unwrap());
}