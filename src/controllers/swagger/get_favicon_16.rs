use crate::controllers::swagger::static_files::SWAGGER_FAVICON_16_PNG;
use crate::libs::response_builder_ext::ResponseBuilderExt;
use crate::libs::response_ext::WellKnownResponses;
use crate::server::route_error::MockersRouteError;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};

pub async fn get_favicon_16(
    _: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    let response = Response::ok()
        .content_type_png()
        .body(SWAGGER_FAVICON_16_PNG.into())
        .unwrap_or_default();
    Ok(response)
}
