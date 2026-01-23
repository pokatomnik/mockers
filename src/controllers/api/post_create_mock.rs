use crate::libs::mockers_errors::MockersErrors;
use crate::libs::protocol_result::ProtocolResultConverter;
use crate::libs::response_cache::CachedResponse;
use crate::server::mockers_context::MockersContext;
use crate::server::route_error::MockersRouteError;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::header::CONTENT_TYPE;
use hyper::{Request, Response, StatusCode};
use mimetype_detector::APPLICATION_JSON;
use routerify_ng::ext::RequestExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateMockParams {
    path: String,
    method: String,
    status_code: u16,
    delay_ms: Option<u64>,
    headers: HashMap<String, String>,
    body: String,
}

pub async fn post_create_mock(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    let response_cache = req
        .data::<Arc<MockersContext>>()
        .map(|c| c.clone().response_cache.clone());

    let Some(cache) = response_cache else {
        let json = serde_json::to_string(
            &Err::<HashMap<String, HashMap<String, CachedResponse>>, String>(
                MockersErrors::AddMockFailed.to_string(),
            )
            .to_protocol(),
        );
        let bytes = json.map(|s| Bytes::from(s)).unwrap_or_default();
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(CONTENT_TYPE, APPLICATION_JSON)
            .body(bytes.into())
            .unwrap_or_default());
    };

    let body_bytes = req
        .body()
        .clone()
        .collect()
        .await
        .unwrap_or_default()
        .to_bytes();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap_or_default();
    let create_mock_params = serde_json::from_str::<CreateMockParams>(&body_str);

    let Ok(mock_params) = create_mock_params else {
        let json = serde_json::to_string(
            &Err::<HashMap<String, HashMap<String, CachedResponse>>, String>(
                MockersErrors::AddMockFailed.to_string(),
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

    let path = mock_params.path.clone();
    let method = mock_params.method.clone();

    // insert asynchronously
    tokio::spawn(async move {
        let response = &CachedResponse::new(
            mock_params.status_code,
            mock_params.delay_ms.unwrap_or(0),
            mock_params.headers,
            mock_params.body,
        );
        cache.upsert(&path, &method, response).await;
    });

    let json = serde_json::to_string(
        &Ok::<String, String>(format!(
            "Mock inserted for path: '{}' and method: '{}'",
            mock_params.path, mock_params.method
        ))
        .to_protocol(),
    );
    let status = json
        .as_ref()
        .map(|_| StatusCode::OK)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let bytes: Bytes = json.map(|str| str.into()).unwrap_or_default();
    return Ok(Response::builder()
        .status(status)
        .header(CONTENT_TYPE, APPLICATION_JSON)
        .body(bytes.into())
        .unwrap_or_default());
}
