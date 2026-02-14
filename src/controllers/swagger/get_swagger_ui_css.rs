use crate::controllers::swagger::static_files::SWAGGER_UI_CSS;
use crate::libs::response_builder_ext::ResponseBuilderExt;
use crate::libs::response_ext::WellKnownResponses;
use crate::server::route_error::MockersRouteError;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};

pub async fn get_swagger_ui_css(
    _: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    let response = Response::ok()
        .content_type_css()
        .body(SWAGGER_UI_CSS.into())
        .unwrap_or_default();

    Ok(response)
}
