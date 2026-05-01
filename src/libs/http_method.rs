use hyper::Method;

pub(crate) trait HyperHTTPMethodExt {
    fn get_all_methods() -> Vec<hyper::Method>;
}

pub(crate) trait StandardMethodValidator<T> {
    fn validate(&self, err: impl FnOnce() -> T) -> Result<(), T>;
}

impl HyperHTTPMethodExt for Method {
    fn get_all_methods() -> Vec<hyper::Method> {
        return vec![
            Method::CONNECT,
            Method::DELETE,
            Method::GET,
            Method::HEAD,
            Method::OPTIONS,
            Method::PATCH,
            Method::POST,
            Method::PUT,
            Method::TRACE,
        ];
    }
}

impl<T> StandardMethodValidator<T> for Method {
    fn validate(&self, err: impl FnOnce() -> T) -> Result<(), T> {
        match self {
            &Method::CONNECT
            | &Method::DELETE
            | &Method::GET
            | &Method::HEAD
            | &Method::OPTIONS
            | &Method::PATCH
            | &Method::POST
            | &Method::PUT
            | &Method::TRACE => Ok(()),
            _ => Err(err()),
        }
    }
}
