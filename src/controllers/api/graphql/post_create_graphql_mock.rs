//! Create GraphQL mock endpoint.

use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use http_body_util::{BodyExt, Full};
use hyper::header::CONTENT_TYPE;
use hyper::{body::Bytes, Request, Response, StatusCode};
use mimetype_detector::APPLICATION_JSON;
use routerify_ng::ext::RequestExt;
use serde::Deserialize;

use crate::libs::graphql_cache::{
    GraphQLCachedResponse, GraphQLError, GraphQLMockKey, GraphQLOperationType,
};
use crate::libs::mockers_errors::MockersErrors;
use crate::libs::protocol_result::ProtocolResultConverter;
use crate::server::mockers_context::MockersContext;

/// Parameters for creating a GraphQL mock
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGraphQLMockParams {
    /// Endpoint path (e.g., "/graphql")
    pub path: String,
    /// GraphQL operation name (e.g., "GetUser")
    pub operation_name: String,
    /// Operation type: query, mutation, or subscription
    #[serde(default)]
    pub operation_type: GraphQLOperationType,
    /// HTTP status code (default: 200)
    #[serde(default = "default_status_code")]
    pub status_code: u16,
    /// Response delay in milliseconds (default: 0)
    #[serde(default)]
    pub delay_ms: u64,
    /// Custom response headers
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// GraphQL response data
    pub data: serde_json::Value,
    /// Optional GraphQL errors
    #[serde(default)]
    pub errors: Option<Vec<GraphQLError>>,
}

fn default_status_code() -> u16 {
    200
}

/// POST /api/v1/graphql/mocks - Create a new GraphQL mock
pub async fn post_create_graphql_mock(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let graphql_cache = req
        .data::<Arc<MockersContext>>()
        .map(|ctx| ctx.graphql_cache.clone());

    match graphql_cache {
        Some(cache) => {
            // Parse request body
            let body_bytes = req
                .body()
                .clone()
                .collect()
                .await
                .map(|buf| buf.to_bytes())
                .unwrap_or_default();

            let body_str = String::from_utf8(body_bytes.to_vec()).unwrap_or_default();

            let params: CreateGraphQLMockParams = match serde_json::from_str(&body_str) {
                Ok(p) => p,
                Err(e) => {
                    return Ok(build_error_response(
                        StatusCode::BAD_REQUEST,
                        &format!("Invalid request body: {}", e),
                    ));
                }
            };

            // Validate operation name
            if params.operation_name.is_empty() {
                return Ok(build_error_response(
                    StatusCode::BAD_REQUEST,
                    "Operation name cannot be empty",
                ));
            }

            // Validate path
            if params.path.is_empty() {
                return Ok(build_error_response(
                    StatusCode::BAD_REQUEST,
                    "Path cannot be empty",
                ));
            }

            let key = GraphQLMockKey::new(
                &params.path,
                &params.operation_name,
                params.operation_type,
            );

            let response = GraphQLCachedResponse::new(
                params.status_code,
                params.delay_ms,
                params.headers,
                params.data,
                params.errors,
            );

            // Store mock asynchronously
            let path = params.path.clone();
            let operation_name = params.operation_name.clone();
            let operation_type = params.operation_type;

            tokio::spawn(async move {
                cache.upsert(key, response).await;
            });

            let json = serde_json::to_string(
                &Ok::<_, String>(format!(
                    "GraphQL mock created for operation '{}' ({}) at path '{}'",
                    operation_name, operation_type, path
                ))
                .to_protocol(),
            );
            let bytes = json.map(Bytes::from).unwrap_or_default();

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, APPLICATION_JSON)
                .body(Full::new(bytes))
                .unwrap_or_default())
        }
        None => Ok(build_error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &MockersErrors::AddMockFailed.to_string(),
        )),
    }
}

fn build_error_response(status: StatusCode, error: &str) -> Response<Full<Bytes>> {
    let json = serde_json::to_string(&Err::<String, _>(error.to_string()).to_protocol());
    let bytes = json.map(Bytes::from).unwrap_or_default();

    Response::builder()
        .status(status)
        .header(CONTENT_TYPE, APPLICATION_JSON)
        .body(Full::new(bytes))
        .unwrap_or_default()
}
