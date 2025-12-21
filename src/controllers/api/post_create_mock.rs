use crate::libs::response_cache::CachedResponse;
use crate::server::mockers_context::MockersContext;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::{Request, Response, StatusCode};
use routerify_ng::ext::RequestExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use crate::libs::protocol_result::ProtocolResult;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateMockParams {
    path: String,
    method: String,
    status_code: u16,
    headers: HashMap<String, String>,
    body: String,
}

pub async fn post_create_mock(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let response_cache = req
        .data::<Arc<MockersContext>>()
        .map(|c| c.clone().response_cache.clone());

    match response_cache {
        Some(cache) => {
            let body_bytes = req.body().clone().collect().await.unwrap().to_bytes();
            let body_str = String::from_utf8(body_bytes.to_vec()).unwrap_or(String::new());
            let create_mock_params = serde_json::from_str::<CreateMockParams>(&body_str);

            match create_mock_params {
                Ok(mock_params) => {
                    // do not wait, respond immediately
                    let path = mock_params.path.clone();
                    let method = mock_params.method.clone();
                    tokio::spawn(async move {
                        let response = &CachedResponse {
                            status_code: mock_params.status_code,
                            headers: mock_params.headers,
                            body: mock_params.body,
                        };
                        cache.upsert(path, method, response).await;
                    });
                    let json = serde_json::to_string(&ProtocolResult::Ok::<String, String>(format!(
                        "Mock inserted for path: '{}' and method: '{}'",
                        mock_params.path, mock_params.method
                    )));
                    let status = json
                        .as_ref()
                        .map(|_| StatusCode::OK)
                        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
                    let bytes = json.map(|str| Bytes::from(str)).unwrap_or(Bytes::new());
                    Ok(Response::builder()
                        .status(status)
                        .body(Full::new(Bytes::from(bytes)))
                        .unwrap())
                }
                Err(_) => {
                    let json = serde_json::to_string(&ProtocolResult::Err::<
                        HashMap<String, HashMap<String, CachedResponse>>,
                        String,
                    >(
                        "ADD_MOCK_FAILED".to_string()
                    ));
                    let bytes = json.map(|s| Bytes::from(s)).unwrap_or(Bytes::new());
                    Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Full::new(bytes))
                        .unwrap())
                }
            }
        }
        None => {
            let json = serde_json::to_string(&ProtocolResult::Err::<
                HashMap<String, HashMap<String, CachedResponse>>,
                String,
            >("GET_ALL_MOCKS_FAILED".to_string()));
            let bytes = json.map(|s| Bytes::from(s)).unwrap_or(Bytes::new());
            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from(bytes)))
                .unwrap())
        }
    }
}
