#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct Empty {}

impl Empty {
    pub fn new() -> Self {
        Self {}
    }
}
