use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::{Mutex, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

pub struct InMemoryMocks {
    mocks: RwLock<
        Mutex<
            HashMap<
                // pathname
                String,
                HashMap<
                    // method
                    String,
                    CachedResponse,
                >,
            >,
        >,
    >,
}

impl InMemoryMocks {
    pub fn create() -> InMemoryMocks {
        return InMemoryMocks {
            mocks: RwLock::new(Mutex::new(HashMap::new())),
        };
    }

    /// Get all mocks
    pub async fn get_all(&self) -> HashMap<String, HashMap<String, CachedResponse>> {
        let guard = self.mocks.read().await;
        let mocks = guard.lock().await;
        return mocks.clone();
    }

    /// Get all mocks by specified path
    pub async fn get_by_path<P>(&self, path: P) -> Option<HashMap<String, CachedResponse>>
    where
        P: Into<String>,
    {
        let guard = self.mocks.write().await;
        let mocks = guard.lock().await;
        return mocks.get(&path.into()).cloned();
    }

    /// Get response from in-memory cache
    pub async fn get<M: Into<String>, P: Into<String>>(
        &self,
        method: M,
        path: P,
    ) -> Option<CachedResponse> {
        let guard = self.mocks.read().await;
        let mocks = guard.lock().await;
        let mocks_by_path = mocks.get(&path.into())?;
        mocks_by_path.get(&method.into()).map(|v| v.clone())
    }

    /// Set response to in-memory cache
    pub async fn upsert<M: Into<String>, P: Into<String>>(
        &self,
        method: M,
        path: P,
        response: &CachedResponse,
    ) -> () {
        let guard = self.mocks.write().await;
        let mut mocks = guard.lock().await;

        let path = path.into();
        let method = method.into();
        let method_map = mocks.entry(path.clone()).or_insert(HashMap::new());
        method_map.insert(method.clone(), response.clone());
    }

    /// Remove cached response from in-memory cache
    pub async fn remove<M, P>(&self, method: M, path: P)
    where
        M: Into<String>,
        P: Into<String>,
    {
        let guard = self.mocks.write().await;
        let mut mocks = guard.lock().await;
        let path = path.into();
        let method = method.into();
        let mut mocks_by_method = mocks.get_mut(&path);
        if let Some(mocks_by_method) = &mut mocks_by_method {
            mocks_by_method.remove(&method);
            if mocks_by_method.is_empty() {
                mocks.remove(&path);
                mocks.shrink_to_fit();
            }
        }
    }
}
