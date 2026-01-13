//! GraphQL mock cache implementation based on AST hashing.
//!
//! This module provides in-memory storage for GraphQL mock responses,
//! matching requests by computing a hash of the normalized AST structure.
//! This allows matching queries regardless of whitespace, comments, or
//! field ordering while still distinguishing structurally different queries.

use graphql_parser::query::{
    parse_query, Definition, Document, Field, FragmentDefinition, FragmentSpread, InlineFragment,
    OperationDefinition, Selection, SelectionSet, TypeCondition, VariableDefinition,
};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Errors that can occur when parsing GraphQL requests
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum GraphQLParseError {
    InvalidJson(String),
    MissingQuery,
    ParseError(String),
}

impl std::fmt::Display for GraphQLParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidJson(msg) => write!(f, "Invalid JSON: {}", msg),
            Self::MissingQuery => write!(f, "Missing 'query' field in GraphQL request"),
            Self::ParseError(msg) => write!(f, "GraphQL parse error: {}", msg),
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
    /// Parse the query and compute a normalized hash for matching
    pub fn compute_query_hash(&self) -> Result<u64, GraphQLParseError> {
        compute_query_hash(&self.query, self.operation_name.as_deref())
    }

    /// Extract operation info from the parsed query
    pub fn extract_operation_info(&self) -> Result<OperationInfo, GraphQLParseError> {
        extract_operation_info(&self.query, self.operation_name.as_deref())
    }
}

/// Information extracted from a GraphQL operation
#[derive(Debug, Clone)]
pub struct OperationInfo {
    pub operation_type: GraphQLOperationType,
    pub operation_name: Option<String>,
    pub query_hash: u64,
}

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
            _ => Err(GraphQLParseError::ParseError(format!(
                "Invalid operation type: {}",
                s
            ))),
        }
    }
}

/// Key for identifying GraphQL mock entries - based on path and query hash
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GraphQLMockKey {
    /// Endpoint path (e.g., "graphql")
    pub path: String,
    /// Hash computed from normalized AST
    pub query_hash: u64,
}

impl GraphQLMockKey {
    pub fn new(path: impl Into<String>, query_hash: u64) -> Self {
        Self {
            path: path.into().normalize_graphql_path(),
            query_hash,
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
    /// Original query string for display purposes
    #[serde(default)]
    pub original_query: Option<String>,
    /// Operation name for display purposes
    #[serde(default)]
    pub operation_name: Option<String>,
    /// Operation type for display purposes
    #[serde(default)]
    pub operation_type: Option<GraphQLOperationType>,
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
            original_query: None,
            operation_name: None,
            operation_type: None,
        }
    }

