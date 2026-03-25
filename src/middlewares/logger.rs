use std::fmt::Debug;
use std::{fmt::Display, sync::Arc};

use crate::libs::header_map_ext::HeaderMapConverter;
use crate::libs::mockers_request_ext::BodyReader;
use crate::server::mockers_context::MockersContext;
use crate::server::route_error::MockersRouteError;
use clap::ValueEnum;
use http_body_util::Full;
use hyper::Request;
use log::info;
use routerify_ng::ext::RequestExt;
use serde::{Deserialize, Serialize};

static NL: char = '\n';

pub async fn logger(
    req: Request<Full<hyper::body::Bytes>>,
) -> Result<Request<Full<hyper::body::Bytes>>, MockersRouteError> {
    let context = req.data::<Arc<MockersContext>>();
    let request_log = match context.map(|ctx| &ctx.server_params) {
        None => None,
        Some(sp) => Some(sp.log_request().await),
    }
    .unwrap_or_default();
    let (log_path, log_headers, log_body) = match request_log {
        VerbosityLevel::Info => (true, false, false),
        VerbosityLevel::Debug => (true, true, false),
        VerbosityLevel::Trace => (true, true, true),
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
        true => req.body().clone().read().await,
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
    if let Some(log_body) = log_body
        && !log_body.is_empty()
    {
        output.push(NL);
        output.push_str("[BODY]:");
        output.push(NL);
        output.push_str(log_body.as_str());
    }
    info!("{}", output);
    Ok(req)
}

#[derive(ValueEnum, Debug, Clone, Copy, Serialize, Deserialize)]
#[clap(rename_all = "kebab-case")]
#[serde(rename_all = "lowercase")]
pub(crate) enum VerbosityLevel {
    /// Log only request path and query params
    Info,

    /// Log request path, query params and headers
    Debug,

    /// Log request path, query params, headers and body
    Trace,
}

impl Default for VerbosityLevel {
    fn default() -> Self {
        VerbosityLevel::Info
    }
}

impl Display for VerbosityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerbosityLevel::Info => f.write_str("info"),
            VerbosityLevel::Debug => f.write_str("debug"),
            VerbosityLevel::Trace => f.write_str("trace"),
        }
    }
}
