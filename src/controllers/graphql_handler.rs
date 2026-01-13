//! GraphQL mock request handler.
//!
//! Handles incoming GraphQL requests and returns mocked responses
//! based on operation name and type.

use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use http_body_util::{BodyExt, Full};
use hyper::header::CONTENT_TYPE;
use hyper::{body::Bytes, Method, Request, Response, StatusCode};
use routerify_ng::ext::RequestExt;
use tokio::time::sleep;

use crate::libs::graphql_cache::{
    GraphQLCachedResponse, GraphQLError, GraphQLPathNormalizer, GraphQLRequest,
};
use crate::server::mockers_context::MockersContext;
use crate::server::params::{DEFAULT_CORS_ENABLED, DEFAULT_MOCKS_RESPONSE_DELAY, DEFAULT_VERBOSE_ENABLED};
use crate::controllers::utils::add_headers;

const APPLICATION_JSON: &str = "application/json";

/// Main GraphQL mock handler
///
/// Handles POST requests to GraphQL endpoints and returns mocked responses
/// based on operation name and type matching.
pub async fn graphql_handler(req: Request<Full<Bytes>>) -> Result<Response<Full<Bytes>>, Infallible> {
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
            eprintln!("[GraphQL] Failed to parse request");
        }
        return Ok(build_graphql_error_response(
            StatusCode::BAD_REQUEST,
            "Invalid GraphQL request",
            cors,
        ));
    };

    // Extract operation info
    let operation_type = graphql_request.parse_operation_type();
    let operation_name = graphql_request.resolve_operation_name();

    if verbose {
        println!(
            "[GraphQL] Operation: {:?} {:?}",
            operation_type,
            operation_name.as_deref().unwrap_or("<anonymous>")
        );
    }

    // Operation name is required for mocking
    let Some(operation_name) = operation_name else {
        return Ok(build_graphql_error_response(
            StatusCode::BAD_REQUEST,
            "Operation name is required for GraphQL mocking. \
             Provide it via 'operationName' field or in the query itself.",
            cors,
        ));
    };

    // Try to find mock in cache
    let graphql_cache = context.map(|ctx| ctx.graphql_cache.clone());

    if let Some(cache) = graphql_cache {
        let normalized_path = path.normalize_graphql_path();

        if let Some(cached_response) = cache
            .find_mock(&normalized_path, &operation_name, operation_type)
            .await
        {
            if verbose {
                println!(
                    "[GraphQL] Found mock for {} {} at {}",
                    operation_type, operation_name, normalized_path
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

            return Ok(build_cached_response(&cached_response, cors));
        }
    }

    // No mock found
    if verbose {
        println!(
            "[GraphQL] No mock found for {} {} at {}",
            operation_type, operation_name, path
        );
    }

    Ok(build_graphql_error_response(
        StatusCode::NOT_FOUND,
        &format!(
            "No mock found for GraphQL operation '{}' (type: {})",
            operation_name, operation_type
        ),
        cors,
    ))
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

    builder = add_headers(builder, cors, &std::collections::HashMap::new());

    builder
        .body(Full::new(Bytes::from(body_bytes)))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_graphql_error_response() {
        let response = build_graphql_error_response(
            StatusCode::BAD_REQUEST,
            "Test error",
            false,
        );
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response.body();
        assert!(!body.is_empty());
    }
}
