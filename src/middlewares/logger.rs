use std::convert::Infallible;

use http_body_util::Full;
use hyper::Request;
use routerify_ng::ext::RequestExt;

pub async fn logger(
    req: Request<Full<hyper::body::Bytes>>,
) -> Result<Request<Full<hyper::body::Bytes>>, Infallible> {
    println!(
        "{} {} {}",
        req.remote_addr(),
        req.method(),
        req.uri().path()
    );
    Ok(req)
}
