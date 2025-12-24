use http_body_util::Full;
use hyper::{Response, StatusCode};
use routerify_ng::RequestInfo;

pub async fn error_handler(
    err: routerify_ng::RouteError,
    _: RequestInfo,
) -> Response<Full<hyper::body::Bytes>> {
    eprintln!("{}", err);
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Full::new(hyper::body::Bytes::from(format!(
            "Something went wrong: {}",
            err
        ))))
        .unwrap_or(Response::default())
}
