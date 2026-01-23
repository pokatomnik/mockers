use crate::libs::mockers_errors::MockersErrors;
use crate::libs::protocol_result::ProtocolResultConverter;
use crate::libs::response_cache::CachedResponse;
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

pub async fn get_all_mocks(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    let response_cache = req
        .data::<Arc<MockersContext>>()
        .map(|c| c.clone().response_cache.clone());

    let Some(cache) = response_cache else {
        let json = serde_json::to_string(
            &Err::<HashMap<String, HashMap<String, CachedResponse>>, String>(
                MockersErrors::GetAllMocksFailed.to_string(),
            )
            .to_protocol(),
        );
        let bytes: Bytes = json.map(|str| str.into()).unwrap_or_default();
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(CONTENT_TYPE, APPLICATION_JSON)
            .body(bytes.into())
            .unwrap_or_default());
    };

    let all_mocks = cache.get_all().await;
    let json = serde_json::to_string(
        &Ok::<HashMap<String, HashMap<String, CachedResponse>>, String>(all_mocks).to_protocol(),
    );
    let status_code = json
        .as_ref()
        .map(|_| StatusCode::OK)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let bytes: Bytes = json.map(|str| str.into()).unwrap_or_default();
    Ok(Response::builder()
        .status(status_code)
        .header(CONTENT_TYPE, APPLICATION_JSON)
        .body(bytes.into())
        .unwrap_or_default())
}
