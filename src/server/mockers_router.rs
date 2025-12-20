use std::convert::Infallible;
use std::sync::Arc;

use crate::controllers::api::admin_page_handler::admin_page_handler;
use crate::controllers::api::delete_remove_mock_by_path_and_method::delete_remove_mock_by_path_and_method;
use crate::controllers::api::delete_remove_mocks_by_path::delete_remove_mocks_by_path;
use crate::controllers::api::get_all_mocks::get_all_mocks;
use crate::controllers::api::get_mocks_by_path::get_all_mocks_by_path;
use crate::controllers::api::get_mocks_by_path_and_method::get_mocks_by_path_and_method;
use crate::controllers::api::post_create_mock::post_create_mock;
use crate::controllers::error::error_handler;
use crate::controllers::mock_handler::mock_handler;
use crate::controllers::swagger::get_favicon_16::get_favicon_16;
use crate::controllers::swagger::get_favicon_32::get_favicon_32;
use crate::controllers::swagger::get_index_css::get_index_css;
use crate::controllers::swagger::get_swagger_html::get_swagger_html;
use crate::controllers::swagger::get_swagger_initializer_js::get_swagger_initializer_js;
use crate::controllers::swagger::get_swagger_ui_bundle_js::get_swagger_ui_bundle_js;
use crate::controllers::swagger::get_swagger_ui_css::get_swagger_ui_css;
use crate::controllers::swagger::get_swagger_ui_standalone_preset::get_swagger_ui_standalone_preset;
use crate::libs::response_cache::InMemoryMocks;
use crate::middlewares::logger::logger;
use crate::server::mockers_context::MockersContext;
use crate::server::params::ServerParams;
use reqwest::Client;
use routerify_ng::Middleware;
use routerify_ng::Router;

pub fn admin_router(_params: &ServerParams) -> Router<Infallible> {
    Router::builder()
        // TODO web app routes
        .get("/", admin_page_handler)
        // REST api routes
        .get("/api/v1/mocks", get_all_mocks)
        .get("/api/v1/mocks/:path_encoded", get_all_mocks_by_path)
        .get(
            "/api/v1/mocks/:path_encoded/:method",
            get_mocks_by_path_and_method,
        )
        .post("/api/v1/mocks", post_create_mock)
        .delete(
            "/api/v1/mocks/:path_encoded/:method",
            delete_remove_mock_by_path_and_method,
        )
        .delete("/api/v1/mocks/:path_encoded", delete_remove_mocks_by_path)
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
        .build()
        .unwrap()
}

pub fn mockers_router(params: &ServerParams) -> Router<Infallible> {
    let router_builder = {
        let mut router = Router::builder()
            .data(Arc::new(MockersContext {
                client: Arc::new(Client::new()),
                server_params: params.clone(),
                response_cache: Arc::new(InMemoryMocks::create()),
            }))
            .middleware(Middleware::pre(logger));
        if let Some(admin_base_url) = &params.admin_base_url {
            router = router.scope(admin_base_url, admin_router(&params))
        }
        router
    };

    return router_builder
        .any(mock_handler)
        .err_handler_with_info(error_handler)
        .build()
        .unwrap();
}
