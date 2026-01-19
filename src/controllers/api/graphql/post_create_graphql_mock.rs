//! Create GraphQL mock endpoint.

use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use http_body_util::{BodyExt, Full};
use hyper::header::CONTENT_TYPE;
use hyper::{body::Bytes, Request, Response, StatusCode};
use mimetype_detector::APPLICATION_JSON;
use routerify_ng::ext::RequestExt;
use serde::{Deserialize, Serialize};

use crate::libs::graphql_cache::{
    compute_query_hash, GraphQLCachedResponse, GraphQLError, GraphQLMockKey,
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
    /// GraphQL query string to match
    pub query: String,
    /// Operation name (optional, for multi-operation documents)
    #[serde(default)]
    pub operation_name: Option<String>,
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

/// Response for successful mock creation
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateMockResponse {
    message: String,
    query_hash: String,
}

/// POST /api/v1/graphql/mocks - Create a new GraphQL mock
pub async fn post_create_graphql_mock(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let Some(cache) = req
        .data::<Arc<MockersContext>>()
        .map(|ctx| ctx.graphql_cache.clone())
    else {
        return Ok(build_error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &MockersErrors::AddMockFailed.to_string(),
        ));
    };

    let body_bytes = req
        .body()
        .clone()
        .collect()
        .await
        .map(|buf| buf.to_bytes())
        .unwrap_or_default();

    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap_or_default();

    let Ok(params) = serde_json::from_str::<CreateGraphQLMockParams>(&body_str) else {
        return Ok(build_error_response(
            StatusCode::BAD_REQUEST,
            "Invalid request body",
        ));
    };

    if params.query.is_empty() {
        return Ok(build_error_response(
            StatusCode::BAD_REQUEST,
            "Query cannot be empty",
        ));
    }

    if params.path.is_empty() {
        return Ok(build_error_response(
            StatusCode::BAD_REQUEST,
            "Path cannot be empty",
        ));
    }

    // Compute hash from the query AST
    let query_hash = match compute_query_hash(&params.query, params.operation_name.as_deref()) {
        Ok(hash) => hash,
        Err(e) => {
            return Ok(build_error_response(
                StatusCode::BAD_REQUEST,
                &format!("Failed to parse GraphQL query: {}", e),
            ));
        }
    };

    let key = GraphQLMockKey::new(&params.path, query_hash);

    // Extract operation info for metadata
    let op_info = crate::libs::graphql_cache::extract_operation_info(
        &params.query,
        params.operation_name.as_deref(),
    )
    .ok();

    let response = GraphQLCachedResponse::new(
        params.status_code,
        params.delay_ms,
        params.headers,
        params.data,
        params.errors,
    )
    .with_metadata(
        params.query.clone(),
        op_info.as_ref().and_then(|i| i.operation_name.clone()),
        op_info
            .as_ref()
            .map(|i| i.operation_type)
            .unwrap_or_default(),
    );

    let path = params.path.clone();

    tokio::spawn(async move {
        cache.upsert(key, response).await;
    });

    let result = CreateMockResponse {
        message: format!("GraphQL mock created at path '{}' with hash {:016x}", path, query_hash),
        query_hash: format!("{:016x}", query_hash),
    };

    let json = serde_json::to_string(&Ok::<_, String>(result).to_protocol());
    let bytes = json.map(Bytes::from).unwrap_or_default();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, APPLICATION_JSON)
        .body(Full::new(bytes))
        .unwrap_or_default())
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
