use http_body_util::Full;
use hyper::Request;

use crate::{libs::http_method::StandardMethodValidator, server::route_error::MockersRouteError};

pub async fn check_request(
    req: Request<Full<hyper::body::Bytes>>,
) -> Result<Request<Full<hyper::body::Bytes>>, MockersRouteError> {
    req.method()
        .validate(|| MockersRouteError::IncorrectHTTPMethod)
        .map(|_| req)
}
