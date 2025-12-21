use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProtocolResult<T, E> {
    Ok(T),
    Err(E),
}

impl<T, E> ProtocolResult<T, E> {
    fn from(r: Result<T, E>) -> Self {
        match r {
            Ok(v) => ProtocolResult::Ok(v),
            Err(e) => ProtocolResult::Err(e),
        }
    }
}
