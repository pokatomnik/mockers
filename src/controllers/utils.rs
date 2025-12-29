use http_body_util::Full;

use crate::libs::response_cache::InMemoryMocks;
use http::response::Builder;
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
    builder.body(Full::new(Bytes::new())).unwrap()
}

pub fn get_404_response(
    cors: bool,
    custom_headers: &HashMap<String, String>,
) -> Response<Full<Bytes>> {
    let mut builder = Response::builder().status(StatusCode::NOT_FOUND);
    builder = add_headers(builder, cors, &custom_headers);
    builder.body(Full::new(Bytes::new())).unwrap()
}

pub fn add_headers(
    mut builder: Builder,
    add_cors: bool,
    custom_headers: &HashMap<String, String>,
) -> Builder {
    if add_cors {
        let headers = builder.headers_mut().unwrap();
        headers.insert("Access-Control-Allow-Methods", "*".parse().unwrap());
        headers.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
        headers.insert("Access-Control-Allow-Headers", "*".parse().unwrap());
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

        return Some(builder.body(c.body.into()).unwrap_or(Response::default()))
    }
    
    None
}
