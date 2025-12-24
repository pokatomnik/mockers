use crate::libs::mockers_errors::MockersErrors;
use crate::libs::protocol_result::ProtocolResultConverter;
use crate::libs::response_cache::{MethodNormalizer, PathDecoder, PathNormalizer};
use crate::server::mockers_context::MockersContext;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::CONTENT_TYPE;
use hyper::{Request, Response, StatusCode};
use mimetype_detector::APPLICATION_JSON;
use routerify_ng::ext::RequestExt;
use std::convert::Infallible;
use std::sync::Arc;

pub async fn delete_remove_mock_by_path_and_method(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let path = req
        .params()
        .get("path_encoded")
        .and_then(|str| str.decode_path().ok())
        .unwrap_or(String::new())
        .normalize_path();

    let method = req
        .params()
        .get("method")
        .map(|v| v.to_owned())
        .unwrap_or(String::new())
        .normalize_method();

    let response_cache = req
        .data::<Arc<MockersContext>>()
        .map(|c| c.clone().response_cache.clone());

    match response_cache {
        Some(cache) => {
            let p = path.clone();
            let m = method.clone();
            tokio::spawn(async move {
                cache.remove_by_path_and_method(p, m).await;
            });
            let json = serde_json::to_string(
                &Ok::<String, String>(
                    format!(
                        "Removed mocks for path: '{}' and method: '{}'",
                        path, method
                    )
                    .to_string(),
                )
                .to_protocol(),
            );
            let status = json
                .as_ref()
                .map(|_| StatusCode::OK)
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            let bytes = json.map(|str| Bytes::from(str)).unwrap_or(Bytes::new());
            Ok(Response::builder()
                .status(status)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body(Full::new(Bytes::from(bytes)))
                .unwrap_or(Response::default()))
        }
        None => {
            let json = serde_json::to_string(
                &Err::<String, String>(MockersErrors::RemoveMocksFailedByPathAndMethod.to_string())
                    .to_protocol(),
            );
            let bytes = json.map(|str| Bytes::from(str)).unwrap_or(Bytes::new());
            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body(Full::new(Bytes::from(bytes)))
                .unwrap_or(Response::default()))
        }
    }
}
