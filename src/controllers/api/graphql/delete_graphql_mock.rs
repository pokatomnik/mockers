//! Delete GraphQL mock endpoint.

use std::convert::Infallible;
use std::sync::Arc;

use http_body_util::Full;
use hyper::header::CONTENT_TYPE;
use hyper::{body::Bytes, Request, Response, StatusCode};
use mimetype_detector::APPLICATION_JSON;
use routerify_ng::ext::RequestExt;

use crate::libs::graphql_cache::{GraphQLMockKey, GraphQLOperationType};
use crate::libs::mockers_errors::MockersErrors;
use crate::libs::protocol_result::ProtocolResultConverter;
use crate::libs::response_cache::PathDecoder;
use crate::server::mockers_context::MockersContext;

/// DELETE /api/v1/graphql/mocks/:path_encoded/:operation_name/:operation_type
/// Delete a specific GraphQL mock
pub async fn delete_graphql_mock(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let graphql_cache = req
        .data::<Arc<MockersContext>>()
        .map(|ctx| ctx.graphql_cache.clone());

    let path_encoded = req.param("path_encoded").map(|s| s.to_string());
    let operation_name = req.param("operation_name").map(|s| s.to_string());
    let operation_type_str = req.param("operation_type").map(|s| s.to_string());

    // Validate parameters
    let (path_encoded, operation_name, operation_type_str) =
        match (path_encoded, operation_name, operation_type_str) {
            (Some(p), Some(o), Some(t)) => (p, o, t),
            _ => {
                return Ok(build_error_response(
                    StatusCode::BAD_REQUEST,
                    "Missing required parameters",
                ));
            }
        };

    // Decode path
    let path = match path_encoded.decode_path() {
        Ok(p) => p,
        Err(_) => {
            return Ok(build_error_response(
                StatusCode::BAD_REQUEST,
                "Invalid path encoding",
            ));
        }
    };

    // Parse operation type
    let operation_type: GraphQLOperationType = match operation_type_str.parse() {
        Ok(t) => t,
        Err(_) => {
            return Ok(build_error_response(
                StatusCode::BAD_REQUEST,
                "Invalid operation type. Must be 'query', 'mutation', or 'subscription'",
            ));
        }
    };

    match graphql_cache {
        Some(cache) => {
            let key = GraphQLMockKey::new(&path, &operation_name, operation_type);

            match cache.remove(&key).await {
                Some(_) => {
                    let json = serde_json::to_string(
                        &Ok::<_, String>(format!(
                            "GraphQL mock deleted for operation '{}' ({}) at path '{}'",
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
                    StatusCode::NOT_FOUND,
                    &MockersErrors::NoSuchMock.to_string(),
                )),
            }
        }
        None => Ok(build_error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &MockersErrors::RemoveMocksFailedByPathAndMethod.to_string(),
        )),
    }
}

/// DELETE /api/v1/graphql/mocks/:path_encoded
/// Delete all GraphQL mocks for a path
pub async fn delete_graphql_mocks_by_path(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let graphql_cache = req
        .data::<Arc<MockersContext>>()
        .map(|ctx| ctx.graphql_cache.clone());

    let path_encoded = req.param("path_encoded").map(|s| s.to_string());

    let path_encoded = match path_encoded {
        Some(p) => p,
        None => {
            return Ok(build_error_response(
                StatusCode::BAD_REQUEST,
                "Missing path parameter",
            ));
        }
    };

    // Decode path
    let path = match path_encoded.decode_path() {
        Ok(p) => p,
        Err(_) => {
            return Ok(build_error_response(
                StatusCode::BAD_REQUEST,
                "Invalid path encoding",
            ));
        }
    };

    match graphql_cache {
        Some(cache) => {
            let count = cache.remove_by_path(&path).await;

            let json = serde_json::to_string(
                &Ok::<_, String>(format!(
                    "Deleted {} GraphQL mock(s) for path '{}'",
                    count, path
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
            &MockersErrors::RemoveMocksFailedByPath.to_string(),
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
