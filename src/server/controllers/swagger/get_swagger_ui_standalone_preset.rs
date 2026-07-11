use crate::server::controllers::swagger::static_files::SWAGGER_UI_STANDALONE_PRESET_JS;
use crate::libs::hyper_response_ext::HyperWellKnownResponses;
use crate::libs::response_builder_ext::ResponseBuilderExt;
use crate::server::route_error::MockersRouteError;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};

pub async fn get_swagger_ui_standalone_preset(
    _: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    let response = Response::ok()
        .content_type_js()
        .body(SWAGGER_UI_STANDALONE_PRESET_JS.into())
        .unwrap_or_default();
    Ok(response)
}
