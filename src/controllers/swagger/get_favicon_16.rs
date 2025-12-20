use crate::controllers::swagger::static_files::SWAGGER_FAVICON_16_PNG;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};
use reqwest::StatusCode;
use std::convert::Infallible;

pub async fn get_favicon_16(_: Request<Full<Bytes>>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "image/png")
        .body(Full::from(SWAGGER_FAVICON_16_PNG)).unwrap())
}
