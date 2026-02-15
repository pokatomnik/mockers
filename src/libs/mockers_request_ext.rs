use crate::libs::preflight_type::PreflightType;
use hyper::{Method, Request};
use std::collections::HashMap;

trait PreflightRequestHeaders {
    fn get_header(&self, key: &str) -> Option<&str>;
    fn origin(&self) -> Option<&str>;
    fn access_control_request_method(&self) -> Option<&str>;
    fn access_control_request_headers(&self) -> Option<&str>;
    fn access_control_request_private_network(&self) -> Option<&str>;
}

impl<T> PreflightRequestHeaders for Request<T> {
    fn get_header(&self, key: &str) -> Option<&str> {
        self.headers().get(key).map(|h| h.to_str().ok()).flatten()
    }
    fn origin(&self) -> Option<&str> {
        self.get_header("Origin")
    }

    fn access_control_request_method(&self) -> Option<&str> {
        self.get_header("Access-Control-Request-Method")
    }

    fn access_control_request_headers(&self) -> Option<&str> {
        self.get_header("Access-Control-Request-Headers")
    }

    fn access_control_request_private_network(&self) -> Option<&str> {
        self.get_header("Access-Control-Request-Private-Network")
    }
}

pub(crate) trait MockersRequestExt<T> {
    /// Check if the request is a "preflight" browser request
    fn is_preflight(&self) -> bool;

    /// Convert preflight request headers to response headers for browser
    fn preflight_response_headers(&self, preflight: PreflightType) -> HashMap<String, String>;
}

static ORIGIN_HEADER: &'static str = "Origin";
static ACCESS_CONTROL_REQUEST_METHOD_HEADER: &'static str = "Access-Control-Request-Method";

impl<T> MockersRequestExt<T> for Request<T> {
    fn is_preflight(&self) -> bool {
        let method = self.method();
        let headers = self.headers();
        let origin_header_exists = headers.contains_key(ORIGIN_HEADER);
        let access_control_request_header_exists =
            headers.contains_key(ACCESS_CONTROL_REQUEST_METHOD_HEADER);

        method == Method::OPTIONS && origin_header_exists && access_control_request_header_exists
    }
    fn preflight_response_headers(&self, preflight: PreflightType) -> HashMap<String, String> {
        let mut result = HashMap::new();

        result.insert("Vary".to_string(), "Origin".to_string());

        let access_control_allow_origin_header = "Access-Control-Allow-Origin".to_string();
        match (preflight, self.origin()) {
            (PreflightType::Mirror, Some(origin)) => {
                result.insert(access_control_allow_origin_header, origin.to_string());
            }
            (PreflightType::Permissive, _) => {
                result.insert(access_control_allow_origin_header, "*".to_string());
            }
            _ => {}
        };

        let access_control_allow_methods_header = "Access-Control-Allow-Methods".to_string();
        match (preflight, self.access_control_request_method()) {
            (PreflightType::Mirror, Some(method)) => {
                result.insert(
                    access_control_allow_methods_header.to_string(),
                    method.to_string(),
                );
            }
            (PreflightType::Permissive, _) => {
                result.insert(
                    access_control_allow_methods_header.to_string(),
                    "GET,HEAD,POST,PUT,DELETE,CONNECT,OPTIONS,TRACE,PATCH".to_string(),
                );
            }
            _ => {}
        }

        let access_control_allow_headers_header = "Access-Control-Allow-Headers".to_string();
        match (preflight, self.access_control_request_headers()) {
            (PreflightType::Mirror, Some(headers)) => {
                result.insert(access_control_allow_headers_header, headers.to_string());
            }
            (PreflightType::Permissive, _) => {
                result.insert(access_control_allow_headers_header, "*".to_string());
            }
            _ => {}
        }

        let access_control_request_private_network_header =
            "Access-Control-Allow-Private-Network".to_string();
        match (preflight, self.access_control_request_private_network()) {
            (PreflightType::Mirror, Some(allow)) => {
                result.insert(
                    access_control_request_private_network_header,
                    allow.to_string(),
                );
            }
            (PreflightType::Permissive, _) => {
                result.insert(
                    access_control_request_private_network_header,
                    "true".to_string(),
                );
            }
            _ => {}
        }

        result
    }
}
