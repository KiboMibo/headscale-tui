use crate::api::types::*;
use crate::config::Config;
use crate::messages::*;
use bubbletea_rs::Msg;
use std::time::Duration;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone)]
pub struct HeadscaleClient {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl HeadscaleClient {
    pub fn new(config: &Config) -> Self {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            // Headscale endpoint can close idle connections aggressively.
            .pool_max_idle_per_host(0)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        HeadscaleClient {
            client,
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

    async fn resolve_user_reference(&self, user_ref: &str) -> Result<String, String> {
        let trimmed = user_ref.trim();
        if trimmed.is_empty() {
            return Err("user reference is empty".to_string());
        }
        if trimmed.chars().all(|c| c.is_ascii_digit()) {
            return Ok(trimmed.to_string());
        }

        let users = self.list_users().await?;

        if let Some(user) = users.iter().find(|u| u.name == trimmed) {
            return Ok(user.id.clone());
        }

        let mut matches: Vec<&User> = users
            .iter()
            .filter(|u| {
                u.name.eq_ignore_ascii_case(trimmed)
                    || u.email.eq_ignore_ascii_case(trimmed)
                    || u.display_name.eq_ignore_ascii_case(trimmed)
            })
            .collect();

        if matches.len() == 1 {
            return Ok(matches.remove(0).id.clone());
        }

        if matches.is_empty() {
            return Err(format!("user '{}' not found; use user id or exact user name", trimmed));
        }

        Err(format!(
            "user '{}' is ambiguous; use numeric user id",
            trimmed
        ))
    }

    // ── Users ──

    pub async fn list_users(&self) -> Result<Vec<User>, String> {
        let url = self.url("/api/v1/user");
        debug!(method = "GET", %url, "list_users request");

        let resp = self
            .client
            .get(url.clone())
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| {
                error!(%url, error = %e, "list_users request failed");
                format!("Request failed: {}", e)
            })?;

        let status = resp.status();
        debug!(%status, "list_users response");

        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            error!(%status, %body, "list_users HTTP error");
            return Err(format!("HTTP {}: {}", status, body));
        }

        let text = resp.text().await.map_err(|e| format!("Read error: {}", e))?;
        debug!(body = %text, "list_users response body");

        let body: ListUsersResponse = serde_json::from_str(&text)
            .map_err(|e| {
                error!(body = %text, error = %e, "list_users parse error");
                format!("Parse error: {}", e)
            })?;

        info!(count = body.users.len(), "list_users loaded");
        Ok(body.users)
    }

    pub async fn create_user(
        &self,
        name: &str,
        display_name: Option<&str>,
        email: Option<&str>,
    ) -> Result<User, String> {
        let url = self.url("/api/v1/user");
        let mut req_body = serde_json::json!({ "name": name });
        if let Some(display_name) = display_name.filter(|v| !v.trim().is_empty()) {
            req_body["display_name"] = serde_json::Value::String(display_name.to_string());
        }
        if let Some(email) = email.filter(|v| !v.trim().is_empty()) {
            req_body["email"] = serde_json::Value::String(email.to_string());
        }
        debug!(method = "POST", %url, body = %req_body, "create_user request");

        let resp = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(&req_body)
            .send()
            .await
            .map_err(|e| {
                error!(%url, error = %e, "create_user request failed");
                format!("Request failed: {}", e)
            })?;

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        debug!(%status, body = %text, "create_user response");

        if !status.is_success() {
            error!(%status, body = %text, "create_user HTTP error");
            return Err(format!("HTTP {}: {}", status, text));
        }

        let body: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| {
                error!(body = %text, error = %e, "create_user parse error");
                format!("Parse error: {}", e)
            })?;
        let user: User = serde_json::from_value(body["user"].clone())
            .map_err(|e| {
                error!(body = %text, error = %e, "create_user user parse error");
                format!("Parse error: {}", e)
            })?;
        info!(user_name = %user.name, "create_user success");
        Ok(user)
    }

    pub async fn delete_user(&self, id: &str) -> Result<(), String> {
        let url = self.url(&format!("/api/v1/user/{}", id));
        debug!(method = "DELETE", %url, %id, "delete_user request");

        let resp = self
            .client
            .delete(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| {
                error!(%url, error = %e, "delete_user request failed");
                format!("Request failed: {}", e)
            })?;

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        debug!(%status, body = %text, "delete_user response");

        if !status.is_success() {
            error!(%status, body = %text, "delete_user HTTP error");
            return Err(format!("HTTP {}: {}", status, text));
        }
        info!(%id, "delete_user success");
        Ok(())
    }

    pub async fn rename_user(&self, old_id: &str, new_name: &str) -> Result<User, String> {
        let url = self.url(&format!("/api/v1/user/{}/rename/{}", old_id, new_name));
        debug!(method = "POST", %url, %old_id, %new_name, "rename_user request");

        let resp = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| {
                error!(%url, error = %e, "rename_user request failed");
                format!("Request failed: {}", e)
            })?;

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        debug!(%status, body = %text, "rename_user response");

