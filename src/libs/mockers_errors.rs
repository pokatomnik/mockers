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
    // REMOVE_MOCKS_FAILED_BY_PATH_AND_METHOD
    // RemoveMocksFailedByPathAndMethod,
    // REMOVE_MOCKS_FAILED_BY_PATH
    // RemoveMocksFailedByPath,
    // GET_MOCKS_BY_PATH_FAILED
    // GetMocksByPathFailed,
    // GET_MOCKS_BY_PATH_AND_METHOD_FAILED
    // GetMocksByPathAndMethodFailed,
    // ADD_MOCK_FAILED
    // AddMockFailed,
    // NO_SUCH_MOCK
    // NoSuchMock,
    // MISSING_MOCKS_DIRECTORY
    // MissingMocksDirectory,
    // DUMP_MOCK_BODY_FAILED
    // DumpMockBodyFailed,
    // DUMP_MOCK_CONFIG_FAILED,
    // DumpMockConfigFailed,
    // DUMP_MOCK_AND__MOCK_CONFIG_FAILED
    // DumpMockAndMockConfigFailed,
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
