//! Get all GraphQL mocks endpoint.

use std::convert::Infallible;
use std::sync::Arc;

use http_body_util::Full;
use hyper::header::CONTENT_TYPE;
use hyper::{body::Bytes, Request, Response, StatusCode};
use mimetype_detector::APPLICATION_JSON;
use routerify_ng::ext::RequestExt;
use serde::Serialize;

use crate::libs::graphql_cache::{GraphQLCachedResponse, GraphQLOperationType};
use crate::libs::mockers_errors::MockersErrors;
use crate::libs::protocol_result::ProtocolResultConverter;
use crate::server::mockers_context::MockersContext;

/// Serializable representation of a GraphQL mock for API responses
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphQLMockResponse {
    pub path: String,
    pub operation_name: String,
    pub operation_type: GraphQLOperationType,
    pub status_code: u16,
    pub delay_ms: u64,
    pub data: serde_json::Value,
    pub errors: Option<Vec<serde_json::Value>>,
}

impl GraphQLMockResponse {
    pub fn from_cached(
        path: String,
        operation_name: String,
        operation_type: GraphQLOperationType,
        cached: &GraphQLCachedResponse,
    ) -> Self {
        Self {
            path,
            operation_name,
            operation_type,
            status_code: cached.status_code,
            delay_ms: cached.delay_ms,
            data: cached.data.clone(),
            errors: cached.errors.as_ref().map(|e| {
                e.iter()
                    .map(|err| serde_json::to_value(err).unwrap_or_default())
                    .collect()
            }),
        }
    }
}

/// GET /api/v1/graphql/mocks - Get all GraphQL mocks
pub async fn get_all_graphql_mocks(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let Some(cache) = req.data::<Arc<MockersContext>>().map(|ctx| ctx.graphql_cache.clone()) else {
        let json = serde_json::to_string(
            &Err::<Vec<GraphQLMockResponse>, _>(MockersErrors::GetAllMocksFailed.to_string())
                .to_protocol(),
        );
        let bytes = json.map(Bytes::from).unwrap_or_default();

        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(CONTENT_TYPE, APPLICATION_JSON)
            .body(Full::new(bytes))
            .unwrap_or_default());
    };

    let mocks: Vec<GraphQLMockResponse> = cache
        .get_all()
        .await
        .into_iter()
        .map(|(key, response)| {
            GraphQLMockResponse::from_cached(key.path, key.operation_name, key.operation_type, &response)
        })
        .collect();

    let json = serde_json::to_string(&Ok::<_, String>(mocks).to_protocol());
    let bytes = json.map(Bytes::from).unwrap_or_default();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, APPLICATION_JSON)
        .body(Full::new(bytes))
        .unwrap_or_default())
}
