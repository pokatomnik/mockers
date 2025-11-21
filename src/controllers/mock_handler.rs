use std::{convert::Infallible, sync::Arc, time::Duration};

use http_body_util::Full;
use hyper::{Request, Response, StatusCode, body::Bytes};
use routerify_ng::ext::RequestExt;
use tokio::{fs, time::sleep};

use crate::{
    libs::get_mime::get_mime,
    server::params::{
        DEFAULT_CORS_ENABLED, DEFAULT_MOCKS_RESPONSE_DELAY, DEFAULT_VERBOSE_ENABLED, ServerParams,
    },
};

pub async fn mock_handler(req: Request<Full<Bytes>>) -> Result<Response<Full<Bytes>>, Infallible> {
    let params = req.data::<Arc<ServerParams>>();
    let verbose = params
        .map(|params| params.verbose)
        .unwrap_or(DEFAULT_VERBOSE_ENABLED);
    let cors = params
        .map(|params| params.cors)
        .unwrap_or(DEFAULT_CORS_ENABLED);
    let delay = params
        .map(|params| params.delay_ms)
        .unwrap_or(DEFAULT_MOCKS_RESPONSE_DELAY);
    let mocks_dir = params
        .map(|params| params.get_absolute_mocks_path())
        .unwrap_or(Err(Box::from("Params not specified")));

    if verbose {
        println!("Serving {}", &req.uri());
    }
    let method = req.method().to_string().to_lowercase();
    let uri_pathname = req.uri().path().to_string();

    if mocks_dir.is_err() {
        return Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Full::new(Bytes::from("")))
            .unwrap());
    }
    let mocks_dir = mocks_dir.unwrap();
    let target_file_path = mocks_dir.join(format!("{}.{}", uri_pathname[1..].to_string(), method));

    if verbose {
        println!("Mocks dir: {}", &mocks_dir.display().to_string());
        println!(
            "Requested file: {}",
            &target_file_path.display().to_string()
        );
    }

    return match fs::read(&target_file_path).await {
        Ok(data) => {
            let mime = get_mime(&data);

            if verbose {
                println!("File mime: {}", &mime);
            }

            let mut builder = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", mime);
            if cors {
                builder = builder
                    .header("Access-Control-Allow-Origin", "*")
                    .header("Access-Control-Allow-Methods", "*");
            }
            let response = builder.body(Full::from(data)).unwrap();

            sleep(Duration::from_millis(delay)).await;

            Ok(response)
        }
        Err(_) => {
            let mut builder = Response::builder().status(StatusCode::NOT_FOUND);

            if cors {
                builder = builder
                    .header("Access-Control-Allow-Origin", "*")
                    .header("Access-Control-Allow-Methods", "*");
            }
            let response = builder.body(Full::new(Bytes::from(""))).unwrap();
            Ok(response)
        }
    };
}
