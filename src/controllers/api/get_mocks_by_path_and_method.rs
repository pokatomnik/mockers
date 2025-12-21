use crate::libs::protocol_result::ProtocolResult;
use crate::libs::response_cache::CachedResponse;
use crate::server::mockers_context::MockersContext;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response, StatusCode};
use routerify_ng::ext::RequestExt;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

pub async fn get_mocks_by_path_and_method(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let path = req
        .params()
        .get("path_encoded")
        .map(|path_raw| {
            BASE64_STANDARD
                .decode(path_raw)
                .map(|v| String::from_utf8(v).unwrap_or(String::new()))
                .unwrap_or(String::new())
        })
        .unwrap_or(String::new());
    let method = req
        .params()
        .get("method")
        .map(|v| v.to_owned())
        .unwrap_or(String::new());

    let response_cache = req
        .data::<Arc<MockersContext>>()
        .map(|c| c.clone().response_cache.clone());

    match response_cache {
        Some(cache) => {
            let mocks_by_path_and_method = cache.get_mock_by_path_and_method(path, method).await;
            let json = serde_json::to_string(
                &ProtocolResult::Ok::<Option<CachedResponse>, String>(mocks_by_path_and_method),
            );
            let status_code = json
                .as_ref()
                .map(|_| StatusCode::OK)
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            let bytes = json
                .map(|str| Bytes::from(str))
                .unwrap_or(Bytes::from(Bytes::new()));
            Ok(Response::builder()
                .status(&status_code)
                .body(Full::new(bytes))
                .unwrap())
        }
        None => {
            let json = serde_json::to_string(&ProtocolResult::Err::<
                HashMap<String, HashMap<String, CachedResponse>>,
                String,
            >(
                "GET_MOCKS_BY_PATH_AND_METHOD_FAILED".to_string()
            ));
            let bytes = json.map(|s| Bytes::from(s)).unwrap_or(Bytes::new());
            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from(bytes)))
                .unwrap())
        }
    }
}