        if !status.is_success() {
            error!(%status, body = %text, "rename_user HTTP error");
            return Err(format!("HTTP {}: {}", status, text));
        }

        let body: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| {
                error!(body = %text, error = %e, "rename_user parse error");
                format!("Parse error: {}", e)
            })?;
        let user: User = serde_json::from_value(body["user"].clone())
            .map_err(|e| {
                error!(body = %text, error = %e, "rename_user user parse error");
                format!("Parse error: {}", e)
            })?;
        info!(user_name = %user.name, "rename_user success");
        Ok(user)
    }

    // ── Nodes ──

    pub async fn list_nodes(&self) -> Result<Vec<Node>, String> {
        let url = self.url("/api/v1/node");
        debug!(method = "GET", %url, "list_nodes request");

        let resp = self
            .client
            .get(url.clone())
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| {
                error!(%url, error = %e, "list_nodes request failed");
                format!("Request failed: {}", e)
            })?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            error!(%status, %body, "list_nodes HTTP error");
            return Err(format!("HTTP {}: {}", status, body));
        }

        let text = resp.text().await.map_err(|e| format!("Read error: {}", e))?;
        debug!(body = %text, "list_nodes response body");

        let body: ListNodesResponse = serde_json::from_str(&text)
            .map_err(|e| {
                error!(body = %text, error = %e, "list_nodes parse error");
                format!("Parse error: {}", e)
            })?;

        info!(count = body.nodes.len(), "list_nodes loaded");
        Ok(body.nodes)
    }

    pub async fn delete_node(&self, id: &str) -> Result<(), String> {
        let url = self.url(&format!("/api/v1/node/{}", id));
        debug!(method = "DELETE", %url, %id, "delete_node request");

        let resp = self
            .client
            .delete(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| {
                error!(%url, error = %e, "delete_node request failed");
                format!("Request failed: {}", e)
            })?;

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        debug!(%status, body = %text, "delete_node response");

        if !status.is_success() {
            error!(%status, body = %text, "delete_node HTTP error");
            return Err(format!("HTTP {}: {}", status, text));
        }
        info!(%id, "delete_node success");
        Ok(())
    }

    pub async fn expire_node(&self, id: &str) -> Result<(), String> {
        let url = self.url(&format!("/api/v1/node/{}/expire", id));
        debug!(method = "POST", %url, %id, "expire_node request");

        let resp = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| {
                error!(%url, error = %e, "expire_node request failed");
                format!("Request failed: {}", e)
            })?;

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        debug!(%status, body = %text, "expire_node response");

        if !status.is_success() {
            error!(%status, body = %text, "expire_node HTTP error");
            return Err(format!("HTTP {}: {}", status, text));
        }
        info!(%id, "expire_node success");
        Ok(())
    }

    pub async fn rename_node(&self, id: &str, new_name: &str) -> Result<(), String> {
        let url = self.url(&format!("/api/v1/node/{}/rename/{}", id, new_name));
        debug!(method = "POST", %url, %id, %new_name, "rename_node request");

        let resp = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| {
                error!(%url, error = %e, "rename_node request failed");
                format!("Request failed: {}", e)
            })?;

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        debug!(%status, body = %text, "rename_node response");

        if !status.is_success() {
            error!(%status, body = %text, "rename_node HTTP error");
            return Err(format!("HTTP {}: {}", status, text));
        }
        info!(%id, %new_name, "rename_node success");
        Ok(())
    }

    pub async fn set_node_tags(&self, id: &str, tags: Vec<String>) -> Result<(), String> {
        let url = self.url(&format!("/api/v1/node/{}/tags", id));
        let req_body = serde_json::json!({ "tags": tags });
        debug!(method = "POST", %url, %id, body = %req_body, "set_node_tags request");

        let resp = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(&req_body)
            .send()
            .await
            .map_err(|e| {
                error!(%url, error = %e, "set_node_tags request failed");
                format!("Request failed: {}", e)
            })?;

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        debug!(%status, body = %text, "set_node_tags response");

        if !status.is_success() {
            error!(%status, body = %text, "set_node_tags HTTP error");
            return Err(format!("HTTP {}: {}", status, text));
        }
        info!(%id, "set_node_tags success");
        Ok(())
    }

    pub async fn approve_routes(&self, id: &str, routes: Vec<String>) -> Result<(), String> {
        let url = self.url(&format!("/api/v1/node/{}/approve_routes", id));
        let req_body = serde_json::json!({ "routes": routes });
        debug!(method = "POST", %url, %id, body = %req_body, "approve_routes request");

        let resp = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(&req_body)
            .send()
            .await
            .map_err(|e| {
                error!(%url, error = %e, "approve_routes request failed");
                format!("Request failed: {}", e)
            })?;

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        debug!(%status, body = %text, "approve_routes response");

        if !status.is_success() {
            error!(%status, body = %text, "approve_routes HTTP error");
            return Err(format!("HTTP {}: {}", status, text));
        }
        info!(%id, "approve_routes success");
        Ok(())
    }

    // ── PreAuth Keys ──

    pub async fn list_preauthkeys(&self, user: Option<&str>) -> Result<Vec<PreAuthKey>, String> {
        if let Some(user_name) = user {
            return self.list_preauthkeys_for_user(user_name).await;
        }

        let users = self.list_users().await?;
        if users.is_empty() {
            return Ok(Vec::new());
        }

        let mut all_keys = Vec::new();
        let mut successful_users = 0usize;
        let mut errors = Vec::new();

        for user in users {
            let mut candidates = Vec::new();
            let user_name = user.name.trim();
            let user_id = user.id.trim();
            if !user_name.is_empty() {
                candidates.push(user_name.to_string());
            }
            if !user_id.is_empty() && user_id != user_name {
                candidates.push(user_id.to_string());
            }

            let mut loaded = false;
            let mut last_error = String::new();

            for candidate in candidates {
                match self.list_preauthkeys_for_user(&candidate).await {
                    Ok(user_keys) => {
                        all_keys.extend(user_keys);
                        successful_users += 1;
                        loaded = true;
                        break;
                    }
                    Err(err) => {
                        last_error = err.clone();
                        if !is_user_not_found_error(&err) {
                            warn!(candidate, error = %err, "list_preauthkeys candidate failed");
                        }
                    }
                }
            }

            if !loaded {
                let label = if user_name.is_empty() { user_id } else { user_name };
                errors.push(format!("{}: {}", label, last_error));
            }
        }

        if successful_users == 0 && !errors.is_empty() {
            return Err(format!("failed for all users: {}", errors.join("; ")));
        }

        if !errors.is_empty() {
            warn!(error_count = errors.len(), "list_preauthkeys partially failed");
        }

        info!(count = all_keys.len(), "list_preauthkeys loaded for all users");
        Ok(all_keys)
    }

    async fn list_preauthkeys_for_user(&self, user: &str) -> Result<Vec<PreAuthKey>, String> {
        let mut url = reqwest::Url::parse(&self.url("/api/v1/preauthkey"))
            .map_err(|e| format!("URL parse error: {}", e))?;
        url.query_pairs_mut().append_pair("user", user);
        debug!(method = "GET", %url, user, "list_preauthkeys_for_user request");

        let resp = self
            .client
            .get(url.clone())
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| {
                error!(%url, error = %e, "list_preauthkeys_for_user request failed");
                format!("Request failed: {}", e)
            })?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            error!(%status, %body, user, "list_preauthkeys_for_user HTTP error");
            return Err(format!("HTTP {}: {}", status, body));
        }

        let text = resp.text().await.map_err(|e| format!("Read error: {}", e))?;
        debug!(body = %text, user, "list_preauthkeys_for_user response body");

        let body: ListPreAuthKeysResponse = serde_json::from_str(&text)
            .map_err(|e| {
                error!(body = %text, error = %e, user, "list_preauthkeys_for_user parse error");
                format!("Parse error: {}", e)
            })?;

        info!(count = body.pre_auth_keys.len(), user, "list_preauthkeys_for_user loaded");
        Ok(body.pre_auth_keys)
    }

    pub async fn create_preauthkey(&self, req: &CreatePreAuthKeyRequest) -> Result<PreAuthKey, String> {
        let url = self.url("/api/v1/preauthkey");
        let mut req_body = req.clone();
        req_body.user = self.resolve_user_reference(&req.user).await?;

        let req_json = serde_json::to_string(&req_body).unwrap_or_default();
        debug!(method = "POST", %url, body = %req_json, "create_preauthkey request");

        let resp = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(&req_body)
            .send()
            .await
            .map_err(|e| {
                error!(%url, error = %e, "create_preauthkey request failed");
                format!("Request failed: {}", e)
            })?;

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        debug!(%status, body = %text, "create_preauthkey response");

        if !status.is_success() {
            error!(%status, body = %text, "create_preauthkey HTTP error");
            return Err(format!("HTTP {}: {}", status, text));
        }

        let body: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| {
                error!(body = %text, error = %e, "create_preauthkey parse error");
                format!("Parse error: {}", e)
            })?;
        let key: PreAuthKey = serde_json::from_value(body["preAuthKey"].clone())
            .map_err(|e| {
                error!(body = %text, error = %e, "create_preauthkey key parse error");
                format!("Parse error: {}", e)
            })?;
        info!("create_preauthkey success");
        Ok(key)
    }

    pub async fn expire_preauthkey(&self, user: &str, key: &str) -> Result<(), String> {
        let url = self.url("/api/v1/preauthkey/expire");
        let resolved_user = self.resolve_user_reference(user).await?;
        let req_body = serde_json::json!({ "user": resolved_user, "key": key });
        debug!(method = "POST", %url, %user, body = %req_body, "expire_preauthkey request");

        let resp = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(&req_body)
            .send()
            .await
            .map_err(|e| {
                error!(%url, error = %e, "expire_preauthkey request failed");
                format!("Request failed: {}", e)
            })?;

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        debug!(%status, body = %text, "expire_preauthkey response");

        if !status.is_success() {
            error!(%status, body = %text, "expire_preauthkey HTTP error");
            return Err(format!("HTTP {}: {}", status, text));
        }
        info!(%user, "expire_preauthkey success");
        Ok(())
    }

    // ── API Keys ──

    pub async fn list_apikeys(&self) -> Result<Vec<ApiKey>, String> {
        let url = self.url("/api/v1/apikey");
        debug!(method = "GET", %url, "list_apikeys request");

        let resp = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| {
                error!(%url, error = %e, "list_apikeys request failed");
                format!("Request failed: {}", e)
            })?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            error!(%status, %body, "list_apikeys HTTP error");
            return Err(format!("HTTP {}: {}", status, body));
        }

        let text = resp.text().await.map_err(|e| format!("Read error: {}", e))?;
        debug!(body = %text, "list_apikeys response body");

        let body: ListApiKeysResponse = serde_json::from_str(&text)
            .map_err(|e| {
                error!(body = %text, error = %e, "list_apikeys parse error");
                format!("Parse error: {}", e)
            })?;

        info!(count = body.api_keys.len(), "list_apikeys loaded");
        Ok(body.api_keys)
    }

    pub async fn create_apikey(&self, expiration: Option<&str>) -> Result<String, String> {
        let url = self.url("/api/v1/apikey");
        let mut req_body = serde_json::json!({});
        if let Some(exp) = expiration {
            req_body["expiration"] = serde_json::Value::String(exp.to_string());
        }
        debug!(method = "POST", %url, body = %req_body, "create_apikey request");

        let resp = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(&req_body)
            .send()
            .await
            .map_err(|e| {
                error!(%url, error = %e, "create_apikey request failed");
                format!("Request failed: {}", e)
            })?;

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        debug!(%status, body = %text, "create_apikey response");

        if !status.is_success() {
            error!(%status, body = %text, "create_apikey HTTP error");
            return Err(format!("HTTP {}: {}", status, text));
        }

        let parsed: CreateApiKeyResponse = serde_json::from_str(&text)
            .map_err(|e| {
                error!(body = %text, error = %e, "create_apikey parse error");
                format!("Parse error: {}", e)
            })?;
        info!("create_apikey success");
        Ok(parsed.api_key)
    }

    pub async fn expire_apikey(&self, prefix: &str) -> Result<(), String> {
        let url = self.url("/api/v1/apikey/expire");
        let req_body = serde_json::json!({ "prefix": prefix });
        debug!(method = "POST", %url, %prefix, body = %req_body, "expire_apikey request");

        let resp = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(&req_body)
            .send()
            .await
            .map_err(|e| {
                error!(%url, error = %e, "expire_apikey request failed");
                format!("Request failed: {}", e)
            })?;

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        debug!(%status, body = %text, "expire_apikey response");

        if !status.is_success() {
            error!(%status, body = %text, "expire_apikey HTTP error");
            return Err(format!("HTTP {}: {}", status, text));
        }
        info!(%prefix, "expire_apikey success");
        Ok(())
    }

    pub async fn delete_apikey(&self, prefix: &str) -> Result<(), String> {
        let url = self.url(&format!("/api/v1/apikey/{}", prefix));
        debug!(method = "DELETE", %url, %prefix, "delete_apikey request");

        let resp = self
            .client
            .delete(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| {
                error!(%url, error = %e, "delete_apikey request failed");
                format!("Request failed: {}", e)
            })?;

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        debug!(%status, body = %text, "delete_apikey response");

        if !status.is_success() {
            error!(%status, body = %text, "delete_apikey HTTP error");
            return Err(format!("HTTP {}: {}", status, text));
        }
        info!(%prefix, "delete_apikey success");
        Ok(())
    }
}

fn is_user_not_found_error(err: &str) -> bool {
    err.contains("user not found")
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
pub fn cmd_create_user(
    client: HeadscaleClient,
    name: String,
    display_name: Option<String>,
    email: Option<String>,
) -> bubbletea_rs::Cmd {
    Box::pin(async move {
        match client
            .create_user(&name, display_name.as_deref(), email.as_deref())
            .await
        {
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
