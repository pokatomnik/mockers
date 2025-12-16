use crate::libs::response_cache::CachedResponse;
use crate::server::mockers_context::MockersContext;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response, StatusCode};
use routerify_ng::ext::RequestExt;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

pub async fn get_all_mocks(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let response_cache = req
        .data::<Arc<MockersContext>>()
        .map(|c| c.clone().response_cache.clone());
    match response_cache {
        Some(cache) => {
            let all_mocks = cache.get_all().await;
            let json = serde_json::to_string(&Ok::<
                HashMap<String, HashMap<String, CachedResponse>>,
                String,
            >(all_mocks));
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Full::new(Bytes::from(json.unwrap_or("".to_string()))))
                .unwrap())
        }
        None => {
            let json = serde_json::to_string(&Err::<
                HashMap<String, HashMap<String, CachedResponse>>,
                String,
            >("GET_ALL_MOCKS_FAILED".to_string()));
            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from(json.unwrap_or("".to_string()))))
                .unwrap())
        }
    }
}
