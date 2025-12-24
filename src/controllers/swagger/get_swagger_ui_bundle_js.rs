use crate::controllers::swagger::static_files::SWAGGER_UI_BUNDLE_JS;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};
use reqwest::StatusCode;
use std::convert::Infallible;
use hyper::header::CONTENT_TYPE;
use mimetype_detector::APPLICATION_JAVASCRIPT;

pub async fn get_swagger_ui_bundle_js(_: Request<Full<Bytes>>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, APPLICATION_JAVASCRIPT)
        .body(Full::from(SWAGGER_UI_BUNDLE_JS)).unwrap())
}
