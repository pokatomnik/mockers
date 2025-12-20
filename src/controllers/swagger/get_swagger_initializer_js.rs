use crate::controllers::swagger::static_files::SWAGGER_INITIALIZER_JS;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};
use reqwest::StatusCode;
use std::convert::Infallible;

pub async fn get_swagger_initializer_js(_: Request<Full<Bytes>>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/javascript")
        .body(Full::from(SWAGGER_INITIALIZER_JS)).unwrap())
}
