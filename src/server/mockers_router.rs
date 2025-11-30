use std::convert::Infallible;
use std::sync::Arc;

use reqwest::Client;
use routerify_ng::Middleware;
use routerify_ng::Router;

use crate::controllers::error::error_handler;
use crate::controllers::mock_handler::mock_handler;
use crate::middlewares::logger::logger;
use crate::server::mockers_context::MockersContext;
use crate::server::params::ServerParams;

pub fn mockers_router(params: &ServerParams) -> Router<Infallible> {
    let router = Router::builder()
        .data(Arc::new(MockersContext {
            client: Arc::new(Client::new()),
            server_params: params.clone(),
        }))
        .middleware(Middleware::pre(logger))
        .any(mock_handler)
        .err_handler_with_info(error_handler)
        .build()
        .unwrap();
    return router;
}
