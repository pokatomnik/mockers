use std::{collections::HashMap, convert::Infallible, path::PathBuf, sync::Arc, time::Duration};

use crate::controllers::utils;
use crate::controllers::utils::{
    add_headers, get_404_response, get_502_response, join_origin_and_path, write_mock,
};
use crate::libs::response_cache::{MethodNormalizer, PathNormalizer};
use crate::libs::{cache_mode::CacheMode, get_mime::get_mime, mock_config::read_config};
use crate::server::mockers_context::MockersContext;
use crate::server::params::{
    CONFIG_FILE_NAME, DEFAULT_CORS_ENABLED, DEFAULT_MOCKS_RESPONSE_DELAY, DEFAULT_VERBOSE_ENABLED,
};
use http_body_util::{BodyExt, Full};
use hyper::{body::Bytes, Request, Response, StatusCode};
use reqwest::header::CONTENT_TYPE;
use routerify_ng::ext::RequestExt;
use tokio::{fs, time::sleep};

pub async fn mock_handler(req: Request<Full<Bytes>>) -> Result<Response<Full<Bytes>>, Infallible> {
    let context = req.data::<Arc<MockersContext>>();
    let mocks_cache = context.map(|ctx| ctx.clone().response_cache.clone());
    let verbose = context
        .map(|context| context.server_params.verbose)
        .unwrap_or(DEFAULT_VERBOSE_ENABLED);
    let cors = context
        .map(|context| context.server_params.cors)
        .unwrap_or(DEFAULT_CORS_ENABLED);
    let global_delay_ms = context
        .map(|context| context.server_params.delay_ms)
        .unwrap_or(DEFAULT_MOCKS_RESPONSE_DELAY);
    let absolute_mocks_dir = context
        .map(|context| context.server_params.get_absolute_mocks_path())
        .unwrap_or(Err(Box::from("Params not specified")));
    let origin = context.and_then(|context| context.clone().server_params.origin.clone());
    let client = context.map(|context| context.clone().client.clone());

    if verbose {
        println!("Serving {}", &req.uri());
    }
    let method = req.method().to_string().to_lowercase();
    let uri_pathname = req.uri().path().to_string();

    if absolute_mocks_dir.is_err() {
        return Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Full::new(Bytes::new()))
            .unwrap_or(Response::default()));
    }
    /*
     * Example:
     * `/home/username/.cargo/bin/mocks`
     */
    let absolute_mocks_dir = absolute_mocks_dir.expect("absolute_mocks_dir is missing");
    /*
     * Example:
     * `/foo/bar/baz.get`
     */
    let relative_mock_file_name = format!("{}.{}", uri_pathname[1..].to_string(), method);
    /*
     * Example:
     * `/home/username/.cargo/bin/mocks/foo/bar/baz.get`
     */
    let absolute_mock_file_name = absolute_mocks_dir.join(&relative_mock_file_name);
    /*
     * Example:
     * `baz.get`
     */
    let mock_config_entry_name = relative_mock_file_name.split("/").last().unwrap_or("");
    /*
     * Example:
     * `/foo/bar/`
     */
    let relative_current_mock_config_dir = {
        let mut pathbuf = PathBuf::from(uri_pathname[1..].to_string());
        pathbuf.pop();
        pathbuf
    };
    /*
     * Example:
     * `/home/username/.cargo/bin/mocks/foo/baz/config.json`
     */
    let config_file_path = absolute_mocks_dir
        .join(relative_current_mock_config_dir)
        .join(CONFIG_FILE_NAME);
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
        println!("Mocks dir: {}", &absolute_mocks_dir.display().to_string());
        println!(
            "Requested file: {}",
            &absolute_mock_file_name.display().to_string()
        );
    }

    // Try respond from cache
    let cached_data = utils::get_response_from_cache(
        mocks_cache,
        req.uri().path().to_string().normalize_path(),
        req.method().to_string().normalize_method(),
        cors,
    )
    .await;

    if let Some((response, delay_ms)) = cached_data {
        sleep(Duration::from_millis(delay_ms)).await;
        return Ok(response);
    }

    // Try respond from file-based mock
    if let Ok(data) = fs::read(&absolute_mock_file_name).await {
        let mime = get_mime(&data);

        if verbose {
            println!("File mime: {}", &mime);
        }

        let response = {
            let mut builder = Response::builder()
                .status(status_if_file_found)
                .header(CONTENT_TYPE, mime);
            builder = add_headers(builder, cors, &custom_headers);
            builder
                .body(Full::from(data))
                .unwrap_or(Response::default())
        };

        sleep(Duration::from_millis(delay)).await;

        return Ok(response);
    }

    // make request if remote server is specified
    let request_params = origin.zip(client).map(|(origin, client)| {
        let target_url = join_origin_and_path(&origin, &uri_pathname, req.uri().query());
        if verbose {
            println!("Mock is missing, proxying request to {}", target_url);
        }

        let headers = {
            let mut res = req.headers().to_owned().clone();
            res.remove(hyper::header::HOST);
            res.remove(hyper::header::CONTENT_LENGTH);
            res
        };

        let request_builder = client
            .request(req.method().to_owned(), &target_url)
            .headers(headers)
            .body(reqwest::Body::wrap_stream(
                req.body().clone().into_data_stream(),
            ));

        (request_builder, target_url)
    });

    // Send response if remove server is specified
    let response_params = async {
        match request_params {
            None => None,
            Some((request, target_url)) => Some((request.send().await, target_url)),
        }
    }
    .await;

    // return 404 if no remote server specified
    let Some((response_result, target_url)) = response_params else {
        return Ok(get_404_response(cors, &custom_headers));
    };

    let Ok(resp) = response_result else {
        if verbose {
            eprintln!("No response from origin: '{}'", target_url)
        }
        return Ok(get_502_response(cors, &custom_headers));
    };

    let builder = {
        let resp_headers = resp.headers().clone();
        let resp_status = resp.status().clone();
        let mut builder = Response::builder().status(resp_status);
        for (header_key, header_val) in resp_headers {
            if let Some(header_key) = header_key {
                builder = builder.header(header_key, header_val);
            }
        }
        builder = add_headers(builder, cors, &custom_headers);
        builder
    };
    let response_bytes = resp.bytes().await.unwrap_or(Bytes::new());

    if cache_mode == CacheMode::Overwrite {
        write_mock(
            &absolute_mock_file_name.display().to_string(),
            &response_bytes,
            verbose,
        );
    }

    Ok(builder
        .body(Full::new(response_bytes))
        .unwrap_or(Response::default()))
}
