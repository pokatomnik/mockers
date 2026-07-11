use http_body_util::Full;
use hyper::{Response, body::Bytes};

use crate::libs::header_map_ext::HeaderMapExt;
use crate::server::route_error::MockersRouteError;

pub(crate) async fn admin_api_cors(
    mut response: Response<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    response.headers_mut().add_cors();
    Ok(response)
}
