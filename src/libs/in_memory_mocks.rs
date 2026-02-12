use crate::libs::cached_response::CachedResponse;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct InMemoryMocks {
    mocks: RwLock<
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
}

impl InMemoryMocks {
    pub fn create() -> InMemoryMocks {
        InMemoryMocks {
            mocks: RwLock::new(HashMap::new()),
        }
    }

    /// Get all mocks
    pub async fn get_all(&self) -> HashMap<String, HashMap<String, CachedResponse>> {
        let mocks = self.mocks.read().await;
        mocks.clone()
    }

    /// Get all mocks by specified path
    pub async fn get_by_path<P>(&self, path: P) -> Option<HashMap<String, CachedResponse>>
    where
        P: Into<String>,
    {
        let mocks = self.mocks.write().await;
        mocks.get(&path.into().normalize_path()).cloned()
    }

    /// Get response from in-memory cache
    pub async fn get_mock_by_path_and_method<M, P>(
        &self,
        path: P,
        method: M,
    ) -> Option<CachedResponse>
    where
        M: Into<String>,
        P: Into<String>,
    {
        let mocks = self.mocks.read().await;
        let mocks_by_path = mocks.get(&path.into().normalize_path()).cloned()?;
        mocks_by_path
            .get(&method.into().normalize_method())
            .map(|v| v.clone())
    }

    /// Set response to in-memory cache
    pub async fn upsert<M, P>(&self, path: P, method: M, response: &CachedResponse)
    where
        M: Into<String>,
        P: Into<String>,
    {
        let mut mocks = self.mocks.write().await;

        let path = path.into().normalize_path();
        let method = method.into().normalize_method();
        let method_map = mocks.entry(path.clone()).or_insert(HashMap::new());
        method_map.insert(method.clone(), response.clone());
    }

    /// Remove cached response from in-memory cache by path and method
    pub async fn remove_by_path_and_method<M, P>(&self, path: P, method: M)
    where
        M: Into<String>,
        P: Into<String>,
    {
        let mut mocks = self.mocks.write().await;
        let path = path.into().normalize_path();
        let method = method.into().normalize_method();
        let mocks_by_method = mocks.get_mut(&path);
        if let Some(mocks_by_method) = mocks_by_method {
            mocks_by_method.remove(&method);
            if mocks_by_method.is_empty() {
                mocks.remove(&path);
                mocks.shrink_to_fit();
            }
        }
    }

    /// Remove cached responses by path
    pub async fn remove_by_path<P>(&self, path: P)
    where
        P: Into<String>,
    {
        let mut mocks = self.mocks.write().await;
        let path = path.into().normalize_path();
        mocks.remove(&path);
    }
}

pub trait PathDecoder {
    fn decode_path(&self) -> Result<String, Box<dyn std::error::Error>>;
}

impl PathDecoder for String {
    fn decode_path(&self) -> Result<String, Box<dyn std::error::Error>> {
        let bytes = BASE64_STANDARD.decode(self)?;
        let str = String::from_utf8(bytes)?;
        Ok(str)
    }
}

pub trait PathNormalizer {
    fn normalize_path(&self) -> String;
}

impl PathNormalizer for String {
    fn normalize_path(&self) -> String {
        self.trim_matches('/').to_string()
    }
}

pub trait MethodNormalizer {
    fn normalize_method(&self) -> String;
}

impl MethodNormalizer for String {
    fn normalize_method(&self) -> String {
        self.to_lowercase()
    }
}
