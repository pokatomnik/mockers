use http_body_util::Full;
use hyper::{Request, Response, StatusCode, body::Bytes};

use crate::server::route_error::MockersRouteError;

pub(crate) async fn handle_options(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    if req.method().to_string().to_lowercase() != "options" {
        let not_found_response = Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Bytes::new().into())
            .unwrap_or_default();
        return Ok(not_found_response);
    }

    let options_response = Response::builder()
        .status(StatusCode::OK)
        .body(Bytes::new().into())
        .unwrap_or_default();

    Ok(options_response)
}
