#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MockInfo {
    method: String,
    path: String,
}

impl MockInfo {
    pub(crate) fn new(method: String, path: String) -> Self {
        Self { method, path }
    }

    pub(crate) fn method(&self) -> &str {
        &self.method
    }

    pub(crate) fn path(&self) -> &str {
        &self.path
    }
}
