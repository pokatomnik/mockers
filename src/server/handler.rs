use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response, StatusCode};
use tokio::fs;

use super::get_mime::get_mime;
use super::params::ServerParams;
use super::query_params::QueryParams;

pub async fn mock_handler(
    req: Request<hyper::body::Incoming>,
    params: Arc<ServerParams>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let delay = req
        .uri()
        .query()
        .and_then(|q| q.parse::<QueryParams>().ok())
        .and_then(|qp| qp.get("delay")?.get(0)?.parse().ok())
        .unwrap_or(0);
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
                println!("File mime: {}", &mime);
            }

            let mut builder = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", mime);
            if params.cors {
                builder = builder
                    .header("Access-Control-Allow-Origin", "*")
                    .header("Access-Control-Allow-Methods", "*");
            }
            let response = builder.body(Full::from(data)).unwrap();

            tokio::time::sleep(Duration::from_millis(delay)).await;

            Ok(response)
        }
        Err(_) => {
            let mut builder = Response::builder().status(StatusCode::NOT_FOUND);

            if params.cors {
                builder = builder
                    .header("Access-Control-Allow-Origin", "*")
                    .header("Access-Control-Allow-Methods", "*");
            }
            let response = builder.body(Full::new(Bytes::from(""))).unwrap();
            Ok(response)
        }
    };
}
