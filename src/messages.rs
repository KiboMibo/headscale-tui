use crate::api::types::{ApiKey, Node, PreAuthKey, User};

// ── API Response Messages ──────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct UsersLoadedMsg {
    pub users: Vec<User>,
}

#[derive(Debug, Clone)]
pub struct NodesLoadedMsg {
    pub nodes: Vec<Node>,
}

#[derive(Debug, Clone)]
pub struct PreAuthKeysLoadedMsg {
    pub keys: Vec<PreAuthKey>,
}

#[derive(Debug, Clone)]
pub struct ApiKeysLoadedMsg {
    pub keys: Vec<ApiKey>,
}

#[derive(Debug, Clone)]
pub struct ApiErrorMsg {
    pub error: String,
    pub context: String,
}

#[derive(Debug, Clone)]
pub struct StatusMsg {
    pub message: String,
}

// ── Action Result Messages ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct UserCreatedMsg {
    pub user: User,
}

#[derive(Debug, Clone)]
pub struct UserDeletedMsg {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct NodeDeletedMsg {
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct NodeExpiredMsg {
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct NodeRenamedMsg {
    pub id: String,
    pub new_name: String,
}

#[derive(Debug, Clone)]
pub struct PreAuthKeyCreatedMsg {
    pub key: PreAuthKey,
}

#[derive(Debug, Clone)]
pub struct PreAuthKeyExpiredMsg {
    pub key: String,
}

#[derive(Debug, Clone)]
pub struct ApiKeyCreatedMsg {
    pub key: String,
}

#[derive(Debug, Clone)]
pub struct ApiKeyExpiredMsg {
    pub prefix: String,
}

#[derive(Debug, Clone)]
pub struct ApiKeyDeletedMsg {
    pub prefix: String,
}

// ── Navigation / UI Messages ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct InitRenderMsg;

#[derive(Debug, Clone)]
pub struct RefreshMsg;

#[derive(Debug, Clone)]
pub struct ConfirmDeleteMsg {
    pub confirmed: bool,
}
