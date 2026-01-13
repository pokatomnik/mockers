//! GraphQL mock cache implementation.
//!
//! This module provides in-memory storage for GraphQL mock responses,
//! allowing matching by operation name, operation type, and optional variables.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// GraphQL operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GraphQLOperationType {
    Query,
    Mutation,
    Subscription,
}

impl Default for GraphQLOperationType {
    fn default() -> Self {
        Self::Query
    }
}

impl std::fmt::Display for GraphQLOperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Query => write!(f, "query"),
            Self::Mutation => write!(f, "mutation"),
            Self::Subscription => write!(f, "subscription"),
        }
    }
}

impl std::str::FromStr for GraphQLOperationType {
    type Err = GraphQLParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "query" => Ok(Self::Query),
            "mutation" => Ok(Self::Mutation),
            "subscription" => Ok(Self::Subscription),
            _ => Err(GraphQLParseError::InvalidOperationType(s.to_string())),
        }
    }
}

/// Errors that can occur when parsing GraphQL requests
#[derive(Debug, Clone)]
pub enum GraphQLParseError {
    InvalidJson(String),
    MissingQuery,
    InvalidOperationType(String),
    InvalidQuery(String),
}

impl std::fmt::Display for GraphQLParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidJson(msg) => write!(f, "Invalid JSON: {}", msg),
            Self::MissingQuery => write!(f, "Missing 'query' field in GraphQL request"),
            Self::InvalidOperationType(op) => write!(f, "Invalid operation type: {}", op),
            Self::InvalidQuery(msg) => write!(f, "Invalid GraphQL query: {}", msg),
        }
    }
}

impl std::error::Error for GraphQLParseError {}

/// Incoming GraphQL request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphQLRequest {
    pub query: String,
    #[serde(default)]
    pub operation_name: Option<String>,
    #[serde(default)]
    pub variables: Option<serde_json::Value>,
}

impl GraphQLRequest {
    /// Extract operation type from the query string
    pub fn parse_operation_type(&self) -> GraphQLOperationType {
        let query = self.query.trim();

        // Check for explicit operation type at the start
        if query.starts_with("mutation") {
            return GraphQLOperationType::Mutation;
        }
        if query.starts_with("subscription") {
            return GraphQLOperationType::Subscription;
        }
        if query.starts_with("query") {
            return GraphQLOperationType::Query;
        }

        // If no explicit type, check for shorthand query syntax (starts with '{')
        if query.starts_with('{') {
            return GraphQLOperationType::Query;
        }

        // Default to query
        GraphQLOperationType::Query
    }

    /// Extract operation name from the query if not provided explicitly
    pub fn resolve_operation_name(&self) -> Option<String> {
        if self.operation_name.is_some() {
            return self.operation_name.clone();
        }

        // Try to parse operation name from query string
        // Pattern: (query|mutation|subscription) OperationName { ... }
        // or:      (query|mutation|subscription) OperationName($var: Type) { ... }
        let query = self.query.trim();

        for prefix in ["query", "mutation", "subscription"] {
            if query.starts_with(prefix) {
                let rest = query[prefix.len()..].trim_start();
                // Extract name before '(' or '{'
                let name_end = rest
                    .find(|c: char| c == '(' || c == '{' || c.is_whitespace())
                    .unwrap_or(rest.len());

                let name = rest[..name_end].trim();
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }

        None
    }
}

/// Key for identifying GraphQL mock entries
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GraphQLMockKey {
    /// Endpoint path (e.g., "/graphql")
    pub path: String,
    /// Operation name (e.g., "GetUser", "CreatePost")
    pub operation_name: String,
    /// Operation type (query, mutation, subscription)
    pub operation_type: GraphQLOperationType,
}

impl GraphQLMockKey {
    pub fn new(
        path: impl Into<String>,
        operation_name: impl Into<String>,
        operation_type: GraphQLOperationType,
    ) -> Self {
        Self {
            path: path.into().normalize_graphql_path(),
            operation_name: operation_name.into(),
            operation_type,
        }
    }
}

/// Cached GraphQL response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphQLCachedResponse {
    pub status_code: u16,
    pub delay_ms: u64,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    pub data: serde_json::Value,
    #[serde(default)]
    pub errors: Option<Vec<GraphQLError>>,
}

