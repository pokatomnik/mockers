use crate::libs::in_memory_mocks::{PathDecoder, PathNormalizer};
use crate::libs::mockers_errors::MockersErrors;
use crate::libs::protocol_result::ProtocolResultConverter;
use crate::libs::response_builder_ext::ResponseBuilderExt;
use crate::libs::response_ext::WellKnownResponses;
use crate::server::mockers_context::MockersContext;
use crate::server::route_error::MockersRouteError;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};
use routerify_ng::ext::RequestExt;
use std::sync::{Arc, LazyLock};

pub async fn delete_remove_mocks_by_path(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    let path = req
        .params()
        .get("path_encoded")
        .and_then(|str| str.decode_path().ok())
        .unwrap_or_default()
        .normalize_path();

    let response_cache = req
        .data::<Arc<MockersContext>>()
        .map(|c| c.clone().response_cache.clone());

    let Some(cache) = response_cache else {
        return Ok(REMOVE_MOCKS_FAILED_BY_PATH.clone());
    };

    cache.remove_by_path(&path).await;

    let message = format!("Removed mocks for path: '{}'", &path).to_string();
    let body = Ok::<String, &str>(message).to_protocol().to_string().into();
    let response = Response::ok()
        .content_type_json()
        .body(body)
        .unwrap_or_default();
    Ok(response)
}

static REMOVE_MOCKS_FAILED_BY_PATH: LazyLock<Response<Full<Bytes>>> = LazyLock::new(|| {
    let body = Err::<&str, String>(MockersErrors::RemoveMocksFailedByPath.to_string())
        .to_protocol()
        .to_string()
        .into();
    Response::internal_server_error()
        .content_type_json()
        .body(body)
        .unwrap_or_default()
});
