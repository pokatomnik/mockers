use std::{collections::HashMap, convert::Infallible, pin::Pin, sync::Arc, time::Duration};

use http_body_util::Full;
use hyper::{Request, Response, StatusCode, body::Bytes};
use tokio::fs;

use super::get_mime::get_mime;
use super::query_params::QueryParams;
use crate::server::params::ServerParams;

pub type HandlerFuture =
    Pin<Box<dyn Future<Output = Result<Response<Full<Bytes>>, Infallible>> + Send>>;

pub type Handler =
    Arc<dyn Fn(Request<hyper::body::Incoming>, Arc<ServerParams>) -> HandlerFuture + Send + Sync>;

pub struct Router {
    params: Arc<ServerParams>,
    handlers: HashMap<String, Handler>,
}

impl Router {
    pub fn new(params: ServerParams) -> Router {
        let router = Router {
            params: Arc::new(params),
            handlers: HashMap::new(),
        };

        return router;
    }

    pub fn route(mut self, pattern: String, handler: Handler) -> Self {
        self.handlers.insert(pattern, handler);
        self
    }

    pub async fn handle(
        self: Arc<Self>,
        req: Request<hyper::body::Incoming>,
    ) -> Result<Response<Full<Bytes>>, Infallible> {
        let handler = self.handlers.get(req.uri().path());
        if handler.is_some() {
            let handler = handler.unwrap();
            let result = handler(req, self.params.clone());
            return result.await;
        }

        return mock_handler(req, self.params.clone()).await;
    }
}

pub async fn mock_handler(
    req: Request<hyper::body::Incoming>,
    params: Arc<ServerParams>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let delay = req
        .uri()
        .query()
        .and_then(|q| q.parse::<QueryParams>().ok())
        .and_then(|qp| qp.get("delay")?.get(0)?.parse().ok())
        .unwrap_or(0);
    if params.verbose {
        println!("Serving {}", &req.uri());
    }
    let method = req.method().to_string().to_lowercase();
    let uri_pathname = req.uri().path().to_string();

    let mocks_dir = params.get_absolute_mocks_path();
    if mocks_dir.is_err() {
        return Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Full::new(Bytes::from("")))
            .unwrap());
    }
    let mocks_dir = mocks_dir.unwrap();
    let target_file_path = mocks_dir.join(format!("{}.{}", uri_pathname[1..].to_string(), method));

    if params.verbose {
        println!("Mocks dir: {}", &mocks_dir.display().to_string());
        println!(
            "Requested file: {}",
            &target_file_path.display().to_string()
        );
    }

    return match fs::read(&target_file_path).await {
        Ok(data) => {
            let mime = get_mime(&data);

            if params.verbose {
                println!("File mime: {}", &mime);
            }

            let mut builder = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", mime);
            if params.cors {
                builder = builder
                    .header("Access-Control-Allow-Origin", "*")
                    .header("Access-Control-Allow-Methods", "*");
            }
            let response = builder.body(Full::from(data)).unwrap();

            tokio::time::sleep(Duration::from_millis(delay)).await;

            Ok(response)
        }
        Err(_) => {
            let mut builder = Response::builder().status(StatusCode::NOT_FOUND);

            if params.cors {
                builder = builder
                    .header("Access-Control-Allow-Origin", "*")
                    .header("Access-Control-Allow-Methods", "*");
            }
            let response = builder.body(Full::new(Bytes::from(""))).unwrap();
            Ok(response)
        }
    };
}