impl GraphQLCachedResponse {
    pub fn new(
        status_code: u16,
        delay_ms: u64,
        headers: HashMap<String, String>,
        data: serde_json::Value,
        errors: Option<Vec<GraphQLError>>,
    ) -> Self {
        Self {
            status_code,
            delay_ms,
            headers,
            data,
            errors,
        }
    }

    /// Create a success response with data
    pub fn success(data: serde_json::Value) -> Self {
        Self::new(200, 0, HashMap::new(), data, None)
    }

    /// Create an error response
    pub fn error(errors: Vec<GraphQLError>) -> Self {
        Self::new(200, 0, HashMap::new(), serde_json::Value::Null, Some(errors))
    }

    /// Convert to JSON response body
    pub fn to_response_body(&self) -> serde_json::Value {
        let mut response = serde_json::json!({
            "data": self.data
        });

        if let Some(ref errors) = self.errors {
            response["errors"] = serde_json::to_value(errors).unwrap_or_default();
        }

        response
    }
}

/// GraphQL error structure following the spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLError {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locations: Option<Vec<GraphQLErrorLocation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Value>,
}

impl GraphQLError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            locations: None,
            path: None,
            extensions: None,
        }
    }
}

/// Location of an error in the GraphQL query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLErrorLocation {
    pub line: u32,
    pub column: u32,
}

/// In-memory storage for GraphQL mocks
pub struct InMemoryGraphQLMocks {
    mocks: RwLock<HashMap<GraphQLMockKey, GraphQLCachedResponse>>,
}

impl InMemoryGraphQLMocks {
    pub fn new() -> Self {
        Self {
            mocks: RwLock::new(HashMap::new()),
        }
    }

    /// Get all GraphQL mocks
    pub async fn get_all(&self) -> HashMap<GraphQLMockKey, GraphQLCachedResponse> {
        self.mocks.read().await.clone()
    }

    /// Get all mocks for a specific path
    pub async fn get_by_path(&self, path: impl Into<String>) -> Vec<(GraphQLMockKey, GraphQLCachedResponse)> {
        let path = path.into().normalize_graphql_path();
        let mocks = self.mocks.read().await;
        mocks
            .iter()
            .filter(|(key, _)| key.path == path)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Get a specific mock by key
    pub async fn get(&self, key: &GraphQLMockKey) -> Option<GraphQLCachedResponse> {
        self.mocks.read().await.get(key).cloned()
    }

    /// Find mock matching request
    pub async fn find_mock(
        &self,
        path: impl Into<String>,
        operation_name: &str,
        operation_type: GraphQLOperationType,
    ) -> Option<GraphQLCachedResponse> {
        let key = GraphQLMockKey::new(path, operation_name, operation_type);
        self.get(&key).await
    }

    /// Insert or update a mock
    pub async fn upsert(&self, key: GraphQLMockKey, response: GraphQLCachedResponse) {
        let mut mocks = self.mocks.write().await;
        mocks.insert(key, response);
    }

    /// Remove a specific mock
    pub async fn remove(&self, key: &GraphQLMockKey) -> Option<GraphQLCachedResponse> {
        let mut mocks = self.mocks.write().await;
        mocks.remove(key)
    }

    /// Remove all mocks for a path
    pub async fn remove_by_path(&self, path: impl Into<String>) -> usize {
        let path = path.into().normalize_graphql_path();
        let mut mocks = self.mocks.write().await;
        let keys_to_remove: Vec<_> = mocks
            .keys()
            .filter(|k| k.path == path)
            .cloned()
            .collect();
        let count = keys_to_remove.len();
        for key in keys_to_remove {
            mocks.remove(&key);
        }
        count
    }

    /// Remove all mocks for a path and operation name
    pub async fn remove_by_path_and_operation(
        &self,
        path: impl Into<String>,
        operation_name: impl Into<String>,
    ) -> usize {
        let path = path.into().normalize_graphql_path();
        let operation_name = operation_name.into();
        let mut mocks = self.mocks.write().await;
        let keys_to_remove: Vec<_> = mocks
            .keys()
            .filter(|k| k.path == path && k.operation_name == operation_name)
            .cloned()
            .collect();
        let count = keys_to_remove.len();
        for key in keys_to_remove {
            mocks.remove(&key);
        }
        count
    }

    /// Clear all mocks
    pub async fn clear(&self) {
        let mut mocks = self.mocks.write().await;
        mocks.clear();
    }
}

impl Default for InMemoryGraphQLMocks {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for normalizing GraphQL paths
pub trait GraphQLPathNormalizer {
    fn normalize_graphql_path(&self) -> String;
}

impl GraphQLPathNormalizer for String {
    fn normalize_graphql_path(&self) -> String {
        self.trim_matches('/').to_string()
    }
}

impl GraphQLPathNormalizer for &str {
    fn normalize_graphql_path(&self) -> String {
        self.trim_matches('/').to_string()
    }
}

/// Wrapper for Arc<InMemoryGraphQLMocks>
pub type SharedGraphQLMocks = Arc<InMemoryGraphQLMocks>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_operation_type_query() {
        let req = GraphQLRequest {
            query: "query GetUser { user { id } }".to_string(),
            operation_name: None,
            variables: None,
        };
        assert_eq!(req.parse_operation_type(), GraphQLOperationType::Query);
    }

