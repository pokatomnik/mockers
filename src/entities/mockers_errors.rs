use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MockersErrors {
    // INTERNAL_SERVER_ERROR
    InternalServerError,
    // BAD_REQUEST
    BadRequest,
    // NOT_FOUND
    NotFound,
}

static UNKNOWN_ERROR: &'static str = "UNKNOWN_ERROR";

impl Display for MockersErrors {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string(self) {
            Ok(v) => f.write_str(v.as_str()),
            Err(_) => f.write_str(UNKNOWN_ERROR),
        }
    }
}