    pub fn with_metadata(
        mut self,
        query: String,
        operation_name: Option<String>,
        operation_type: GraphQLOperationType,
    ) -> Self {
        self.original_query = Some(query);
        self.operation_name = operation_name;
        self.operation_type = Some(operation_type);
        self
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

#[allow(dead_code)]
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
    pub async fn get_by_path(
        &self,
        path: impl Into<String>,
    ) -> Vec<(GraphQLMockKey, GraphQLCachedResponse)> {
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

    /// Find mock by path and query hash
    pub async fn find_by_hash(
        &self,
        path: impl Into<String>,
        query_hash: u64,
    ) -> Option<GraphQLCachedResponse> {
        let key = GraphQLMockKey::new(path, query_hash);
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

    /// Remove mock by path and query hash
    pub async fn remove_by_hash(
        &self,
        path: impl Into<String>,
        query_hash: u64,
    ) -> Option<GraphQLCachedResponse> {
        let key = GraphQLMockKey::new(path, query_hash);
        self.remove(&key).await
    }

    /// Remove all mocks for a path
    pub async fn remove_by_path(&self, path: impl Into<String>) -> usize {
        let path = path.into().normalize_graphql_path();
        let mut mocks = self.mocks.write().await;
        let keys_to_remove: Vec<_> = mocks.keys().filter(|k| k.path == path).cloned().collect();
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

// ============================================================================
// AST Hashing Implementation
// ============================================================================

/// Compute hash for a query string (public API for creating mocks)
pub fn compute_query_hash(
    query: &str,
    operation_name: Option<&str>,
) -> Result<u64, GraphQLParseError> {
    let ast = parse_query::<String>(query).map_err(|e| GraphQLParseError::ParseError(e.to_string()))?;
    Ok(compute_document_hash(&ast, operation_name))
}

/// Compute hash from a GraphQL document AST
fn compute_document_hash<'a, T: graphql_parser::query::Text<'a>>(
    doc: &Document<'a, T>,
    operation_name: Option<&str>,
) -> u64
where
    T::Value: AsRef<str> + Hash,
{
    let mut hasher = DefaultHasher::new();

    for def in &doc.definitions {
        match def {
            Definition::Operation(op) => {
                if should_hash_operation(op, operation_name) {
                    hash_operation(&mut hasher, op);
                }
            }
            Definition::Fragment(frag) => {
                hash_fragment(&mut hasher, frag);
            }
        }
    }

    hasher.finish()
}

/// Check if this operation should be hashed
fn should_hash_operation<'a, T: graphql_parser::query::Text<'a>>(
    op: &OperationDefinition<'a, T>,
    operation_name: Option<&str>,
) -> bool
where
    T::Value: AsRef<str>,
{
    match operation_name {
        Some(name) => match op {
            OperationDefinition::Query(q) => q
                .name
                .as_ref()
                .map(|n| n.as_ref() == name)
                .unwrap_or(false),
            OperationDefinition::Mutation(m) => m
                .name
                .as_ref()
                .map(|n| n.as_ref() == name)
                .unwrap_or(false),
            OperationDefinition::Subscription(s) => s
                .name
                .as_ref()
                .map(|n| n.as_ref() == name)
                .unwrap_or(false),
            OperationDefinition::SelectionSet(_) => false,
        },
        None => true,
    }
}

/// Hash an operation definition
fn hash_operation<'a, T: graphql_parser::query::Text<'a>, H: Hasher>(
    hasher: &mut H,
    op: &OperationDefinition<'a, T>,
) where
    T::Value: AsRef<str> + Hash,
{
    match op {
        OperationDefinition::Query(q) => {
            "query".hash(hasher);
            if let Some(ref name) = q.name {
                name.as_ref().hash(hasher);
            }
            hash_variable_definitions(hasher, &q.variable_definitions);
            hash_selection_set(hasher, &q.selection_set);
        }
        OperationDefinition::Mutation(m) => {
            "mutation".hash(hasher);
            if let Some(ref name) = m.name {
                name.as_ref().hash(hasher);
            }
            hash_variable_definitions(hasher, &m.variable_definitions);
            hash_selection_set(hasher, &m.selection_set);
        }
        OperationDefinition::Subscription(s) => {
            "subscription".hash(hasher);
            if let Some(ref name) = s.name {
                name.as_ref().hash(hasher);
            }
            hash_variable_definitions(hasher, &s.variable_definitions);
            hash_selection_set(hasher, &s.selection_set);
        }
        OperationDefinition::SelectionSet(ss) => {
            "query".hash(hasher);
            hash_selection_set(hasher, ss);
        }
    }
}

/// Hash variable definitions
fn hash_variable_definitions<'a, T: graphql_parser::query::Text<'a>, H: Hasher>(
    hasher: &mut H,
    vars: &[VariableDefinition<'a, T>],
) where
    T::Value: AsRef<str> + Hash,
{
    vars.len().hash(hasher);
    for var in vars {
        var.name.as_ref().hash(hasher);
        format!("{}", var.var_type).hash(hasher);
        if let Some(ref default) = var.default_value {
            format!("{}", default).hash(hasher);
        }
    }
}

/// Hash a selection set
fn hash_selection_set<'a, T: graphql_parser::query::Text<'a>, H: Hasher>(
    hasher: &mut H,
    selection_set: &SelectionSet<'a, T>,
) where
    T::Value: AsRef<str> + Hash,
{
    // Sort selections for consistent hashing
    let mut sort_keys: Vec<(usize, String)> = selection_set
        .items
        .iter()
        .enumerate()
        .map(|(i, s)| (i, selection_sort_key(s)))
        .collect();
    sort_keys.sort_by(|a, b| a.1.cmp(&b.1));

    sort_keys.len().hash(hasher);
    for (idx, _) in sort_keys {
        hash_selection(hasher, &selection_set.items[idx]);
    }
}

/// Get sort key for a selection
fn selection_sort_key<'a, T: graphql_parser::query::Text<'a>>(selection: &Selection<'a, T>) -> String
where
    T::Value: AsRef<str>,
{
    match selection {
        Selection::Field(f) => format!("0_field_{}", f.name.as_ref()),
        Selection::FragmentSpread(fs) => format!("1_spread_{}", fs.fragment_name.as_ref()),
        Selection::InlineFragment(inf) => {
            let type_name = inf
                .type_condition
                .as_ref()
                .map(|TypeCondition::On(name)| name.as_ref())
                .unwrap_or("");
            format!("2_inline_{}", type_name)
        }
    }
}

/// Hash a single selection
fn hash_selection<'a, T: graphql_parser::query::Text<'a>, H: Hasher>(
    hasher: &mut H,
    selection: &Selection<'a, T>,
) where
    T::Value: AsRef<str> + Hash,
{
    match selection {
        Selection::Field(field) => hash_field(hasher, field),
        Selection::FragmentSpread(spread) => hash_fragment_spread(hasher, spread),
        Selection::InlineFragment(inline) => hash_inline_fragment(hasher, inline),
    }
}

/// Hash a field
fn hash_field<'a, T: graphql_parser::query::Text<'a>, H: Hasher>(
    hasher: &mut H,
    field: &Field<'a, T>,
) where
    T::Value: AsRef<str> + Hash,
{
    "field".hash(hasher);
    if let Some(ref alias) = field.alias {
        alias.as_ref().hash(hasher);
    }
    field.name.as_ref().hash(hasher);

    // Hash arguments in sorted order
    let mut args: Vec<_> = field.arguments.iter().collect();
    args.sort_by(|a, b| a.0.as_ref().cmp(b.0.as_ref()));
    args.len().hash(hasher);
    for (name, value) in args {
        name.as_ref().hash(hasher);
        format!("{}", value).hash(hasher);
    }

    hash_selection_set(hasher, &field.selection_set);
}

/// Hash a fragment spread
fn hash_fragment_spread<'a, T: graphql_parser::query::Text<'a>, H: Hasher>(
    hasher: &mut H,
    spread: &FragmentSpread<'a, T>,
) where
    T::Value: AsRef<str> + Hash,
{
    "fragment_spread".hash(hasher);
    spread.fragment_name.as_ref().hash(hasher);
}

/// Hash an inline fragment
fn hash_inline_fragment<'a, T: graphql_parser::query::Text<'a>, H: Hasher>(
    hasher: &mut H,
    inline: &InlineFragment<'a, T>,
) where
    T::Value: AsRef<str> + Hash,
{
    "inline_fragment".hash(hasher);
    if let Some(TypeCondition::On(ref name)) = inline.type_condition {
        name.as_ref().hash(hasher);
    }
    hash_selection_set(hasher, &inline.selection_set);
}

/// Hash a fragment definition
fn hash_fragment<'a, T: graphql_parser::query::Text<'a>, H: Hasher>(
    hasher: &mut H,
    frag: &FragmentDefinition<'a, T>,
) where
    T::Value: AsRef<str> + Hash,
{
    "fragment_def".hash(hasher);
    frag.name.as_ref().hash(hasher);
    let TypeCondition::On(ref type_name) = frag.type_condition;
    type_name.as_ref().hash(hasher);
    hash_selection_set(hasher, &frag.selection_set);
}

/// Extract operation info from a query string
pub fn extract_operation_info(
    query: &str,
    operation_name: Option<&str>,
) -> Result<OperationInfo, GraphQLParseError> {
    let ast = parse_query::<String>(query).map_err(|e| GraphQLParseError::ParseError(e.to_string()))?;

    let mut hasher = DefaultHasher::new();
    let mut found_op: Option<(GraphQLOperationType, Option<String>)> = None;

    for def in &ast.definitions {
        match def {
            Definition::Operation(op) => {
                let (op_type, name) = match op {
                    OperationDefinition::Query(q) => {
                        (GraphQLOperationType::Query, q.name.as_ref().map(|s| s.clone()))
                    }
                    OperationDefinition::Mutation(m) => {
                        (GraphQLOperationType::Mutation, m.name.as_ref().map(|s| s.clone()))
                    }
                    OperationDefinition::Subscription(s) => {
                        (GraphQLOperationType::Subscription, s.name.as_ref().map(|s| s.clone()))
                    }
                    OperationDefinition::SelectionSet(_) => (GraphQLOperationType::Query, None),
                };

                let matches = match operation_name {
                    Some(target) => name.as_ref().map(|n| n == target).unwrap_or(false),
                    None => found_op.is_none(),
                };

                if matches {
                    found_op = Some((op_type, name));
                    hash_operation(&mut hasher, op);
                }
            }
            Definition::Fragment(frag) => {
                hash_fragment(&mut hasher, frag);
            }
        }
    }

    match found_op {
        Some((op_type, name)) => Ok(OperationInfo {
            operation_type: op_type,
            operation_name: name,
            query_hash: hasher.finish(),
        }),
        None => Err(GraphQLParseError::ParseError(
            "No matching operation found".to_string(),
        )),
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
#[allow(dead_code)]
pub type SharedGraphQLMocks = Arc<InMemoryGraphQLMocks>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_query_same_hash() {
        let query1 = "query GetUser { user { id name } }";
        let query2 = "query GetUser { user { id name } }";

        let hash1 = compute_query_hash(query1, Some("GetUser")).unwrap();
        let hash2 = compute_query_hash(query2, Some("GetUser")).unwrap();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_whitespace_invariant() {
        let query1 = "query GetUser { user { id name } }";
        let query2 = "query GetUser {\n  user {\n    id\n    name\n  }\n}";

        let hash1 = compute_query_hash(query1, Some("GetUser")).unwrap();
        let hash2 = compute_query_hash(query2, Some("GetUser")).unwrap();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_queries_different_hash() {
        let query1 = "query GetUser { user { id } }";
        let query2 = "query GetUser { user { id name } }";

        let hash1 = compute_query_hash(query1, Some("GetUser")).unwrap();
        let hash2 = compute_query_hash(query2, Some("GetUser")).unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_field_order_invariant() {
        let query1 = "query GetUser { user { id name email } }";
        let query2 = "query GetUser { user { email id name } }";

        let hash1 = compute_query_hash(query1, Some("GetUser")).unwrap();
        let hash2 = compute_query_hash(query2, Some("GetUser")).unwrap();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_anonymous_query() {
        let query1 = "{ user { id } }";
        let query2 = "{ user { id } }";

        let hash1 = compute_query_hash(query1, None).unwrap();
        let hash2 = compute_query_hash(query2, None).unwrap();

        assert_eq!(hash1, hash2);
    }

    #[tokio::test]
    async fn test_in_memory_mocks_crud() {
        let mocks = InMemoryGraphQLMocks::new();
        let query_hash =
            compute_query_hash("query GetUser { user { id } }", Some("GetUser")).unwrap();
        let key = GraphQLMockKey::new("/graphql", query_hash);
        let response = GraphQLCachedResponse::new(
            200,
            0,
            HashMap::new(),
            serde_json::json!({"user": {"id": "1"}}),
            None,
        );

        mocks.upsert(key.clone(), response).await;

        let retrieved = mocks.get(&key).await;
        assert!(retrieved.is_some());

        let removed = mocks.remove(&key).await;
        assert!(removed.is_some());

        let after_remove = mocks.get(&key).await;
        assert!(after_remove.is_none());
    }
}
