use http_body_util::Full;

use crate::libs::response_cache::InMemoryMocks;
use http::response::Builder;
use hyper::http::HeaderValue;
use hyper::{body::Bytes, http, Response, StatusCode};
use reqwest::header::CONTENT_TYPE;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::fs::write;

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

pub fn write_mock(target_file_name: &str, data: &Bytes, verbose: bool) {
    let data = data.clone();
    let target_file_name = target_file_name.to_string();
    tokio::spawn(async move {
        if let Err(_) = write(&target_file_name, &data).await {
            if verbose {
                eprintln!("Failed to write mock data to: '{}'", target_file_name)
            }
        }
    });
}

pub async fn get_response_from_cache<P, M>(
    cache: Option<Arc<InMemoryMocks>>,
    pathname: P,
    method: M,
    cors: bool,
) -> Option<Response<Full<Bytes>>>
where
    P: Into<String>,
    M: Into<String>,
{
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

        return Some(builder.body(c.body.into()).unwrap_or(Response::default()));
    }

    None
}
