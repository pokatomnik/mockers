use crate::controllers::swagger::static_files::SWAGGER_YAML;
use crate::server::mockers_context::MockersContext;
use crate::server::route_error::MockersRouteError;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::CONTENT_TYPE;
use hyper::{Request, Response};
use reqwest::StatusCode;
use routerify_ng::ext::RequestExt;
use std::sync::Arc;

pub async fn get_mockers_yaml(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    let admin_base_url = req
        .data::<Arc<MockersContext>>()
        .and_then(|ctx| ctx.clone().server_params.admin_base_url.clone());
    let yaml_string = str::from_utf8(SWAGGER_YAML).ok();

    let yaml_contents = yaml_string
        .zip(admin_base_url)
        .map(|(source_yaml, admin_url)| {
            let servers_entry_main = "servers:";
            let admin_base_url_entry = format!("  - url: \"{}\"", admin_url);
            format!(
                "{}\n{}\n{}",
                source_yaml, servers_entry_main, admin_base_url_entry
            )
        });

    let status_code = yaml_contents
        .as_ref()
        .map(|_| StatusCode::OK)
        .unwrap_or(StatusCode::NOT_FOUND);
    let bytes = yaml_contents
        .map(|str| Bytes::from(str))
        .unwrap_or(Bytes::new());

    Ok(Response::builder()
        .status(status_code)
        .header(CONTENT_TYPE, "text/yaml")
        .body(Full::from(bytes))
        .unwrap_or(Response::default()))
}
