use std::sync::Arc;

use crate::controllers::api::admin_page_handler::admin_page_handler;
use crate::controllers::api::get_all_mocks::get_all_mocks;
use crate::controllers::api::get_mock_config::get_mock_config;
use crate::controllers::api::handle_options::handle_options;
use crate::controllers::error::error_handler;
use crate::controllers::mock_handler::mock_handler;
use crate::controllers::swagger::get_favicon_16::get_favicon_16;
use crate::controllers::swagger::get_favicon_32::get_favicon_32;
use crate::controllers::swagger::get_index_css::get_index_css;
use crate::controllers::swagger::get_mockers_yaml::get_mockers_yaml;
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

pub fn admin_router(_params: &ServerParams) -> Result<Router<MockersRouteError>, RouteError> {
    Router::builder()
        // TODO web app routes
        .get("/", admin_page_handler)
        // REST api routes
        .post("/api/v2/mocks", get_all_mocks)
        .post("/api/v2/mocks/config", get_mock_config)
        // Swagger UI routes
        .get("/swagger", get_swagger_html)
        .get("/swagger/swagger-ui.css", get_swagger_ui_css)
        .get("/swagger/index.css", get_index_css)
        .get("/swagger/favicon-32x32.png", get_favicon_32)
        .get("/swagger/favicon-16x16.png", get_favicon_16)
        .get("/swagger/swagger-ui-bundle.js", get_swagger_ui_bundle_js)
        .get(
            "/swagger/swagger-ui-standalone-preset.js",
            get_swagger_ui_standalone_preset,
        )
        .get(
            "/swagger/swagger-initializer.js",
            get_swagger_initializer_js,
        )
        .get("/swagger/swagger.yaml", get_mockers_yaml)
        .any(handle_options)
        .middleware(Middleware::post(admin_api_cors))
        .build()
}

pub async fn mockers_router(
    params: &ServerParams,
) -> Result<Router<MockersRouteError>, RouteError> {
    let router_builder = {
        let mut router = Router::builder().data(Arc::new(MockersContext {
            client: Arc::new(Client::new()),
            server_params: params.clone(),
        }));
        if let Some((admin_base_url, admin_router)) = params
            .admin_base_url()
            .await
            .zip(admin_router(&params).ok())
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
