use crate::api::types::*;
use crate::config::Config;
use crate::messages::*;
use bubbletea_rs::Msg;

#[derive(Debug, Clone)]
pub struct HeadscaleClient {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl HeadscaleClient {
    pub fn new(config: &Config) -> Self {
        HeadscaleClient {
            client: reqwest::Client::new(),
            base_url: config.server_url.clone(),
            api_key: config.api_key.clone(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.api_key)
    }

    // ── Users ──

    pub async fn list_users(&self) -> Result<Vec<User>, String> {
        let resp = self
            .client
            .get(self.url("/api/v1/user"))
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("HTTP {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
        }

        let body: ListUsersResponse = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        Ok(body.users)
    }

    pub async fn create_user(&self, name: &str) -> Result<User, String> {
        let resp = self
            .client
            .post(self.url("/api/v1/user"))
            .header("Authorization", self.auth_header())
            .json(&serde_json::json!({ "name": name }))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("HTTP {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let user: User = serde_json::from_value(body["user"].clone()).map_err(|e| format!("Parse error: {}", e))?;
        Ok(user)
    }

    pub async fn delete_user(&self, id: &str) -> Result<(), String> {
        let resp = self
            .client
            .delete(self.url(&format!("/api/v1/user/{}", id)))
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("HTTP {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
        }
        Ok(())
    }

    pub async fn rename_user(&self, old_id: &str, new_name: &str) -> Result<User, String> {
        let resp = self
            .client
            .post(self.url(&format!("/api/v1/user/{}/rename/{}", old_id, new_name)))
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("HTTP {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let user: User = serde_json::from_value(body["user"].clone()).map_err(|e| format!("Parse error: {}", e))?;
        Ok(user)
    }

    // ── Nodes ──

    pub async fn list_nodes(&self) -> Result<Vec<Node>, String> {
        let resp = self
            .client
            .get(self.url("/api/v1/node"))
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("HTTP {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
        }

        let body: ListNodesResponse = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        Ok(body.nodes)
    }

    pub async fn delete_node(&self, id: &str) -> Result<(), String> {
        let resp = self
            .client
            .delete(self.url(&format!("/api/v1/node/{}", id)))
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("HTTP {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
        }
        Ok(())
    }

    pub async fn expire_node(&self, id: &str) -> Result<(), String> {
        let resp = self
            .client
            .post(self.url(&format!("/api/v1/node/{}/expire", id)))
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("HTTP {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
        }
        Ok(())
    }

    pub async fn rename_node(&self, id: &str, new_name: &str) -> Result<(), String> {
        let resp = self
            .client
            .post(self.url(&format!("/api/v1/node/{}/rename/{}", id, new_name)))
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("HTTP {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
        }
        Ok(())
    }

    pub async fn set_node_tags(&self, id: &str, tags: Vec<String>) -> Result<(), String> {
        let resp = self
            .client
            .post(self.url(&format!("/api/v1/node/{}/tags", id)))
            .header("Authorization", self.auth_header())
            .json(&serde_json::json!({ "tags": tags }))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("HTTP {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
        }
        Ok(())
    }

    pub async fn approve_routes(&self, id: &str, routes: Vec<String>) -> Result<(), String> {
        let resp = self
            .client
            .post(self.url(&format!("/api/v1/node/{}/approve_routes", id)))
            .header("Authorization", self.auth_header())
            .json(&serde_json::json!({ "routes": routes }))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("HTTP {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
        }
        Ok(())
    }

    // ── PreAuth Keys ──

    pub async fn list_preauthkeys(&self, user: Option<&str>) -> Result<Vec<PreAuthKey>, String> {
        let mut url = self.url("/api/v1/preauthkey");
        // When no user is specified, we should fetch all preauthkeys
        // If user is specified, filter by user
        if let Some(u) = user {
            url = format!("{}?user={}", url, u);
        }

        let resp = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("HTTP {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
        }

        let body: ListPreAuthKeysResponse = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        Ok(body.pre_auth_keys)
    }

    pub async fn create_preauthkey(&self, req: &CreatePreAuthKeyRequest) -> Result<PreAuthKey, String> {
        let resp = self
            .client
            .post(self.url("/api/v1/preauthkey"))
            .header("Authorization", self.auth_header())
            .json(req)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("HTTP {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        let key: PreAuthKey =
            serde_json::from_value(body["preAuthKey"].clone()).map_err(|e| format!("Parse error: {}", e))?;
        Ok(key)
    }

    pub async fn expire_preauthkey(&self, user: &str, key: &str) -> Result<(), String> {
        let resp = self
            .client
            .post(self.url("/api/v1/preauthkey/expire"))
            .header("Authorization", self.auth_header())
            .json(&serde_json::json!({ "user": user, "key": key }))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("HTTP {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
        }
        Ok(())
    }

    // ── API Keys ──

    pub async fn list_apikeys(&self) -> Result<Vec<ApiKey>, String> {
        let resp = self
            .client
            .get(self.url("/api/v1/apikey"))
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("HTTP {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
        }

        let body: ListApiKeysResponse = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        Ok(body.api_keys)
    }

    pub async fn create_apikey(&self, expiration: Option<&str>) -> Result<String, String> {
        let mut body = serde_json::json!({});
        if let Some(exp) = expiration {
            body["expiration"] = serde_json::Value::String(exp.to_string());
        }

        let resp = self
            .client
            .post(self.url("/api/v1/apikey"))
            .header("Authorization", self.auth_header())
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("HTTP {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
        }

        let body: CreateApiKeyResponse = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
        Ok(body.api_key)
    }

    pub async fn expire_apikey(&self, prefix: &str) -> Result<(), String> {
        let resp = self
            .client
            .post(self.url("/api/v1/apikey/expire"))
            .header("Authorization", self.auth_header())
            .json(&serde_json::json!({ "prefix": prefix }))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("HTTP {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
        }
        Ok(())
    }

    pub async fn delete_apikey(&self, prefix: &str) -> Result<(), String> {
        let resp = self
            .client
            .delete(self.url(&format!("/api/v1/apikey/{}", prefix)))
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("HTTP {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
        }
        Ok(())
    }
}

// ── Command Factories ──

pub fn cmd_load_users(client: HeadscaleClient) -> bubbletea_rs::Cmd {
    Box::pin(async move {
        match client.list_users().await {
            Ok(users) => Some(Box::new(UsersLoadedMsg { users }) as Msg),
            Err(e) => Some(Box::new(ApiErrorMsg {
                error: e,
                context: "loading users".to_string(),
            }) as Msg),
        }
    })
}

pub fn cmd_load_nodes(client: HeadscaleClient) -> bubbletea_rs::Cmd {
    Box::pin(async move {
        match client.list_nodes().await {
            Ok(nodes) => Some(Box::new(NodesLoadedMsg { nodes }) as Msg),
            Err(e) => Some(Box::new(ApiErrorMsg {
                error: e,
                context: "loading nodes".to_string(),
            }) as Msg),
        }
    })
}

pub fn cmd_load_preauthkeys(client: HeadscaleClient) -> bubbletea_rs::Cmd {
    Box::pin(async move {
        // Try to load preauthkeys without specifying a user first
        match client.list_preauthkeys(None).await {
            Ok(keys) => Some(Box::new(PreAuthKeysLoadedMsg { keys }) as Msg),
            Err(e) => {
                // If that fails, we'll display the error in the UI
                Some(Box::new(ApiErrorMsg {
                    error: e,
                    context: "loading preauthkeys".to_string(),
                }) as Msg)
            }
        }
    })
}

pub fn cmd_load_apikeys(client: HeadscaleClient) -> bubbletea_rs::Cmd {
    Box::pin(async move {
        match client.list_apikeys().await {
            Ok(keys) => Some(Box::new(ApiKeysLoadedMsg { keys }) as Msg),
            Err(e) => Some(Box::new(ApiErrorMsg {
                error: e,
                context: "loading apikeys".to_string(),
            }) as Msg),
        }
    })
}

pub fn cmd_delete_user(client: HeadscaleClient, id: String) -> bubbletea_rs::Cmd {
    Box::pin(async move {
        match client.delete_user(&id).await {
            Ok(()) => Some(Box::new(UserDeletedMsg { name: id }) as Msg),
            Err(e) => Some(Box::new(ApiErrorMsg {
                error: e,
                context: "deleting user".to_string(),
            }) as Msg),
        }
    })
}

pub fn cmd_delete_node(client: HeadscaleClient, id: String) -> bubbletea_rs::Cmd {
    Box::pin(async move {
        let id_clone = id.clone();
        match client.delete_node(&id).await {
            Ok(()) => Some(Box::new(NodeDeletedMsg { id: id_clone }) as Msg),
            Err(e) => Some(Box::new(ApiErrorMsg {
                error: e,
                context: "deleting node".to_string(),
            }) as Msg),
        }
    })
}

pub fn cmd_expire_node(client: HeadscaleClient, id: String) -> bubbletea_rs::Cmd {
    Box::pin(async move {
        let id_clone = id.clone();
        match client.expire_node(&id).await {
            Ok(()) => Some(Box::new(NodeExpiredMsg { id: id_clone }) as Msg),
            Err(e) => Some(Box::new(ApiErrorMsg {
                error: e,
                context: "expiring node".to_string(),
            }) as Msg),
        }
    })
}

pub fn cmd_expire_preauthkey(client: HeadscaleClient, user: String, key: String) -> bubbletea_rs::Cmd {
    Box::pin(async move {
        let key_clone = key.clone();
        match client.expire_preauthkey(&user, &key).await {
            Ok(()) => Some(Box::new(PreAuthKeyExpiredMsg { key: key_clone }) as Msg),
            Err(e) => Some(Box::new(ApiErrorMsg {
                error: e,
                context: "expiring preauthkey".to_string(),
            }) as Msg),
        }
    })
}

pub fn cmd_expire_apikey(client: HeadscaleClient, prefix: String) -> bubbletea_rs::Cmd {
    Box::pin(async move {
        let prefix_clone = prefix.clone();
        match client.expire_apikey(&prefix).await {
            Ok(()) => Some(Box::new(ApiKeyExpiredMsg { prefix: prefix_clone }) as Msg),
            Err(e) => Some(Box::new(ApiErrorMsg {
                error: e,
                context: "expiring apikey".to_string(),
            }) as Msg),
        }
    })
}

pub fn cmd_delete_apikey(client: HeadscaleClient, prefix: String) -> bubbletea_rs::Cmd {
    Box::pin(async move {
        let prefix_clone = prefix.clone();
        match client.delete_apikey(&prefix).await {
            Ok(()) => Some(Box::new(ApiKeyDeletedMsg { prefix: prefix_clone }) as Msg),
            Err(e) => Some(Box::new(ApiErrorMsg {
                error: e,
                context: "deleting apikey".to_string(),
            }) as Msg),
        }
    })
}

#[allow(dead_code)]
pub fn cmd_create_user(client: HeadscaleClient, name: String) -> bubbletea_rs::Cmd {
    Box::pin(async move {
        match client.create_user(&name).await {
            Ok(user) => Some(Box::new(UserCreatedMsg { user }) as Msg),
            Err(e) => Some(Box::new(ApiErrorMsg {
                error: e,
                context: "creating user".to_string(),
            }) as Msg),
        }
    })
}

#[allow(dead_code)]
pub fn cmd_rename_user(client: HeadscaleClient, old_id: String, new_name: String) -> bubbletea_rs::Cmd {
    Box::pin(async move {
        match client.rename_user(&old_id, &new_name).await {
            Ok(user) => Some(Box::new(UserCreatedMsg { user }) as Msg),
            Err(e) => Some(Box::new(ApiErrorMsg {
                error: e,
                context: "renaming user".to_string(),
            }) as Msg),
        }
    })
}

#[allow(dead_code)]
pub fn cmd_rename_node(client: HeadscaleClient, id: String, new_name: String) -> bubbletea_rs::Cmd {
    Box::pin(async move {
        let id_clone = id.clone();
        let name_clone = new_name.clone();
        match client.rename_node(&id, &new_name).await {
            Ok(()) => Some(Box::new(NodeRenamedMsg { id: id_clone, new_name: name_clone }) as Msg),
            Err(e) => Some(Box::new(ApiErrorMsg {
                error: e,
                context: "renaming node".to_string(),
            }) as Msg),
        }
    })
}

#[allow(dead_code)]
pub fn cmd_set_node_tags(client: HeadscaleClient, id: String, tags: Vec<String>) -> bubbletea_rs::Cmd {
    Box::pin(async move {
        let id_clone = id.clone();
        match client.set_node_tags(&id, tags).await {
            Ok(()) => Some(Box::new(StatusMsg {
                message: format!("Tags updated for node {}", id_clone),
            }) as Msg),
            Err(e) => Some(Box::new(ApiErrorMsg {
                error: e,
                context: "setting node tags".to_string(),
            }) as Msg),
        }
    })
}

#[allow(dead_code)]
pub fn cmd_approve_routes(client: HeadscaleClient, id: String, routes: Vec<String>) -> bubbletea_rs::Cmd {
    Box::pin(async move {
        let id_clone = id.clone();
        match client.approve_routes(&id, routes).await {
            Ok(()) => Some(Box::new(StatusMsg {
                message: format!("Routes approved for node {}", id_clone),
            }) as Msg),
            Err(e) => Some(Box::new(ApiErrorMsg {
                error: e,
                context: "approving routes".to_string(),
            }) as Msg),
        }
    })
}

#[allow(dead_code)]
pub fn cmd_create_preauthkey(client: HeadscaleClient, req: CreatePreAuthKeyRequest) -> bubbletea_rs::Cmd {
    Box::pin(async move {
        match client.create_preauthkey(&req).await {
            Ok(key) => Some(Box::new(PreAuthKeyCreatedMsg { key }) as Msg),
            Err(e) => Some(Box::new(ApiErrorMsg {
                error: e,
                context: "creating preauthkey".to_string(),
            }) as Msg),
        }
    })
}

#[allow(dead_code)]
pub fn cmd_create_apikey(client: HeadscaleClient, expiration: Option<String>) -> bubbletea_rs::Cmd {
    Box::pin(async move {
        match client.create_apikey(expiration.as_deref()).await {
            Ok(key) => Some(Box::new(ApiKeyCreatedMsg { key }) as Msg),
            Err(e) => Some(Box::new(ApiErrorMsg {
                error: e,
                context: "creating apikey".to_string(),
            }) as Msg),
        }
    })
}
