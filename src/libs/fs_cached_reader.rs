use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::{Mutex, OnceCell};

#[derive(Default)]
pub(crate) struct FSCachedReader {
    cache: Arc<Mutex<HashMap<PathBuf, Arc<OnceCell<Arc<Result<Vec<u8>, anyhow::Error>>>>>>>,
}

impl FSCachedReader {
    async fn get_or_insert(
        &self,
        path: impl AsRef<Path>,
    ) -> Arc<OnceCell<Arc<Result<Vec<u8>, anyhow::Error>>>> {
        let path: PathBuf = path.as_ref().to_path_buf();
        let mut cache = self.cache.lock().await;
        let result = cache
            .entry(path)
            .or_insert_with(|| Arc::new(OnceCell::new()));

        return result.clone();
    }

    pub async fn read(&self, path: impl AsRef<Path>) -> Arc<anyhow::Result<Vec<u8>>> {
        let path = path.as_ref().to_path_buf();
        let cell = self.get_or_insert(&path).await;
        cell.get_or_init(async move || {
            let result = tokio::fs::read(path)
                .await
                .map_err(anyhow::Error::from)
                .into();
            Arc::new(result)
        })
        .await
        .clone()
    }
}
