use crate::libs::response_builder_ext::ResponseBuilderExt;
use crate::libs::response_ext::WellKnownResponses;
use crate::server::route_error::MockersRouteError;
use http_body_util::Full;
use hyper::{body::Bytes, Request, Response};

pub(crate) async fn handle_options(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    if req.method().to_string().to_lowercase() != "options" {
        return Ok(Response::not_found().empty_body().unwrap_or_default());
    }

    Ok(Response::ok().empty_body().unwrap_or_default())
}
