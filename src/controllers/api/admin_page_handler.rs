use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response, StatusCode, http};
use mimetype_detector::TEXT_HTML;

use crate::server::route_error::MockersRouteError;

pub async fn admin_page_handler(
    _req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    return Ok(Response::builder()
        .status(StatusCode::OK)
        // TODO send SPA HTML here
        .header(http::header::CONTENT_TYPE, TEXT_HTML)
        .body(Full::new(Bytes::from("Hello world!")))
        .unwrap_or(Response::default()));
}
