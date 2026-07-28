use crate::libs::hyper_response_ext::HyperWellKnownResponses;
use crate::server::controllers::swagger::static_files::SWAGGER_FAVICON_32_PNG;
use crate::server::route_error::MockersRouteError;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};
use mimetype_detector::IMAGE_PNG;
use reqwest::header::CONTENT_TYPE;

pub async fn get_favicon_32(
    _: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    let response = Response::ok()
        .header(CONTENT_TYPE, IMAGE_PNG)
        .body(SWAGGER_FAVICON_32_PNG.into())
        .unwrap_or_default();
    Ok(response)
}
