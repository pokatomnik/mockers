//! Get specific GraphQL mock endpoint.

use std::convert::Infallible;
use std::sync::Arc;

use http_body_util::Full;
use hyper::header::CONTENT_TYPE;
use hyper::{body::Bytes, Request, Response, StatusCode};
use mimetype_detector::APPLICATION_JSON;
use routerify_ng::ext::RequestExt;

use crate::controllers::api::graphql::get_all_graphql_mocks::GraphQLMockResponse;
use crate::libs::graphql_cache::GraphQLMockKey;
use crate::libs::mockers_errors::MockersErrors;
use crate::libs::protocol_result::ProtocolResultConverter;
use crate::libs::response_cache::PathDecoder;
use crate::server::mockers_context::MockersContext;

/// GET /api/v1/graphql/mocks/:path_encoded/:query_hash
/// Get a specific GraphQL mock by path and hash
pub async fn get_graphql_mock(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let Some(cache) = req
        .data::<Arc<MockersContext>>()
        .map(|ctx| ctx.graphql_cache.clone())
    else {
        return Ok(build_error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &MockersErrors::GetMocksByPathAndMethodFailed.to_string(),
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

    let Some(response) = cache.get(&key).await else {
        return Ok(build_error_response(
            StatusCode::NOT_FOUND,
            &MockersErrors::NoSuchMock.to_string(),
        ));
    };

    let mock_response = GraphQLMockResponse::from_cached(path, query_hash, &response);

    let json = serde_json::to_string(&Ok::<_, String>(mock_response).to_protocol());
    let bytes = json.map(Bytes::from).unwrap_or_default();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, APPLICATION_JSON)
        .body(Full::new(bytes))
        .unwrap_or_default())
}

fn build_error_response(status: StatusCode, error: &str) -> Response<Full<Bytes>> {
    let json =
        serde_json::to_string(&Err::<GraphQLMockResponse, _>(error.to_string()).to_protocol());
    let bytes = json.map(Bytes::from).unwrap_or_default();

    Response::builder()
        .status(status)
        .header(CONTENT_TYPE, APPLICATION_JSON)
        .body(Full::new(bytes))
        .unwrap_or_default()
}
