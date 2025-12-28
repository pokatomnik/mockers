use crate::controllers::swagger::static_files::SWAGGER_FAVICON_32_PNG;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};
use mimetype_detector::IMAGE_PNG;
use reqwest::StatusCode;
use reqwest::header::CONTENT_TYPE;
use std::convert::Infallible;

pub async fn get_favicon_32(_: Request<Full<Bytes>>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, IMAGE_PNG)
        .body(Full::from(SWAGGER_FAVICON_32_PNG))
        .unwrap_or(Response::default()))
}
