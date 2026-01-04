use crate::libs::mockers_errors::MockersErrors;
use crate::libs::protocol_result::ProtocolResultConverter;
use crate::libs::response_cache::{CachedResponse, PathDecoder, PathNormalizer};
use crate::server::mockers_context::MockersContext;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::CONTENT_TYPE;
use hyper::{Request, Response, StatusCode};
use mimetype_detector::APPLICATION_JSON;
use routerify_ng::ext::RequestExt;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

pub async fn get_all_mocks_by_path(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let path = req
        .params()
        .get("path_encoded")
        .and_then(|str| str.decode_path().ok())
        .unwrap_or(String::new())
        .normalize_path();

    let response_cache = req
        .data::<Arc<MockersContext>>()
        .map(|c| c.clone().response_cache.clone());
    match response_cache {
        Some(cache) => {
            let mocks_by_path = cache.get_by_path(&path).await;
            let json = serde_json::to_string(
                &Ok::<Option<HashMap<String, CachedResponse>>, String>(mocks_by_path).to_protocol(),
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
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body(Full::new(bytes))
                .unwrap_or(Response::default()))
        }
        None => {
            let json = serde_json::to_string(
                &Err::<HashMap<String, HashMap<String, CachedResponse>>, String>(
                    MockersErrors::GetMocksByPathFailed.to_string(),
                )
                .to_protocol(),
            );
            let bytes = json.map(|s| Bytes::from(s)).unwrap_or(Bytes::new());
            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body(Full::new(Bytes::from(bytes)))
                .unwrap_or(Response::default()))
        }
    }
}
