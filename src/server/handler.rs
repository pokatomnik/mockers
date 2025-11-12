use std::convert::Infallible;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};

use super::params::ServerParams;

pub async fn hello(
    req: Request<hyper::body::Incoming>,
    params: &ServerParams,
) -> Result<Response<Full<Bytes>>, Infallible> {
    if params.verbose {
        println!("Serving {}", req.uri());
    }
    Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}
