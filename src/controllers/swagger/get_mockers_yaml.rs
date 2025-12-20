use crate::controllers::swagger::static_files::SWAGGER_YAML;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};
use reqwest::StatusCode;
use std::convert::Infallible;

pub async fn get_mockers_yaml(
    _: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/yaml")
        .body(Full::from(SWAGGER_YAML))
        .unwrap())
}