    #[test]
    fn test_parse_operation_type_mutation() {
        let req = GraphQLRequest {
            query: "mutation CreateUser { createUser { id } }".to_string(),
            operation_name: None,
            variables: None,
        };
        assert_eq!(req.parse_operation_type(), GraphQLOperationType::Mutation);
    }

    #[test]
    fn test_parse_operation_type_shorthand() {
        let req = GraphQLRequest {
            query: "{ user { id } }".to_string(),
            operation_name: None,
            variables: None,
        };
        assert_eq!(req.parse_operation_type(), GraphQLOperationType::Query);
    }

    #[test]
    fn test_resolve_operation_name_explicit() {
        let req = GraphQLRequest {
            query: "query { user { id } }".to_string(),
            operation_name: Some("GetUser".to_string()),
            variables: None,
        };
        assert_eq!(req.resolve_operation_name(), Some("GetUser".to_string()));
    }

    #[test]
    fn test_resolve_operation_name_from_query() {
        let req = GraphQLRequest {
            query: "query GetUserById($id: ID!) { user(id: $id) { id } }".to_string(),
            operation_name: None,
            variables: None,
        };
        assert_eq!(req.resolve_operation_name(), Some("GetUserById".to_string()));
    }

    #[test]
    fn test_graphql_mock_key_equality() {
        let key1 = GraphQLMockKey::new("/graphql", "GetUser", GraphQLOperationType::Query);
        let key2 = GraphQLMockKey::new("graphql", "GetUser", GraphQLOperationType::Query);
        assert_eq!(key1, key2);
    }

    #[tokio::test]
    async fn test_in_memory_mocks_crud() {
        let mocks = InMemoryGraphQLMocks::new();
        let key = GraphQLMockKey::new("/graphql", "GetUser", GraphQLOperationType::Query);
        let response = GraphQLCachedResponse::success(serde_json::json!({"user": {"id": "1"}}));

        // Insert
        mocks.upsert(key.clone(), response.clone()).await;

        // Get
        let retrieved = mocks.get(&key).await;
        assert!(retrieved.is_some());

        // Remove
        let removed = mocks.remove(&key).await;
        assert!(removed.is_some());

        // Verify removed
        let after_remove = mocks.get(&key).await;
        assert!(after_remove.is_none());
    }
}
