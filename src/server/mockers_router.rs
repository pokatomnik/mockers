use std::convert::Infallible;
use std::sync::Arc;

use reqwest::Client;
use routerify_ng::Middleware;
use routerify_ng::Router;
use crate::controllers::api::admin_page_handler::admin_page_handler;
use crate::controllers::api::get_all_mocks::get_all_mocks;
use crate::controllers::api::get_mocks_by_path::get_all_mocks_by_path;
use crate::controllers::api::post_create_mock::post_create_mock;
use crate::controllers::error::error_handler;
use crate::controllers::mocks::mock_handler::mock_handler;
use crate::libs::response_cache::InMemoryMocks;
use crate::middlewares::logger::logger;
use crate::server::mockers_context::MockersContext;
use crate::server::params::ServerParams;

pub fn admin_router(_params: &ServerParams) -> Router<Infallible> {
    return Router::builder()
        .get("/", admin_page_handler)
        .get("/api/v1/mocks", get_all_mocks)
        .get("/api/v1/mocks/:path_encoded", get_all_mocks_by_path)
        .post("/api/v1/mocks", post_create_mock)
        .build()
        .unwrap();
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
