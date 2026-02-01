use http_body_util::Full;
use hyper::{Response, body::Bytes};

use crate::{
    controllers::utils::{CORS_HEADER_KEYS, CORS_HEADER_VALUE},
    server::route_error::MockersRouteError,
};

pub(crate) async fn admin_api_cors(
    mut response: Response<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    let headers = response.headers_mut();
    for header in CORS_HEADER_KEYS.iter() {
        if let Ok(v) = CORS_HEADER_VALUE.parse() {
            headers.insert(*header, v);
        }
    }
    return Ok(response);
}
