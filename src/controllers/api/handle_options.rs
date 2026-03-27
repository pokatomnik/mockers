use crate::libs::hyper_response_ext::HyperWellKnownResponses;
use crate::libs::response_builder_ext::ResponseBuilderExt;
use crate::server::route_error::MockersRouteError;
use http_body_util::Full;
use hyper::{Request, Response, body::Bytes};

pub(crate) async fn handle_options(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    if req.method().to_string().to_lowercase() != "options" {
        return Ok(Response::not_found().empty_body().unwrap_or_default());
    }

    Ok(Response::ok().empty_body().unwrap_or_default())
}
