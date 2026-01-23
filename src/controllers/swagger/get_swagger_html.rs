use crate::controllers::swagger::static_files::SWAGGER_HTML;
use crate::server::route_error::MockersRouteError;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::CONTENT_TYPE;
use hyper::{Request, Response};
use mimetype_detector::TEXT_HTML;
use reqwest::StatusCode;

pub async fn get_swagger_html(
    _: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, TEXT_HTML)
        .body(Full::from(SWAGGER_HTML))
        .unwrap_or(Response::default()))
}
