use std::convert::Infallible;
use std::sync::Arc;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response, StatusCode};
use tokio::fs;

use super::get_mime::get_mime;
use super::params::ServerParams;

pub async fn hello(
    req: Request<hyper::body::Incoming>,
    params: Arc<ServerParams>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    if params.verbose {
        println!("Serving {}", &req.uri());
    }
    let method = req.method().to_string().to_lowercase();
    let uri_pathname = req.uri().path().to_string();

    let mocks_dir = params.get_absolute_mocks_path();
    if mocks_dir.is_err() {
        return Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Full::new(Bytes::from("")))
            .unwrap());
    }
    let mocks_dir = mocks_dir.unwrap();
    let target_file_path = mocks_dir.join(format!("{}.{}", uri_pathname[1..].to_string(), method));

    if params.verbose {
        println!("Mocks dir: {}", &mocks_dir.display().to_string());
        println!(
            "Requested file: {}",
            &target_file_path.display().to_string()
        );
    }

    return match fs::read(&target_file_path).await {
        Ok(data) => {
            let mime = get_mime(&data);

            if params.verbose {
                println!("File mime: {}", &mime)
            }

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", mime)
                .body(Full::from(data))
                .unwrap())
        }
        Err(_) => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Full::new(Bytes::from("")))
            .unwrap()),
    };
}
