use http_body_util::Full;
use hyper::HeaderMap;

use crate::libs::cache_mode::CacheMode;
use crate::libs::mock_config::{MockConfig, read_config};
use crate::libs::response_cache::InMemoryMocks;
use http::response::Builder;
use hyper::http::HeaderValue;
use hyper::{Response, StatusCode, body::Bytes, http};
use reqwest::header::CONTENT_TYPE;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::fs::{self, write};

pub fn join_origin_and_path(origin: &str, path: &str, query: Option<&str>) -> String {
    let origin = origin.trim_end_matches("/");
    let path = path.trim_start_matches("/");
    match query {
        Some(params_str) => format!("{}/{}?{}", origin, path, params_str),
        None => format!("{}/{}", origin, path),
    }
}

pub fn get_502_response(
    cors: bool,
    custom_headers: &HashMap<String, String>,
) -> Response<Full<Bytes>> {
    let mut builder = Response::builder().status(StatusCode::BAD_GATEWAY);
    builder = add_headers(builder, cors, &custom_headers);
    builder.body(Full::new(Bytes::new())).unwrap_or_default()
}

pub fn get_404_response(
    cors: bool,
    custom_headers: &HashMap<String, String>,
) -> Response<Full<Bytes>> {
    let mut builder = Response::builder().status(StatusCode::NOT_FOUND);
    builder = add_headers(builder, cors, &custom_headers);
    builder.body(Full::new(Bytes::new())).unwrap_or_default()
}

static CORS_HEADER_KEYS: &'static [&'static str] = &[
    "Access-Control-Allow-Methods",
    "Access-Control-Allow-Origin",
    "Access-Control-Allow-Headers",
];
static CORS_HEADER_VALUE: &'static str = "*";

pub fn add_headers(
    mut builder: Builder,
    add_cors: bool,
    custom_headers: &HashMap<String, String>,
) -> Builder {
    if add_cors && let Some(headers) = builder.headers_mut() {
        let anything_header: Result<HeaderValue, _> = CORS_HEADER_VALUE.parse();
        if let Ok(header_val) = anything_header {
            for header in CORS_HEADER_KEYS.iter() {
                headers.insert(*header, header_val.clone());
            }
        }
    }

    for (header, header_value) in custom_headers.iter() {
        builder = builder.header(header, header_value);
    }

    builder
}

pub fn write_mock_body(target_file_name: impl Into<String>, data: &Bytes, verbose: bool) {
    let data = data.clone();
    let target_file_name = target_file_name.into();
    tokio::spawn(async move {
        if let Err(e) = write(&target_file_name, &data).await {
            if verbose {
                eprintln!(
                    "Failed to write mock data to: '{}'. Original error: {}",
                    target_file_name, e
                )
            }
        }
    });
}

pub fn write_mock_metadata(
    target_entry_name: impl Into<String>,
    full_config_path: impl Into<String>,
    status_code: u16,
    headers: &HeaderMap,
    verbose: bool,
) {
    let full_config_path = full_config_path.into();
    let target_entry_name = target_entry_name.into();
    let headers = headers.clone().to_owned();
    tokio::spawn(async move {
        let headers_map = {
            let mut headers_map = HashMap::with_capacity(headers.keys_len());
            for (header_name, header_value) in headers {
                let pair = header_name.zip(header_value.to_str().ok());
                if let Some((header_name, header_value)) = pair {
                    headers_map.insert(header_name.to_string(), header_value.to_string());
                }
            }
            headers_map
        };

        let mut config = read_config(&full_config_path)
            .await
            .unwrap_or_else(|| HashMap::with_capacity(1));
        config.insert(
            target_entry_name,
            MockConfig {
                delay_ms: Some(0),
                cache_mode: Some(CacheMode::Overwrite),
                headers: Some(headers_map),
                status_code: status_code.into(),
            },
        );
        let json_str = serde_json::to_string(&config).unwrap_or(String::new());
        if let Err(e) = fs::write(&full_config_path, json_str).await {
            if verbose {
                eprintln!(
                    "Failed to write mock data to: '{}'. Original error: {}",
                    &full_config_path, e
                );
            }
        }
    });
}

pub async fn get_response_from_cache(
    cache: Option<Arc<InMemoryMocks>>,
    pathname: impl Into<String>,
    method: impl Into<String>,
    cors: bool,
) -> Option<(Response<Full<Bytes>>, u64)> {
    if let None = cache {
        return None;
    }

    let cache = cache.unwrap_or_else(|| Arc::new(InMemoryMocks::create()));

    let cached = cache
        .get_mock_by_path_and_method(pathname.into(), method.into())
        .await;

    if let Some(c) = cached {
        let headers = {
            let mut headers_map = HashMap::new();
            headers_map.insert(CONTENT_TYPE.to_string(), c.get_mime().await);
            for (header_key, header_value) in c.headers {
                headers_map.insert(header_key, header_value);
            }
            headers_map
        };
        let mut builder = Response::builder().status(
            c.status_code
                .try_into()
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        );
        builder = add_headers(builder, cors, &headers);

        return Some((
            builder.body(c.body.into()).unwrap_or(Response::default()),
            c.delay_ms,
        ));
    }

    None
}
