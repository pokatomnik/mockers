use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProtocolResult<T, E> {
    Ok(T),
    Err(E),
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
