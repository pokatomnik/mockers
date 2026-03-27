use http_body_util::Full;
use hyper::http::response::Builder;
use hyper::{Response, StatusCode, body::Bytes};

/// Well-known Hyper HTTP responses.
/// This trait relates to Hyper, not Reqwest.
pub(crate) trait HyperWellKnownResponses {
    fn not_found() -> Builder;

    fn internal_server_error() -> Builder;

    fn bad_gateway() -> Builder;

    fn bad_request() -> Builder;

    fn forbidden() -> Builder;

    fn ok() -> Builder;

    fn no_content() -> Builder;
}

impl HyperWellKnownResponses for Response<Full<Bytes>> {
    fn not_found() -> Builder {
        Response::builder().status(StatusCode::NOT_FOUND)
    }

    fn internal_server_error() -> Builder {
        Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn bad_gateway() -> Builder {
        Response::builder().status(StatusCode::BAD_GATEWAY)
    }

    fn bad_request() -> Builder {
        Response::builder().status(StatusCode::BAD_REQUEST)
    }

    fn forbidden() -> Builder {
        Response::builder().status(StatusCode::FORBIDDEN)
    }

    fn ok() -> Builder {
        Response::builder().status(StatusCode::OK)
    }

    fn no_content() -> Builder {
        Response::builder().status(StatusCode::NO_CONTENT)
    }
}
