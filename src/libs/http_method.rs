use hyper::Method;

pub(crate) trait StandardMethodValidator<T> {
    fn validate(&self, err: impl FnOnce() -> T) -> Result<(), T>;
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
