use std::{collections::HashMap, convert::Infallible, sync::Arc, time::Duration};

use http::response::Builder;
use http_body_util::Full;
use hyper::{Request, Response, StatusCode, body::Bytes, http};
use routerify_ng::ext::RequestExt;
use tokio::{fs, time::sleep};

use crate::{
    libs::{get_mime::get_mime, mock_config::read_config},
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
    let global_delay_ms = params
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
    let mock_file_name = format!("{}.{}", uri_pathname[1..].to_string(), method);
    let target_file_path = mocks_dir.join(&mock_file_name);
    let config_file_path = mocks_dir.join("config.json");
    let config = read_config(&config_file_path).await;

    let delay = config
        .as_ref()
        .and_then(|c| c.get(&mock_file_name).and_then(|x| x.delay_ms))
        .unwrap_or(global_delay_ms);
    let custom_headers = config
        .as_ref()
        .and_then(|c| c.get(&mock_file_name).and_then(|x| x.headers.clone()))
        .unwrap_or_else(|| HashMap::new());
    let status_if_file_found = config
        .as_ref()
        .and_then(|c| c.get(&mock_file_name).and_then(|x| x.status_code))
        .map(|status_code| StatusCode::from_u16(status_code).unwrap_or(StatusCode::OK))
        .unwrap_or(StatusCode::OK);

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
                .status(status_if_file_found)
                .header("Content-Type", mime);
            builder = add_headers(builder, cors, &custom_headers);
            let response = builder.body(Full::from(data)).unwrap();

            sleep(Duration::from_millis(delay.to_owned())).await;

            Ok(response)
        }
        Err(_) => {
            let mut builder = Response::builder().status(StatusCode::NOT_FOUND);
            builder = add_headers(builder, cors, &custom_headers);
            let response = builder.body(Full::new(Bytes::from(""))).unwrap();
            Ok(response)
        }
    };
}

fn add_headers(
    mut builder: Builder,
    add_cors: bool,
    custom_headers: &HashMap<String, String>,
) -> Builder {
    if add_cors {
        builder = builder
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "*")
    }

    for (header, header_value) in custom_headers.iter() {
        builder = builder.header(header, header_value);
    }

    return builder;
}
