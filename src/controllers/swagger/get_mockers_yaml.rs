use crate::controllers::swagger::static_files::SWAGGER_YAML;
use crate::libs::response_builder_ext::ResponseBuilderExt;
use crate::libs::response_ext::WellKnownResponses;
use crate::libs::yaml_builder::YamlBuilderCreate;
use crate::server::mockers_context::MockersContext;
use crate::server::route_error::MockersRouteError;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};
use routerify_ng::ext::RequestExt;
use std::sync::Arc;

pub async fn get_mockers_yaml(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    let admin_base_url = req
        .data::<Arc<MockersContext>>()
        .and_then(|ctx| ctx.clone().server_params.admin_base_url().map(String::from));
    let yaml_contents: YamlBuilderCreate = SWAGGER_YAML.try_into();

    let response = yaml_contents
        .ok()
        .zip(admin_base_url)
        .map(|(mut yb, url)| {
            yb.add_servers([url]);
            yb.to_string()
        })
        .map(|yaml_contents| {
            Response::ok()
                .content_type_yaml()
                .body(yaml_contents.into())
                .unwrap_or_default()
        })
        .unwrap_or_else(|| Response::not_found().empty_body().unwrap_or_default());

    Ok(response)
}
