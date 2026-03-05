use serde::{Deserialize, Serialize};

// ── User ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, rename = "createdAt")]
    pub created_at: String,
    #[serde(default, rename = "displayName")]
    pub display_name: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub provider: String,
    #[serde(default, rename = "providerId")]
    pub provider_id: String,
    #[serde(default, rename = "profilePicUrl")]
    pub profile_pic_url: String,
}

// ── Node ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    #[serde(default)]
    pub id: String,
    #[serde(default, rename = "machineKey")]
    pub machine_key: String,
    #[serde(default, rename = "nodeKey")]
    pub node_key: String,
    #[serde(default, rename = "discoKey")]
    pub disco_key: String,
    #[serde(default, rename = "ipAddresses")]
    pub ip_addresses: Vec<String>,
    #[serde(default)]
    pub name: String,
    pub user: Option<User>,
    #[serde(default, rename = "lastSeen")]
    pub last_seen: Option<String>,
    #[serde(default)]
    pub expiry: Option<String>,
    #[serde(default, rename = "createdAt")]
    pub created_at: String,
    #[serde(default, rename = "registerMethod")]
    pub register_method: String,
    #[serde(default, rename = "givenName")]
    pub given_name: String,
    #[serde(default)]
    pub online: bool,
    #[serde(default, rename = "approvedRoutes")]
    pub approved_routes: Vec<String>,
    #[serde(default, rename = "availableRoutes")]
    pub available_routes: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

// ── PreAuthKey ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreAuthKey {
    pub user: Option<User>,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub reusable: bool,
    #[serde(default)]
    pub ephemeral: bool,
    #[serde(default)]
    pub used: bool,
    #[serde(default)]
    pub expiration: Option<String>,
    #[serde(default, rename = "createdAt")]
    pub created_at: String,
    #[serde(default, rename = "aclTags")]
    pub acl_tags: Vec<String>,
}

// ── ApiKey ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    #[serde(default)]
    pub id: String, // Headscale API actually returns string IDs despite proto saying uint64
    #[serde(default)]
    pub prefix: String,
    #[serde(default)]
    pub expiration: Option<String>,
    #[serde(default, rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(default, rename = "lastSeen")]
    pub last_seen: Option<String>,
}

// ── API Response Wrappers ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ListUsersResponse {
    #[serde(default)]
    pub users: Vec<User>,
}

#[derive(Debug, Deserialize)]
pub struct ListNodesResponse {
    #[serde(default)]
    pub nodes: Vec<Node>,
}

#[derive(Debug, Deserialize)]
pub struct ListPreAuthKeysResponse {
    #[serde(default, rename = "preAuthKeys")]
    pub pre_auth_keys: Vec<PreAuthKey>,
}

#[derive(Debug, Deserialize)]
pub struct ListApiKeysResponse {
    #[serde(default, rename = "apiKeys")]
    pub api_keys: Vec<ApiKey>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePreAuthKeyRequest {
    pub user: String,
    pub reusable: bool,
    pub ephemeral: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "aclTags")]
    pub acl_tags: Option<Vec<String>>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct CreateApiKeyResponse {
    #[serde(default, rename = "apiKey")]
    pub api_key: String,
}
