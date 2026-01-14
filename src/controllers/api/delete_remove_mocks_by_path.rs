use crate::libs::mockers_errors::MockersErrors;
use crate::libs::protocol_result::ProtocolResultConverter;
use crate::libs::response_cache::{PathDecoder, PathNormalizer};
use crate::server::mockers_context::MockersContext;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::CONTENT_TYPE;
use hyper::{Request, Response, StatusCode};
use mimetype_detector::APPLICATION_JSON;
use routerify_ng::ext::RequestExt;
use std::convert::Infallible;
use std::sync::Arc;

pub async fn delete_remove_mocks_by_path(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let path = req
        .params()
        .get("path_encoded")
        .and_then(|str| str.decode_path().ok())
        .unwrap_or_default()
        .normalize_path();

    let response_cache = req
        .data::<Arc<MockersContext>>()
        .map(|c| c.clone().response_cache.clone());

    let Some(cache) = response_cache else {
        let json = serde_json::to_string(
            &Err::<String, String>(MockersErrors::RemoveMocksFailedByPath.to_string())
                .to_protocol(),
        );
        let bytes: Bytes = json.map(|str| str.into()).unwrap_or_default();
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(CONTENT_TYPE, APPLICATION_JSON)
            .body(bytes.into())
            .unwrap_or_default());
    };

    let p = path.clone();
    tokio::spawn(async move {
        cache.remove_by_path(path).await;
    });
    let json = serde_json::to_string(
        &Ok::<String, String>(format!("Removed mocks for path: '{}'", p).to_string()).to_protocol(),
    );
    let status = json
        .as_ref()
        .map(|_| StatusCode::OK)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let bytes: Bytes = json.map(|str| str.into()).unwrap_or_default();
    Ok(Response::builder()
        .status(status)
        .header(CONTENT_TYPE, APPLICATION_JSON)
        .body(bytes.into())
        .unwrap_or_default())
}
