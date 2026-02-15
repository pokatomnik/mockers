use std::{path::PathBuf, sync::Arc, time::Duration};

use crate::libs::absolute_mocks_path::AbsoluteMocksPath;
use crate::libs::header_map_ext::{HeaderMapConverter, HeaderMapSanitizer};
use crate::libs::in_memory_mocks::{MethodNormalizer, PathNormalizer};
use crate::libs::mock_config::MockConfig;
use crate::libs::mockers_request_ext::MockersRequestExt;
use crate::libs::response_builder_ext::ResponseBuilderExt;
use crate::libs::response_ext::WellKnownResponses;
use crate::libs::tap::Tap;
use crate::libs::url_ext::UrlExt;
use crate::libs::{cache_mode::CacheMode, get_mime::get_mime};
use crate::server::mockers_context::MockersContext;
use crate::server::params::{CONFIG_FILE_NAME, DEFAULT_CORS_ENABLED, DEFAULT_MOCKS_RESPONSE_DELAY};
use crate::server::route_error::MockersRouteError;
use http_body_util::{BodyExt, Full};
use hyper::{body::Bytes, Request, Response, StatusCode};
use reqwest::Url;
use routerify_ng::ext::RequestExt;
use tokio::join;

pub async fn mock_handler(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    let context = req.data::<Arc<MockersContext>>();
    let mocks_cache = context.map(|ctx| ctx.clone().response_cache.clone());
    let cors = context
        .map(|context| context.server_params.cors())
        .unwrap_or(DEFAULT_CORS_ENABLED);
    let preflight = context
        .map(|context| context.server_params.preflight())
        .flatten();
    let global_delay_ms = context
        .map(|context| context.server_params.delay_ms())
        .unwrap_or(DEFAULT_MOCKS_RESPONSE_DELAY);
    let absolute_mocks_dir = context
        .map(|context| context.server_params.get_absolute_mocks_path())
        .unwrap_or(Err(Box::from("Params not specified")));
    let origin =
        context.and_then(|context| context.clone().server_params.origin().map(String::from));
    let client = context.map(|context| context.clone().client.clone());

    let method = req.method().to_string().to_lowercase();
    let uri_pathname = req.uri().path().to_string();

    if absolute_mocks_dir.is_err() {
        return Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Full::default())
            .unwrap_or_default());
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
    let mock_config_entry_name = relative_mock_file_name
        .split("/")
        .last()
        .unwrap_or("")
        .to_string();
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
    let config = MockConfig::try_read_from_file(&config_file_path).await.ok();

    let delay = config
        .as_ref()
        .and_then(|c| c.get(&mock_config_entry_name).and_then(|x| x.delay_ms()))
        .unwrap_or(global_delay_ms);
    let custom_headers = config
        .as_ref()
        .and_then(|c| {
            c.get(&mock_config_entry_name)
                .and_then(|x| x.headers().cloned())
        })
        .unwrap_or_default();
    let status_if_file_found = config
        .as_ref()
        .and_then(|c| c.get(&mock_config_entry_name).and_then(|x| x.status_code()))
        .map(|status_code| StatusCode::from_u16(status_code).unwrap_or(StatusCode::OK))
        .unwrap_or(StatusCode::OK);
    let cache_mode = config
        .as_ref()
        .and_then(|c| c.get(&mock_config_entry_name))
        .and_then(|x| x.cache_mode())
        .unwrap_or(CacheMode::NoCache);

    let handle_preflight = match (preflight, req.is_preflight()) {
        (Some(preflight), true) => Some(preflight),
        _ => None,
    };

    // Check if this request is a special "preflight" browser request and user enforced handing It
    if let Some(handle_preflight) = handle_preflight {
        let response = Response::no_content()
            .tap(|builder| if cors { builder.add_cors() } else { builder })
            .add_custom_headers(
                req.preflight_response_headers(*handle_preflight)
                    .into_iter(),
            )
            .empty_body()
            .unwrap_or_default();
        return Ok(response);
    }

    // Try respond from cache
    let cached = match mocks_cache {
        Some(mocks_cache) => {
            let path = req.uri().path().to_string().normalize_path();
            let method = req.method().to_string().normalize_method();
            mocks_cache.get_mock_by_path_and_method(path, method).await
        }
        None => None,
    };
    if let Some(cached) = cached {
        let mime = cached.get_mime().await;
        let cached_headers = cached.headers.into_iter();
        let body_bytes = cached.body.into();
        let response = Response::builder()
            .status(cached.status_code)
            .add_content_type_header(mime.as_str())
            .add_custom_headers(cached_headers)
            .tap(|builder| if cors { builder.add_cors() } else { builder })
            .body(body_bytes)
            .unwrap_or_default();
        tokio::time::sleep(Duration::from_millis(cached.delay_ms)).await;
        return Ok(response);
    }

    // Try respond from file-based mock
    if let Ok(data) = tokio::fs::read(&absolute_mock_file_name).await {
        let mime = get_mime(&data);
        let response = Response::builder()
            .status(status_if_file_found)
            .add_content_type_header(&mime)
            .tap(|builder| if cors { builder.add_cors() } else { builder })
            .add_custom_headers(custom_headers.into_iter())
            .body(data.into())
            .unwrap_or_default();

        tokio::time::sleep(Duration::from_millis(delay)).await;

        return Ok(response);
    }

    // return 404 if no remote server specified
    let Some((origin, client)) = origin.zip(client) else {
        let response = Response::not_found()
            .tap(|builder| if cors { builder.add_cors() } else { builder })
            .add_custom_headers(custom_headers.into_iter())
            .empty_body()
            .unwrap_or_default();

        return Ok(response);
    };

    // make request if remote server is specified
    let Ok(target_url) = Url::from_parts(&origin, &uri_pathname, req.uri().query()) else {
        let response = Response::bad_request()
            .tap(|builder| if cors { builder.add_cors() } else { builder })
            .add_custom_headers(custom_headers.into_iter())
            .empty_body()
            .unwrap_or_default();

        return Ok(response);
    };

    let request_headers = req
        .headers()
        .to_owned()
        .remove_host_header()
        .remove_content_length_header();

    let request_builder = client
        .request(req.method().to_owned(), target_url)
        .headers(request_headers)
        .body(reqwest::Body::wrap_stream(
            req.body().clone().into_data_stream(),
        ));

    // Send response if remove server is specified
    let Ok(response) = request_builder.send().await else {
        let response = Response::bad_gateway()
            .tap(|builder| if cors { builder.add_cors() } else { builder })
            .add_custom_headers(custom_headers.into_iter())
            .empty_body()
            .unwrap_or_default();
        return Ok(response);
    };

    let resp_status = response.status().clone();

    let resp_headers = response.headers().clone();
    let save_headers = resp_headers.clone();

    let response_bytes = response.bytes().await.unwrap_or_default();
    let save_bytes = response_bytes.clone();

    let response = Response::builder()
        .status(resp_status)
        .add_custom_headers(resp_headers.kv_iter())
        .body(response_bytes.into())
        .unwrap_or_default();

    if cache_mode == CacheMode::Overwrite {
        tokio::spawn(async move {
            let mock_config = MockConfig::new()
                .with_delay_ms(0)
                .with_cache_mode(CacheMode::Overwrite)
                .with_headers(save_headers.to_hash_map())
                .with_status_code(resp_status.into());

            let write_config_fut =
                mock_config.try_write_to_file(&config_file_path, mock_config_entry_name);
            let write_mock_fut = tokio::fs::write(absolute_mock_file_name, &save_bytes);

            // TODO unused, should be logged maybe
            let _ = join!(write_config_fut, write_mock_fut);
        });
    }

    Ok(response)
}
