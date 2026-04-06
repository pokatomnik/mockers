use crate::controllers::swagger::static_files::SWAGGER_JSON;
use crate::libs::hyper_response_ext::HyperWellKnownResponses;
use crate::libs::response_builder_ext::ResponseBuilderExt;
use crate::server::mockers_context::MockersContext;
use crate::server::params::ServerParams;
use crate::server::route_error::MockersRouteError;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};
use routerify_ng::ext::RequestExt;
use serde_json::json;
use std::sync::{Arc, LazyLock};

pub async fn get_mockers_yaml(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    let Some(server_params) = (match req.data::<Arc<MockersContext>>() {
        None => None,
        Some(ctx) => Some(ctx.server_params.clone()),
    }) else {
        return Ok(OPENAPI_FULFILL_JSON_ERROR.clone());
    };

    let Ok(mut openapi_specs) = serde_json::from_str::<serde_json::Value>(SWAGGER_JSON) else {
        return Ok(OPENAPI_FULFILL_JSON_ERROR.clone());
    };

    if postprocess_openapi(&mut openapi_specs, &server_params)
        .await
        .is_err()
    {
        return Ok(OPENAPI_FULFILL_JSON_ERROR.clone());
    }

    let Ok(result_json) = serde_json::to_string_pretty(&openapi_specs) else {
        return Ok(OPENAPI_FULFILL_JSON_ERROR.clone());
    };

    let response = Response::ok()
        .content_type_json()
        .body(result_json.into())
        .unwrap_or_default();

    Ok(response)
}

async fn postprocess_openapi(
    specs: &mut serde_json::Value,
    server_params: &ServerParams,
) -> anyhow::Result<()> {
    let specs = specs.as_object_mut();
    let admin_base_url = server_params.admin_base_url().await;

    if let Some((specs, admin_base_url)) = specs.zip(admin_base_url) {
        specs.insert("servers".to_string(), json!([{ "url": admin_base_url }]));
    }

    Ok(())
}

static OPENAPI_FULFILL_JSON_ERROR: LazyLock<Response<Full<Bytes>>> = LazyLock::new(|| {
    Response::internal_server_error()
        .content_type_octet_stream()
        .body(Full::default())
        .unwrap_or_default()
});
