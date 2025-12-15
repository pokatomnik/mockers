use hyper::StatusCode;
use std::collections::HashMap;
use tokio::sync::{Mutex, RwLock};

#[derive(Debug, Clone)]
pub struct CachedResponse {
    pub status_code: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct CachedRequest {
    pub method: String,
    pub path: String,
    pub cached_response: CachedResponse,
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
                    CachedRequest,
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

    /// Get response from in-memory cache
    pub async fn get<M: Into<String>, P: Into<String>>(
        &self,
        method: M,
        path: P,
    ) -> Option<CachedResponse> {
        let guard = self.mocks.read().await;
        let mocks = guard.lock().await;
        let mocks_by_path = mocks.get(&path.into())?;
        mocks_by_path
            .get(&method.into())
            .map(|v| v.cached_response.clone())
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
        let mut method_map = mocks.entry(path.clone()).or_insert(HashMap::new());
        method_map.insert(
            method.clone(),
            CachedRequest {
                method: method.clone(),
                path: path.clone(),
                cached_response: response.clone(),
            },
        );
    }

    /// Remove cached response from in-memory cache
    pub async fn remove<M: Into<String>, P: Into<String>>(&self, method: M, path: P) {
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
