use std::{collections::HashMap, convert::Infallible, path::PathBuf, sync::Arc, time::Duration};

use http_body_util::{BodyExt, Full};
use hyper::{Request, Response, StatusCode, body::Bytes};
use routerify_ng::ext::RequestExt;
use tokio::{fs, time::sleep};

use crate::{
    controllers::utils::{
        add_headers, get_404_response, get_502_response, join_origin_and_path, write_mock,
    },
    libs::{cache_mode::CacheMode, get_mime::get_mime, mock_config::read_config},
    server::{
        mockers_context::MockersContext,
        params::{DEFAULT_CORS_ENABLED, DEFAULT_MOCKS_RESPONSE_DELAY, DEFAULT_VERBOSE_ENABLED},
    },
};

pub async fn mock_handler(req: Request<Full<Bytes>>) -> Result<Response<Full<Bytes>>, Infallible> {
    let context = req.data::<Arc<MockersContext>>();
    let verbose = context
        .map(|context| context.server_params.verbose)
        .unwrap_or(DEFAULT_VERBOSE_ENABLED);
    let cors = context
        .map(|context| context.server_params.cors)
        .unwrap_or(DEFAULT_CORS_ENABLED);
    let global_delay_ms = context
        .map(|context| context.server_params.delay_ms)
        .unwrap_or(DEFAULT_MOCKS_RESPONSE_DELAY);
    let mocks_dir = context
        .map(|context| context.server_params.get_absolute_mocks_path())
        .unwrap_or(Err(Box::from("Params not specified")));
    let origin = context.and_then(|context| context.clone().server_params.origin.clone());
    let client = context.map(|context| context.clone().client.clone());

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
    let mock_config_entry_name = mock_file_name.split("/").last().unwrap_or("");
    let mut current_mock_config_dir = PathBuf::from(uri_pathname[1..].to_string());
    current_mock_config_dir.pop();
    let config_file_path = mocks_dir.join(current_mock_config_dir).join("config.json");
    let config = read_config(&config_file_path).await;

    let delay = config
        .as_ref()
        .and_then(|c| c.get(mock_config_entry_name).and_then(|x| x.delay_ms))
        .unwrap_or(global_delay_ms);
    let custom_headers = config
        .as_ref()
        .and_then(|c| {
            c.get(mock_config_entry_name)
                .and_then(|x| x.headers.clone())
        })
        .unwrap_or_else(|| HashMap::new());
    let status_if_file_found = config
        .as_ref()
        .and_then(|c| c.get(mock_config_entry_name).and_then(|x| x.status_code))
        .map(|status_code| StatusCode::from_u16(status_code).unwrap_or(StatusCode::OK))
        .unwrap_or(StatusCode::OK);
    let cache_mode = config
        .as_ref()
        .and_then(|c| c.get(mock_config_entry_name))
        .and_then(|x| x.cache_mode.clone())
        .unwrap_or(CacheMode::NoCache);

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
            if let Some(origin) = origin
                && let Some(client) = client
            {
                let target_url = join_origin_and_path(&origin, &uri_pathname, req.uri().query());
                if verbose {
                    println!("Mock is missing, proxying request to {}", target_url);
                }

                let mut headers = req.headers().to_owned().clone();
                headers.remove(hyper::header::HOST);
                headers.remove(hyper::header::CONTENT_LENGTH);

                let response = client
                    .request(req.method().to_owned(), &target_url)
                    .headers(headers)
                    .body(reqwest::Body::wrap_stream(
                        req.body().clone().into_data_stream(),
                    ))
                    .send()
                    .await;

                return match response {
                    Err(_) => {
                        if verbose {
                            eprintln!("No response from origin: {}", &target_url)
                        }
                        Ok(get_502_response(cors, &custom_headers))
                    }
                    Ok(resp) => {
                        let resp_headers = resp.headers().clone();
                        let resp_status = resp.status().clone();
                        let response_bytes = resp.bytes().await.unwrap_or(Bytes::new());
                        let mut builder = Response::builder().status(resp_status);
                        for (header_key, header_val) in resp_headers {
                            if let Some(header_key) = header_key {
                                builder = builder.header(header_key, header_val);
                            }
                        }
                        builder = add_headers(builder, cors, &custom_headers);

                        if cache_mode == CacheMode::Overwrite {
                            write_mock(
                                &target_file_path.display().to_string(),
                                &response_bytes,
                                verbose,
                            );
                        }

                        Ok(builder.body(Full::new(response_bytes)).unwrap())
                    }
                };
            } else {
                return Ok(get_404_response(cors, &custom_headers));
            }
        }
    };
}
