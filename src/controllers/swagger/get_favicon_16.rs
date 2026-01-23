use crate::controllers::swagger::static_files::SWAGGER_FAVICON_16_PNG;
use crate::server::route_error::MockersRouteError;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::CONTENT_TYPE;
use hyper::{Request, Response};
use mimetype_detector::IMAGE_PNG;
use reqwest::StatusCode;

pub async fn get_favicon_16(_: Request<Full<Bytes>>) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, IMAGE_PNG)
        .body(Full::from(SWAGGER_FAVICON_16_PNG))
        .unwrap_or(Response::default()))
}
