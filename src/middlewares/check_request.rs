use http_body_util::Full;
use hyper::{Method, Request};

use crate::server::route_error::MockersRouteError;

pub async fn check_request(
    req: Request<Full<hyper::body::Bytes>>,
) -> Result<Request<Full<hyper::body::Bytes>>, MockersRouteError> {
    match req.method() {
        &Method::CONNECT
        | &Method::DELETE
        | &Method::GET
        | &Method::HEAD
        | &Method::OPTIONS
        | &Method::PATCH
        | &Method::POST
        | &Method::PUT
        | &Method::TRACE => Ok(req),
        _ => Err(MockersRouteError::IncorrectHTTPMethod),
    }
}
