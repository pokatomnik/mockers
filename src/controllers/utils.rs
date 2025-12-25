use http_body_util::Full;

use http::response::Builder;
use hyper::{Response, StatusCode, body::Bytes, http};
use std::collections::HashMap;
use tokio::fs::write;

pub fn join_origin_and_path(origin: &str, path: &str, query: Option<&str>) -> String {
    let origin = origin.trim_end_matches("/");
    let path = path.trim_start_matches("/");
    return match query {
        Some(params_str) => format!("{}/{}?{}", origin, path, params_str),
        None => format!("{}/{}", origin, path),
    };
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

    return builder;
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