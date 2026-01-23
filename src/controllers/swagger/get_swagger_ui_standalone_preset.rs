use crate::controllers::swagger::static_files::SWAGGER_UI_STANDALONE_PRESET_JS;
use crate::server::route_error::MockersRouteError;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::CONTENT_TYPE;
use hyper::{Request, Response};
use mimetype_detector::APPLICATION_JAVASCRIPT;
use reqwest::StatusCode;

pub async fn get_swagger_ui_standalone_preset(
    _: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, APPLICATION_JAVASCRIPT)
        .body(Full::from(SWAGGER_UI_STANDALONE_PRESET_JS))
        .unwrap_or(Response::default()))
}
