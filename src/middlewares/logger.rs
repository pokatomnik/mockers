use std::fmt::Debug;
use std::{fmt::Display, sync::Arc};

use clap::ValueEnum;
use http_body_util::Full;
use hyper::Request;
use routerify_ng::ext::RequestExt;

use crate::libs::header_map_ext::HeaderMapConverter;
use crate::libs::mockers_request_ext::BodyReader;
use crate::server::mockers_context::MockersContext;
use crate::server::route_error::MockersRouteError;

static NL: char = '\n';

pub async fn logger(
    req: Request<Full<hyper::body::Bytes>>,
) -> Result<Request<Full<hyper::body::Bytes>>, MockersRouteError> {
    let context = req.data::<Arc<MockersContext>>();
    let request_log = context
        .map(|f| f.server_params.log_request())
        .unwrap_or_default();
    let (log_path, log_headers, log_body) = match request_log {
        RequestLogLevel::Info => (true, false, false),
        RequestLogLevel::Debug => (true, true, false),
        RequestLogLevel::Trace => (true, true, true),
    };
    let path_info = match log_path {
        true => Some(format!("{} {}", req.method(), req.uri())),
        false => None,
    };
    let headers_info = match log_headers {
        true => serde_json::to_string_pretty(&req.headers().to_hash_map()).ok(),
        false => None,
    };
    let log_body = match log_body {
        true => req.body().clone().body().await,
        false => None,
    };

    let Some(mut output) = path_info else {
        return Ok(req);
    };
    if let Some(headers_info) = headers_info {
        output.push(NL);
        output.push_str("[HEADERS]:");
        output.push(NL);
        output.push_str(headers_info.as_str());
    };
    if let Some(log_body) = log_body {
        output.push(NL);
        output.push_str("[BODY]:");
        output.push(NL);
        output.push_str(log_body.as_str());
    }
    println!("{}", output);
    Ok(req)
}

#[derive(ValueEnum, Debug, Clone, Copy)]
#[clap(rename_all = "kebab-case")]
pub(crate) enum RequestLogLevel {
    /// Log only request path and query params
    Info,

    /// Log request path, query params and headers
    Debug,

    /// Log request path, query params, headers and body
    Trace,
}

impl Default for RequestLogLevel {
    fn default() -> Self {
        RequestLogLevel::Info
    }
}

impl Display for RequestLogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestLogLevel::Info => f.write_str("info"),
            RequestLogLevel::Debug => f.write_str("debug"),
            RequestLogLevel::Trace => f.write_str("trace"),
        }
    }
}
