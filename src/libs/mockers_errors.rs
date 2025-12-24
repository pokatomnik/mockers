use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MockersErrors {
    // REMOVE_MOCKS_FAILED_BY_PATH_AND_METHOD
    RemoveMocksFailedByPathAndMethod,
    // REMOVE_MOCKS_FAILED_BY_PATH
    RemoveMocksFailedByPath,
    // GET_ALL_MOCKS_FAILED
    GetAllMocksFailed,
    // GET_MOCKS_BY_PATH_FAILED
    GetMocksByPathFailed,
    // GET_MOCKS_BY_PATH_AND_METHOD_FAILED
    GetMocksByPathAndMethodFailed,
    // ADD_MOCK_FAILED
    AddMockFailed,
}

impl ToString for MockersErrors {
    fn to_string(&self) -> String {
        serde_json::to_string(self).expect("MockersErrors::to_string")
    }
}
