use http_body_util::Full;
use hyper::http::response::Builder;
use hyper::{body::Bytes, Response, StatusCode};

pub(crate) trait WellKnownResponses {
    fn not_found() -> Builder;

    fn internal_server_error() -> Builder;

    fn bad_gateway() -> Builder;

    fn bad_request() -> Builder;

    fn ok() -> Builder;
}

impl WellKnownResponses for Response<Full<Bytes>> {
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

    fn ok() -> Builder {
        Response::builder().status(StatusCode::OK)
    }
}
