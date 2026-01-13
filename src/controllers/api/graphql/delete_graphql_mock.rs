//! Delete GraphQL mock endpoint.

use std::convert::Infallible;
use std::sync::Arc;

use http_body_util::Full;
use hyper::header::CONTENT_TYPE;
use hyper::{body::Bytes, Request, Response, StatusCode};
use mimetype_detector::APPLICATION_JSON;
use routerify_ng::ext::RequestExt;

use crate::libs::graphql_cache::GraphQLMockKey;
use crate::libs::mockers_errors::MockersErrors;
use crate::libs::protocol_result::ProtocolResultConverter;
use crate::libs::response_cache::PathDecoder;
use crate::server::mockers_context::MockersContext;

/// DELETE /api/v1/graphql/mocks/:path_encoded/:query_hash
/// Delete a specific GraphQL mock by path and hash
pub async fn delete_graphql_mock(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let Some(cache) = req
        .data::<Arc<MockersContext>>()
        .map(|ctx| ctx.graphql_cache.clone())
    else {
        return Ok(build_error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &MockersErrors::RemoveMocksFailedByPathAndMethod.to_string(),
        ));
    };

    let (Some(path_encoded), Some(query_hash_str)) =
        (req.param("path_encoded"), req.param("query_hash"))
    else {
        return Ok(build_error_response(
            StatusCode::BAD_REQUEST,
            "Missing required parameters",
        ));
    };

    let Ok(path) = path_encoded.to_string().decode_path() else {
        return Ok(build_error_response(
            StatusCode::BAD_REQUEST,
            "Invalid path encoding",
        ));
    };

    let Ok(query_hash) = u64::from_str_radix(query_hash_str, 16) else {
        return Ok(build_error_response(
            StatusCode::BAD_REQUEST,
            "Invalid query hash. Must be a 16-character hex string",
        ));
    };

    let key = GraphQLMockKey::new(&path, query_hash);

    let Some(_) = cache.remove(&key).await else {
        return Ok(build_error_response(
            StatusCode::NOT_FOUND,
            &MockersErrors::NoSuchMock.to_string(),
        ));
    };

    let json = serde_json::to_string(
        &Ok::<_, String>(format!(
            "GraphQL mock deleted for path '{}' with hash {:016x}",
            path, query_hash
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

/// DELETE /api/v1/graphql/mocks/:path_encoded
/// Delete all GraphQL mocks for a path
pub async fn delete_graphql_mocks_by_path(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let Some(cache) = req
        .data::<Arc<MockersContext>>()
        .map(|ctx| ctx.graphql_cache.clone())
    else {
        return Ok(build_error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &MockersErrors::RemoveMocksFailedByPath.to_string(),
        ));
    };

    let Some(path_encoded) = req.param("path_encoded") else {
        return Ok(build_error_response(
            StatusCode::BAD_REQUEST,
            "Missing path parameter",
        ));
    };

    let Ok(path) = path_encoded.to_string().decode_path() else {
        return Ok(build_error_response(
            StatusCode::BAD_REQUEST,
            "Invalid path encoding",
        ));
    };

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

fn build_error_response(status: StatusCode, error: &str) -> Response<Full<Bytes>> {
    let json = serde_json::to_string(&Err::<String, _>(error.to_string()).to_protocol());
    let bytes = json.map(Bytes::from).unwrap_or_default();

    Response::builder()
        .status(status)
        .header(CONTENT_TYPE, APPLICATION_JSON)
        .body(Full::new(bytes))
        .unwrap_or_default()
}
