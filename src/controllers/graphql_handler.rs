//! GraphQL mock request handler.
//!
//! Handles incoming GraphQL requests and returns mocked responses
//! based on AST hash matching.

use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use http_body_util::{BodyExt, Full};
use hyper::header::CONTENT_TYPE;
use hyper::{body::Bytes, Method, Request, Response, StatusCode};
use routerify_ng::ext::RequestExt;
use tokio::time::sleep;

use crate::controllers::utils::add_headers;
use crate::libs::graphql_cache::{
    GraphQLCachedResponse, GraphQLError, GraphQLPathNormalizer, GraphQLRequest,
};
use crate::server::mockers_context::MockersContext;
use crate::server::params::{
    DEFAULT_CORS_ENABLED, DEFAULT_MOCKS_RESPONSE_DELAY, DEFAULT_VERBOSE_ENABLED,
};

const APPLICATION_JSON: &str = "application/json";

/// Main GraphQL mock handler
///
/// Handles POST requests to GraphQL endpoints and returns mocked responses
/// by computing AST hash and finding matching mock.
pub async fn graphql_handler(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let context = req.data::<Arc<MockersContext>>();
    let verbose = context
        .map(|ctx| ctx.server_params.verbose)
        .unwrap_or(DEFAULT_VERBOSE_ENABLED);
    let cors = context
        .map(|ctx| ctx.server_params.cors)
        .unwrap_or(DEFAULT_CORS_ENABLED);
    let global_delay_ms = context
        .map(|ctx| ctx.server_params.delay_ms)
        .unwrap_or(DEFAULT_MOCKS_RESPONSE_DELAY);

    let path = req.uri().path().to_string();

    if verbose {
        println!("[GraphQL] Incoming request to: {}", &path);
    }

    // Only handle POST requests for GraphQL
    if req.method() != Method::POST {
        return Ok(build_graphql_error_response(
            StatusCode::METHOD_NOT_ALLOWED,
            "GraphQL endpoint only accepts POST requests",
            cors,
        ));
    }

    // Parse request body
    let body_bytes = req
        .body()
        .clone()
        .collect()
        .await
        .map(|buf| buf.to_bytes())
        .unwrap_or_default();

    let Ok(body_str) = String::from_utf8(body_bytes.to_vec()) else {
        return Ok(build_graphql_error_response(
            StatusCode::BAD_REQUEST,
            "Invalid UTF-8 in request body",
            cors,
        ));
    };

    // Parse GraphQL request
    let Ok(graphql_request) = serde_json::from_str::<GraphQLRequest>(&body_str) else {
        if verbose {
            eprintln!("[GraphQL] Failed to parse JSON request");
        }
        return Ok(build_graphql_error_response(
            StatusCode::BAD_REQUEST,
            "Invalid GraphQL request JSON",
            cors,
        ));
    };

    // Compute query hash from AST
    let query_hash = match graphql_request.compute_query_hash() {
        Ok(hash) => hash,
        Err(e) => {
            if verbose {
                eprintln!("[GraphQL] Failed to parse query: {}", e);
            }
            return Ok(build_graphql_error_response(
                StatusCode::BAD_REQUEST,
                &format!("Failed to parse GraphQL query: {}", e),
                cors,
            ));
        }
    };

    // Extract operation info for logging
    let operation_info = graphql_request.extract_operation_info().ok();

    if verbose {
        if let Some(ref info) = operation_info {
            println!(
                "[GraphQL] Operation: {} {:?} (hash: {:016x})",
                info.operation_type,
                info.operation_name.as_deref().unwrap_or("<anonymous>"),
                info.query_hash
            );
        } else {
            println!("[GraphQL] Query hash: {:016x}", query_hash);
        }
    }

    // Try to find mock in cache by hash
    let Some(cache) = context.map(|ctx| ctx.graphql_cache.clone()) else {
        return Ok(build_graphql_error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "GraphQL cache not available",
            cors,
        ));
    };

    let normalized_path = path.normalize_graphql_path();

    let Some(cached_response) = cache.find_by_hash(&normalized_path, query_hash).await else {
        if verbose {
            println!(
                "[GraphQL] No mock found for hash {:016x} at {}",
                query_hash, normalized_path
            );
        }
        return Ok(build_graphql_error_response(
            StatusCode::NOT_FOUND,
            &format!(
                "No mock found for GraphQL query (hash: {:016x})",
                query_hash
            ),
            cors,
        ));
    };

    if verbose {
        println!(
            "[GraphQL] Found mock for hash {:016x} at {}",
            query_hash, normalized_path
        );
    }

    // Apply delay
    let delay = if cached_response.delay_ms > 0 {
        cached_response.delay_ms
    } else {
        global_delay_ms
    };

    if delay > 0 {
        sleep(Duration::from_millis(delay)).await;
    }

    Ok(build_cached_response(&cached_response, cors))
}

/// Build response from cached GraphQL mock
fn build_cached_response(cached: &GraphQLCachedResponse, cors: bool) -> Response<Full<Bytes>> {
    let body = cached.to_response_body();
    let body_bytes = serde_json::to_vec(&body).unwrap_or_default();

    let status = StatusCode::from_u16(cached.status_code).unwrap_or(StatusCode::OK);

    let mut builder = Response::builder()
        .status(status)
        .header(CONTENT_TYPE, APPLICATION_JSON);

    builder = add_headers(builder, cors, &cached.headers);

    builder
        .body(Full::new(Bytes::from(body_bytes)))
        .unwrap_or_default()
}

/// Build GraphQL error response following the spec
fn build_graphql_error_response(
    status: StatusCode,
    message: &str,
    cors: bool,
) -> Response<Full<Bytes>> {
    let error = GraphQLError::new(message);
    let body = serde_json::json!({
        "data": null,
        "errors": [error]
    });
    let body_bytes = serde_json::to_vec(&body).unwrap_or_default();

    let mut builder = Response::builder()
        .status(status)
        .header(CONTENT_TYPE, APPLICATION_JSON);

    builder = add_headers(builder, cors, &HashMap::new());

    builder
        .body(Full::new(Bytes::from(body_bytes)))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_build_graphql_error_response() {
        let response =
            build_graphql_error_response(StatusCode::BAD_REQUEST, "Test error", false);
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        // Verify the body contains expected JSON structure
        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body_str = String::from_utf8_lossy(&body_bytes);
        assert!(body_str.contains("errors"));
        assert!(body_str.contains("Test error"));
    }
}
