use crate::libs::response_ext::WellKnownResponses;
use crate::server::route_error::MockersRouteError;
use http_body_util::Full;
use hyper::{body::Bytes, Response, StatusCode};
use routerify_ng::RequestInfo;

pub async fn error_handler(err: routerify_ng::RouteError, _: RequestInfo) -> Response<Full<Bytes>> {
    let err_message = &err.to_string();
    eprintln!("Error while processing user request: {}", err);
    if let Some(mockers_route_error) = try_unwrap_err(err) {
        return match mockers_route_error {
            MockersRouteError::IncorrectHTTPMethod => Response::builder()
                .status(StatusCode::IM_A_TEAPOT)
                .body("Incorrect HTTP Method".into())
                .unwrap_or_default(),
        };
    }

    Response::internal_server_error()
        .body(format!("Something went wrong: {}", err_message).into())
        .unwrap_or_default()
}

fn try_unwrap_err(err: Box<dyn std::error::Error>) -> Option<MockersRouteError> {
    let mut cur: &dyn std::error::Error = &*err;

    if let Some(route_err) = cur.downcast_ref::<MockersRouteError>() {
        return Some(route_err.to_owned());
    }

    while let Some(src) = cur.source() {
        if let Some(route_err) = src.downcast_ref::<MockersRouteError>() {
            return Some(route_err.to_owned());
        }
        cur = src;
    }

    None
}
