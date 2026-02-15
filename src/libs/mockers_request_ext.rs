use hyper::{Method, Request};

pub(crate) trait RequestExt<T> {
    fn is_preflight(&self) -> bool;
}

static ORIGIN_HEADER: &'static str = "Origin";
static ACCESS_CONTROL_REQUEST_METHOD_HEADER: &'static str = "Access-Control-Request-Method";

impl<T> RequestExt<T> for Request<T> {
    fn is_preflight(&self) -> bool {
        let method = self.method();
        let headers = self.headers();
        let origin_header_exists = headers.contains_key(ORIGIN_HEADER);
        let access_control_request_header_exists =
            headers.contains_key(ACCESS_CONTROL_REQUEST_METHOD_HEADER);

        method == Method::OPTIONS && origin_header_exists && access_control_request_header_exists
    }
}
