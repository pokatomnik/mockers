use std::fmt::Display;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

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
