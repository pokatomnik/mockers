use std::convert::Infallible;
use std::sync::Arc;

use crate::controllers::api::admin_page_handler::admin_page_handler;
use crate::controllers::api::delete_remove_mock_by_path_and_method::delete_remove_mock_by_path_and_method;
use crate::controllers::api::delete_remove_mocks_by_path::delete_remove_mocks_by_path;
use crate::controllers::api::get_all_mocks::get_all_mocks;
use crate::controllers::api::get_mocks_by_path::get_all_mocks_by_path;
use crate::controllers::api::get_mocks_by_path_and_method::get_mocks_by_path_and_method;
use crate::controllers::api::graphql::delete_graphql_mock::{
    delete_graphql_mock, delete_graphql_mocks_by_path,
};
use crate::controllers::api::graphql::get_all_graphql_mocks::get_all_graphql_mocks;
use crate::controllers::api::graphql::get_graphql_mock::get_graphql_mock;
use crate::controllers::api::graphql::post_create_graphql_mock::post_create_graphql_mock;
use crate::controllers::api::post_create_mock::post_create_mock;
use crate::controllers::api::post_dump_mock::post_dump_mock;
use crate::controllers::error::error_handler;
use crate::controllers::graphql_handler::graphql_handler;
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
use crate::libs::graphql_cache::InMemoryGraphQLMocks;
use crate::libs::response_cache::InMemoryMocks;
use crate::middlewares::logger::logger;
use crate::server::mockers_context::MockersContext;
use crate::server::params::ServerParams;
use reqwest::Client;
use routerify_ng::Router;
use routerify_ng::{Middleware, RouteError};

pub fn admin_router(_params: &ServerParams) -> Result<Router<Infallible>, RouteError> {
    Router::builder()
        // TODO web app routes
        .get("/", admin_page_handler)
        // REST mock API routes
        .get("/api/v1/mocks", get_all_mocks)
        .get("/api/v1/mocks/:path_encoded", get_all_mocks_by_path)
        .get(
            "/api/v1/mocks/:path_encoded/:method",
            get_mocks_by_path_and_method,
        )
        .post("/api/v1/mocks", post_create_mock)
        .post("/api/v1/mocks/:path_encoded/:method/dump", post_dump_mock)
        .delete(
            "/api/v1/mocks/:path_encoded/:method",
            delete_remove_mock_by_path_and_method,
        )
        .delete("/api/v1/mocks/:path_encoded", delete_remove_mocks_by_path)
        // GraphQL mock API routes
        .get("/api/v1/graphql/mocks", get_all_graphql_mocks)
        .get(
            "/api/v1/graphql/mocks/:path_encoded/:query_hash",
            get_graphql_mock,
        )
        .post("/api/v1/graphql/mocks", post_create_graphql_mock)
        .delete(
            "/api/v1/graphql/mocks/:path_encoded/:query_hash",
            delete_graphql_mock,
        )
        .delete(
            "/api/v1/graphql/mocks/:path_encoded",
            delete_graphql_mocks_by_path,
        )
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
        .build()
}

pub fn mockers_router(params: &ServerParams) -> Result<Router<Infallible>, RouteError> {
    let context = Arc::new(MockersContext {
        client: Arc::new(Client::new()),
        server_params: params.clone(),
        response_cache: Arc::new(InMemoryMocks::create()),
        graphql_cache: Arc::new(InMemoryGraphQLMocks::new()),
    });

    let router_builder = {
        let mut router = Router::builder()
            .data(context)
            .middleware(Middleware::pre(logger));

        // Add admin routes if admin_base_url is configured
        if let Some((admin_base_url, admin_router)) =
            params.admin_base_url.clone().zip(admin_router(params).ok())
        {
            router = router.scope(admin_base_url, admin_router);
        }

        // Add GraphQL endpoint if configured
        if let Some(ref graphql_path) = params.graphql_path {
            router = router.post(graphql_path, graphql_handler);
        }

        router
    };

    router_builder
        .any(mock_handler)
        .err_handler_with_info(error_handler)
        .build()
}
