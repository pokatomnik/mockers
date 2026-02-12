use crate::controllers::swagger::static_files::SWAGGER_INITIALIZER_JS;
use crate::libs::response_builder_ext::ResponseBuilderExt;
use crate::libs::response_ext::WellKnownResponses;
use crate::server::route_error::MockersRouteError;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};

pub async fn get_swagger_initializer_js(
    _: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    let response = Response::ok()
        .content_type_js()
        .body(SWAGGER_INITIALIZER_JS.into())
        .unwrap_or_default();

    Ok(response)
}
