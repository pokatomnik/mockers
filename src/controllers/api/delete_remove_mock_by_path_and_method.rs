use crate::libs::in_memory_mocks::{MethodNormalizer, PathDecoder, PathNormalizer};
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

pub async fn delete_remove_mock_by_path_and_method(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    let path = req
        .params()
        .get("path_encoded")
        .and_then(|str| str.decode_path().ok())
        .unwrap_or_default()
        .normalize_path();

    let method = req
        .params()
        .get("method")
        .map(|v| v.to_owned())
        .unwrap_or_default()
        .normalize_method();

    let response_cache = req
        .data::<Arc<MockersContext>>()
        .map(|c| c.clone().response_cache.clone());

    let Some(cache) = response_cache else {
        return Ok(REMOVE_MOCKS_FAILED_BY_PATH_AND_METHOD.clone());
    };

    cache.remove_by_path_and_method(&path, &method).await;

    let message = format!(
        "Removed mocks for path: '{}' and method: '{}'",
        path, method
    );
    let body = Ok::<String, &str>(message).to_protocol().to_string().into();
    let response = Response::ok()
        .content_type_json()
        .body(body)
        .unwrap_or_default();
    Ok(response)
}

static REMOVE_MOCKS_FAILED_BY_PATH_AND_METHOD: LazyLock<Response<Full<Bytes>>> =
    LazyLock::new(|| {
        let body = Err::<&str, String>(MockersErrors::RemoveMocksFailedByPathAndMethod.to_string())
            .to_protocol()
            .to_string()
            .into();
        Response::internal_server_error()
            .content_type_json()
            .body(body)
            .unwrap_or_default()
    });
