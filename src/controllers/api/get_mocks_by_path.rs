use crate::libs::mockers_errors::MockersErrors;
use crate::libs::protocol_result::ProtocolResultConverter;
use crate::libs::response_cache::{CachedResponse, PathDecoder, PathNormalizer};
use crate::server::mockers_context::MockersContext;
use crate::server::route_error::MockersRouteError;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::CONTENT_TYPE;
use hyper::{Request, Response, StatusCode};
use mimetype_detector::APPLICATION_JSON;
use routerify_ng::ext::RequestExt;
use std::collections::HashMap;
use std::sync::Arc;

pub async fn get_all_mocks_by_path(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
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
            &Err::<HashMap<String, HashMap<String, CachedResponse>>, String>(
                MockersErrors::GetMocksByPathFailed.to_string(),
            )
            .to_protocol(),
        );
        let bytes: Bytes = json.map(|s| s.into()).unwrap_or_default();
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(CONTENT_TYPE, APPLICATION_JSON)
            .body(bytes.into())
            .unwrap_or_default());
    };

    let mocks_by_path = cache.get_by_path(&path).await;
    let json = serde_json::to_string(
        &Ok::<Option<HashMap<String, CachedResponse>>, String>(mocks_by_path).to_protocol(),
    );
    let status_code = json
        .as_ref()
        .map(|_| StatusCode::OK)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let bytes: Bytes = json.map(|str| str.into()).unwrap_or_default();
    Ok(Response::builder()
        .status(&status_code)
        .header(CONTENT_TYPE, APPLICATION_JSON)
        .body(bytes.into())
        .unwrap_or_default())
}
