use crate::libs::cached_response::CachedResponse;
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
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

pub async fn get_all_mocks(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    let response_cache = req
        .data::<Arc<MockersContext>>()
        .map(|c| c.clone().response_cache.clone());

    let Some(cache) = response_cache else {
        return Ok(GET_ALL_MOCKS_FAILED.clone());
    };

    let all_mocks = cache.get_all().await;
    let body = Ok::<HashMap<String, HashMap<String, CachedResponse>>, &str>(all_mocks)
        .to_protocol()
        .to_string()
        .into();
    let response = Response::ok()
        .content_type_json()
        .body(body)
        .unwrap_or_default();
    Ok(response)
}

static GET_ALL_MOCKS_FAILED: LazyLock<Response<Full<Bytes>>> = LazyLock::new(|| {
    let body = Err::<&str, String>(MockersErrors::GetAllMocksFailed.to_string())
        .to_protocol()
        .to_string()
        .into();
    Response::internal_server_error()
        .content_type_json()
        .body(body)
        .unwrap_or_default()
});
