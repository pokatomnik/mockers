use std::sync::Arc;

use crate::controllers::api::admin_page_handler::admin_page_handler;
use crate::controllers::api::create_mock::create_mock;
use crate::controllers::api::delete_mock::delete_mock;
use crate::controllers::api::get_all_mocks::get_all_mocks;
use crate::controllers::api::get_mock_config::get_mock_config;
use crate::controllers::api::handle_options::handle_options;
use crate::controllers::error::error_handler;
use crate::controllers::mock_handler::mock_handler;
use crate::controllers::swagger::get_favicon_16::get_favicon_16;
use crate::controllers::swagger::get_favicon_32::get_favicon_32;
use crate::controllers::swagger::get_index_css::get_index_css;
use crate::controllers::swagger::get_mockers_json::get_mockers_json;
use crate::controllers::swagger::get_swagger_html::get_swagger_html;
use crate::controllers::swagger::get_swagger_initializer_js::get_swagger_initializer_js;
use crate::controllers::swagger::get_swagger_ui_bundle_js::get_swagger_ui_bundle_js;
use crate::controllers::swagger::get_swagger_ui_css::get_swagger_ui_css;
use crate::controllers::swagger::get_swagger_ui_standalone_preset::get_swagger_ui_standalone_preset;
use crate::middlewares::admin_api_cors::admin_api_cors;
use crate::middlewares::check_request::check_request;
use crate::middlewares::logger::logger;
use crate::server::mockers_context::MockersContext;
use crate::server::params::ServerParams;
use crate::server::route_error::MockersRouteError;
use reqwest::Client;
use routerify_ng::Router;
use routerify_ng::{Middleware, RouteError};

pub fn admin_router(
    swagger_base_path: &'static str,
) -> Result<Router<MockersRouteError>, RouteError> {
    Router::builder()
        // TODO web app routes
        .get("/", admin_page_handler)
        // REST api routes
        .post("/api/v2/mocks", get_all_mocks)
        .post("/api/v2/mocks/config", get_mock_config)
        .post("/api/v2/mocks/create", create_mock)
        .post("/api/v2/mocks/delete", delete_mock)
        // Swagger UI routes
        .get(format!("{}", swagger_base_path), get_swagger_html)
        .get(
            format!("{}/swagger-ui.css", swagger_base_path),
            get_swagger_ui_css,
        )
        .get(format!("{}/index.css", swagger_base_path), get_index_css)
        .get(
            format!("{}/favicon-32x32.png", swagger_base_path),
            get_favicon_32,
        )
        .get(
            format!("{}/favicon-16x16.png", swagger_base_path),
            get_favicon_16,
        )
        .get(
            format!("{}/swagger-ui-bundle.js", swagger_base_path),
            get_swagger_ui_bundle_js,
        )
        .get(
            format!("{}/swagger-ui-standalone-preset.js", swagger_base_path),
            get_swagger_ui_standalone_preset,
        )
        .get(
            format!("{}/swagger-initializer.js", swagger_base_path),
            get_swagger_initializer_js,
        )
        .get(
            format!("{}/swagger.json", swagger_base_path),
            get_mockers_json,
        )
        .any(handle_options)
        .middleware(Middleware::post(admin_api_cors))
        .build()
}

pub async fn mockers_router(
    params: &ServerParams,
    swagger_base_path: &'static str,
) -> Result<Router<MockersRouteError>, RouteError> {
    let router_builder = {
        let mut router = Router::builder().data(Arc::new(MockersContext {
            client: Arc::new(Client::new()),
            server_params: params.clone(),
        }));
        if let Some((admin_base_url, admin_router)) = params
            .admin_base_url()
            .await
            .zip(admin_router(swagger_base_path).ok())
        {
            router = router.scope(admin_base_url, admin_router)
        }
        router
    };

    router_builder
        .middleware(Middleware::pre(logger))
        .middleware(Middleware::pre(check_request))
        .any(mock_handler)
        .err_handler_with_info(error_handler)
        .build()
}
