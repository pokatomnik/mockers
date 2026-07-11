use crate::libs::hyper_response_ext::HyperWellKnownResponses;
use crate::libs::response_builder_ext::ResponseBuilderExt;
use crate::server::controllers::swagger::static_files::SWAGGER_INDEX_CSS;
use crate::server::route_error::MockersRouteError;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};

pub async fn get_index_css(
    _: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    let response = Response::ok()
        .content_type_css()
        .body(SWAGGER_INDEX_CSS.into())
        .unwrap_or_default();
    Ok(response)
}
