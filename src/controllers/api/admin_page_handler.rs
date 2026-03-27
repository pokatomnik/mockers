use crate::libs::hyper_response_ext::HyperWellKnownResponses;
use crate::libs::response_builder_ext::ResponseBuilderExt;
use crate::server::route_error::MockersRouteError;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};

pub async fn admin_page_handler(
    _req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    let response = Response::ok()
        .content_type_text_plain()
        .body("Hello world!".into())
        .unwrap_or_default();
    Ok(response)
}
