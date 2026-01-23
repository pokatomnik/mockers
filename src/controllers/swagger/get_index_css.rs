use crate::controllers::swagger::static_files::SWAGGER_INDEX_CSS;
use crate::server::route_error::MockersRouteError;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::CONTENT_TYPE;
use hyper::{Request, Response};
use reqwest::StatusCode;

pub async fn get_index_css(_: Request<Full<Bytes>>) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/css")
        .body(Full::from(SWAGGER_INDEX_CSS))
        .unwrap_or(Response::default()))
}
