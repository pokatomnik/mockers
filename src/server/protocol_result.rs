use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::{Display, Formatter};
use std::sync::LazyLock;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProtocolResult<T, E> {
    Ok(T),
    Err(E),
}

static DEFAULT_ERROR: LazyLock<String> =
    LazyLock::new(|| json!({ "err": "UNKNOWN_ERROR" }).to_string());

impl<T, E> Display for ProtocolResult<T, E>
where
    T: Serialize,
    E: Serialize,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string(self) {
            Ok(v) => f.write_str(v.as_str()),
            Err(_) => f.write_str(DEFAULT_ERROR.as_str()),
        }
    }
}

pub trait ProtocolResultConverter<T, E> {
    fn to_protocol(self) -> ProtocolResult<T, E>;
}

impl<T, E> ProtocolResultConverter<T, E> for Result<T, E> {
    /// Convert `Result` to `ProtocolResult` for HTTP transmission
    fn to_protocol(self) -> ProtocolResult<T, E> {
        match self {
            Ok(v) => ProtocolResult::Ok(v),
            Err(e) => ProtocolResult::Err(e),
        }
    }
}
