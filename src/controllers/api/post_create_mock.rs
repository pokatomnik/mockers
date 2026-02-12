use crate::libs::cached_response::CachedResponse;
use crate::libs::mockers_errors::MockersErrors;
use crate::libs::protocol_result::ProtocolResultConverter;
use crate::libs::response_builder_ext::ResponseBuilderExt;
use crate::libs::response_ext::WellKnownResponses;
use crate::server::mockers_context::MockersContext;
use crate::server::route_error::MockersRouteError;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::{Request, Response};
use routerify_ng::ext::RequestExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

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
        let response = Response::internal_server_error()
            .content_type_json()
            .body(bytes.into())
            .unwrap_or_default();
        return Ok(response);
    };

    let body_bytes = req
        .body()
        .clone()
        .collect()
        .await
        .unwrap_or_default()
        .to_bytes();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap_or_default();
    let create_mock_params: Result<CreateMockParams, serde_json::Error> = body_str.try_into();

    let Ok(mock_params) = create_mock_params else {
        return Ok(ADD_MOCK_FAILED.clone());
    };

    // let cached_response = CachedResponse::new(
    //     mock_params.status_code,
    //     mock_params.delay_ms.unwrap_or(0),
    //     mock_params.headers,
    //     mock_params.body,
    // );
    let cached_response = CachedResponse::new()
        .with_status_code(mock_params.status_code)
        .with_delay_ms(mock_params.delay_ms.unwrap_or(0))
        .with_headers(mock_params.headers)
        .with_body(mock_params.body);

    cache
        .upsert(&mock_params.path, &mock_params.method, &cached_response)
        .await;

    let message = format!(
        "Mock inserted for path: '{}' and method: '{}'",
        &mock_params.path, &mock_params.method
    );
    let body = Ok::<String, &str>(message).to_protocol().to_string().into();

    let response = Response::ok()
        .content_type_json()
        .body(body)
        .unwrap_or_default();

    Ok(response)
}

static ADD_MOCK_FAILED: LazyLock<Response<Full<Bytes>>> = LazyLock::new(|| {
    let body = Err::<&str, String>(MockersErrors::AddMockFailed.to_string())
        .to_protocol()
        .to_string()
        .into();
    Response::internal_server_error()
        .content_type_json()
        .body(body)
        .unwrap_or_default()
});

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

impl TryFrom<String> for CreateMockParams {
    type Error = serde_json::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        serde_json::from_str::<CreateMockParams>(&value)
    }
}
