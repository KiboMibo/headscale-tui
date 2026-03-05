use bubbletea_rs::{Cmd, Model, Msg, KeyMsg, WindowSizeMsg, batch, quit};
use crossterm::event::KeyCode;
use lipgloss_extras::lipgloss::{Color, Style};
use tracing::{debug, error};

use crate::api::client::*;
use crate::api::types::CreatePreAuthKeyRequest;
use crate::config::Config;
use crate::messages::*;
use crate::theme;
use crate::views::users::UsersView;
use crate::views::nodes::NodesView;
use crate::views::preauthkeys::PreAuthKeysView;
use crate::views::apikeys::ApiKeysView;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tab {
    Users,
    Nodes,
    PreAuthKeys,
    ApiKeys,
}

impl Tab {
    fn all() -> &'static [Tab] {
        &[Tab::Users, Tab::Nodes, Tab::PreAuthKeys, Tab::ApiKeys]
    }

    fn label(&self) -> &'static str {
        match self {
            Tab::Users => " Users ",
            Tab::Nodes => " Nodes ",
            Tab::PreAuthKeys => " PreAuth Keys ",
            Tab::ApiKeys => " API Keys ",
        }
    }

    fn index(&self) -> usize {
        match self {
            Tab::Users => 0,
            Tab::Nodes => 1,
            Tab::PreAuthKeys => 2,
            Tab::ApiKeys => 3,
        }
    }

    fn from_index(i: usize) -> Tab {
        match i {
            0 => Tab::Users,
            1 => Tab::Nodes,
            2 => Tab::PreAuthKeys,
            3 => Tab::ApiKeys,
            _ => Tab::Users,
        }
    }
}

pub struct App {
    active_tab: Tab,
    client: HeadscaleClient,
    users_view: UsersView,
    nodes_view: NodesView,
    preauthkeys_view: PreAuthKeysView,
    apikeys_view: ApiKeysView,
    width: u16,
    height: u16,
    status_message: Option<String>,
    input_mode: Option<CreateInputMode>,
    input_buffer: String,
    pending_user_name: String,
    pending_user_display_name: String,
    last_created_api_key: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CreateInputMode {
    UserName,
    UserDisplayName,
    UserEmail,
    PreAuthKey,
    ApiKey,
}

impl Model for App {
    fn init() -> (Self, Option<Cmd>) {
        let config = Config::from_env().unwrap_or_else(|e| {
            eprintln!("Configuration error: {}", e);
            eprintln!("Set HEADSCALE_URL and HEADSCALE_API_KEY environment variables");
            std::process::exit(1);
        });

        let client = HeadscaleClient::new(&config);

        let app = App {
            active_tab: Tab::Users,
            client: client.clone(),
            users_view: UsersView::new(),
            nodes_view: NodesView::new(),
            preauthkeys_view: PreAuthKeysView::new(),
            apikeys_view: ApiKeysView::new(),
            width: 80,
            height: 24,
            status_message: None,
            input_mode: None,
            input_buffer: String::new(),
            pending_user_name: String::new(),
            pending_user_display_name: String::new(),
            last_created_api_key: None,
        };

        let client2 = app.client.clone();
        let init_cmd: Cmd = Box::pin(async move {
            Some(Box::new(InitRenderMsg) as Msg)
        });

        (app, Some(batch(vec![cmd_load_users(client2), init_cmd])))
    }

    fn update(&mut self, msg: Msg) -> Option<Cmd> {
        debug!(msg_type = ?msg.type_id(), "App::update received message");
        
        // Window resize
        if let Some(size) = msg.downcast_ref::<WindowSizeMsg>() {
            debug!(width = size.width, height = size.height, "Window resize event");
            self.width = size.width;
            self.height = size.height;
            self.users_view.width = size.width;
            self.nodes_view.width = size.width;
            self.preauthkeys_view.width = size.width;
            self.apikeys_view.width = size.width;
            return None;
        }

        // Init render
        if msg.downcast_ref::<InitRenderMsg>().is_some() {
            debug!("Init render message received");
            return None;
        }

        // API responses
        if let Some(m) = msg.downcast_ref::<UsersLoadedMsg>() {
            debug!(count = m.users.len(), "Users loaded");
            self.users_view.users = m.users.clone();
            self.users_view.loading = false;
            self.users_view.error = None;
            return None;
        }
        if let Some(m) = msg.downcast_ref::<NodesLoadedMsg>() {
            debug!(count = m.nodes.len(), "Nodes loaded");
            self.nodes_view.nodes = m.nodes.clone();
            self.nodes_view.loading = false;
            self.nodes_view.error = None;
            return None;
        }
        if let Some(m) = msg.downcast_ref::<PreAuthKeysLoadedMsg>() {
            debug!(count = m.keys.len(), "PreAuth keys loaded");
            self.preauthkeys_view.keys = m.keys.clone();
            self.preauthkeys_view.loading = false;
            self.preauthkeys_view.error = None;
            return None;
        }
        if let Some(m) = msg.downcast_ref::<ApiKeysLoadedMsg>() {
            debug!(count = m.keys.len(), "API keys loaded");
            self.apikeys_view.keys = m.keys.clone();
            self.apikeys_view.loading = false;
            self.apikeys_view.error = None;
            return None;
        }

        // API errors
        if let Some(m) = msg.downcast_ref::<ApiErrorMsg>() {
            error!(context = %m.context, error = %m.error, "API error occurred");
            let err = format!("{}: {}", m.context, m.error);
            match self.active_tab {
                Tab::Users => {
                    self.users_view.loading = false;
                    self.users_view.error = Some(err);
                }
                Tab::Nodes => {
                    self.nodes_view.loading = false;
                    self.nodes_view.error = Some(err);
                }
                Tab::PreAuthKeys => {
                    self.preauthkeys_view.loading = false;
                    self.preauthkeys_view.error = Some(err);
                }
                Tab::ApiKeys => {
                    self.apikeys_view.loading = false;
                    self.apikeys_view.error = Some(err);
                }
            }
            return None;
        }
        if let Some(m) = msg.downcast_ref::<StatusMsg>() {
            self.status_message = Some(m.message.clone());
            return None;
        }

        // Mutation success messages → refresh
        if msg.downcast_ref::<UserCreatedMsg>().is_some()
            || msg.downcast_ref::<UserDeletedMsg>().is_some()
        {
            debug!("User mutation success, refreshing users");
            self.status_message = Some("Operation successful".to_string());
            self.users_view.loading = true;
            self.users_view.confirm_delete = false;
            return Some(cmd_load_users(self.client.clone()));
        }
        if msg.downcast_ref::<NodeDeletedMsg>().is_some()
            || msg.downcast_ref::<NodeExpiredMsg>().is_some()
            || msg.downcast_ref::<NodeRenamedMsg>().is_some()
        {
            debug!("Node mutation success, refreshing nodes");
            self.status_message = Some("Operation successful".to_string());
            self.nodes_view.loading = true;
            self.nodes_view.confirm_delete = false;
            return Some(cmd_load_nodes(self.client.clone()));
        }
        if msg.downcast_ref::<PreAuthKeyCreatedMsg>().is_some()
            || msg.downcast_ref::<PreAuthKeyExpiredMsg>().is_some()
        {
            debug!("PreAuthKey mutation success, refreshing preauthkeys");
            self.status_message = Some("Operation successful".to_string());
            self.preauthkeys_view.loading = true;
            return Some(cmd_load_preauthkeys(self.client.clone()));
        }
        if let Some(m) = msg.downcast_ref::<ApiKeyCreatedMsg>() {
            debug!("ApiKey created, refreshing apikeys");
            self.last_created_api_key = Some(m.key.clone());
            self.status_message = match copy_to_clipboard(&m.key) {
                Ok(()) => Some("API key created and copied to clipboard".to_string()),
                Err(e) => Some(format!(
                    "API key created, but clipboard copy failed: {}",
                    e
                )),
            };
            self.apikeys_view.loading = true;
            return Some(cmd_load_apikeys(self.client.clone()));
        }
        if msg.downcast_ref::<ApiKeyExpiredMsg>().is_some()
            || msg.downcast_ref::<ApiKeyDeletedMsg>().is_some()
        {
            debug!("ApiKey mutation success, refreshing apikeys");
            self.status_message = Some("Operation successful".to_string());
            self.apikeys_view.loading = true;
            self.apikeys_view.confirm_delete = false;
            return Some(cmd_load_apikeys(self.client.clone()));
        }

        // Refresh
        if msg.downcast_ref::<RefreshMsg>().is_some() {
            debug!("Refresh message received");
            return self.load_current_tab();
        }

        // Key events
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            debug!(key = ?key.key, "Key event received");
            return self.handle_key(key);
        }

        None
    }

    fn view(&self) -> String {
        let tabs = self.render_tabs();

        let title = Style::new()
            .foreground(Color(theme::PRIMARY.to_string()))
            .bold(true)
            .padding_left(1)
            .render("🔐 Headscale TUI");

        let content = match self.active_tab {
            Tab::Users => self.users_view.view(),
            Tab::Nodes => self.nodes_view.view(),
            Tab::PreAuthKeys => self.preauthkeys_view.view(),
            Tab::ApiKeys => self.apikeys_view.view(),
        };

        let mut output = format!("{}\n{}\n{}", title, tabs, content);

        if let Some(ref status) = self.status_message {
            output.push_str(&format!(
                "\n  {}",
                Style::new()
                    .foreground(Color(theme::SUCCESS.to_string()))
                    .render(status)
            ));
        }

        if let Some(mode) = self.input_mode {
            output.push_str(&format!(
                "\n  {}",
                Style::new()
                    .foreground(Color(theme::PRIMARY.to_string()))
                    .bold(true)
                    .render(self.input_prompt(mode))
            ));
            output.push_str(&format!(
                "\n  {}{}",
                Style::new().foreground(Color(theme::TEXT_DIM.to_string())).render("> "),
                self.input_buffer
            ));
        }

        output
    }
}

impl App {
    fn handle_key(&mut self, key: &KeyMsg) -> Option<Cmd> {
        if self.input_mode.is_some() {
            return self.handle_create_input(key);
        }

        // Handle confirm dialogs first
        if self.users_view.confirm_delete {
            return self.handle_confirm_delete_user(key);
        }
        if self.nodes_view.confirm_delete {
            return self.handle_confirm_delete_node(key);
        }
        if self.apikeys_view.confirm_delete {
            return self.handle_confirm_delete_apikey(key);
        }

        match key.key {
            KeyCode::Char('q') => return Some(quit()),
            KeyCode::Esc => return Some(quit()),

            // Tab navigation
            KeyCode::Tab | KeyCode::Right => {
                let next = (self.active_tab.index() + 1) % Tab::all().len();
                self.active_tab = Tab::from_index(next);
                self.status_message = None;
                return self.load_current_tab_if_empty();
            }
            KeyCode::BackTab | KeyCode::Left => {
                let prev = if self.active_tab.index() == 0 {
                    Tab::all().len() - 1
                } else {
                    self.active_tab.index() - 1
                };
                self.active_tab = Tab::from_index(prev);
                self.status_message = None;
                return self.load_current_tab_if_empty();
            }

            // Tab shortcuts
            KeyCode::Char('1') => {
                self.active_tab = Tab::Users;
                self.status_message = None;
                return self.load_current_tab_if_empty();
            }
            KeyCode::Char('2') => {
                self.active_tab = Tab::Nodes;
                self.status_message = None;
                return self.load_current_tab_if_empty();
            }
            KeyCode::Char('3') => {
                self.active_tab = Tab::PreAuthKeys;
                self.status_message = None;
                return self.load_current_tab_if_empty();
            }
            KeyCode::Char('4') => {
                self.active_tab = Tab::ApiKeys;
                self.status_message = None;
                return self.load_current_tab_if_empty();
            }

            // List navigation
            KeyCode::Up | KeyCode::Char('k') => {
                match self.active_tab {
                    Tab::Users => self.users_view.move_up(),
                    Tab::Nodes => self.nodes_view.move_up(),
                    Tab::PreAuthKeys => self.preauthkeys_view.move_up(),
                    Tab::ApiKeys => self.apikeys_view.move_up(),
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                match self.active_tab {
                    Tab::Users => self.users_view.move_down(),
                    Tab::Nodes => self.nodes_view.move_down(),
                    Tab::PreAuthKeys => self.preauthkeys_view.move_down(),
                    Tab::ApiKeys => self.apikeys_view.move_down(),
                }
            }

            // Refresh
            KeyCode::Char('r') => {
                self.status_message = None;
                return self.load_current_tab();
            }

            // Copy selected key/prefix
            KeyCode::Enter => {
                match self.active_tab {
                    Tab::PreAuthKeys => {
                        if let Some(key) = self.preauthkeys_view.selected_key() {
                            return Some(self.copy_to_clipboard_cmd(
                                key.key.clone(),
                                "PreAuth key copied to clipboard".to_string(),
                            ));
                        }
                    }
                    Tab::ApiKeys => {
                        if let Some(key) = self.apikeys_view.selected_key() {
                            let (value, full_key) = self.api_key_value_for_copy(&key.prefix);
                            return Some(self.copy_to_clipboard_cmd(value, if full_key {
                                "API key copied to clipboard".to_string()
                            } else {
                                "Only API key prefix available, copied prefix".to_string()
                            }));
                        }
                    }
                    _ => {}
                }
            }

            // Create
            KeyCode::Char('c') => {
                self.status_message = None;
                match self.active_tab {
                    Tab::Users => {
                        self.input_mode = Some(CreateInputMode::UserName);
                        self.input_buffer.clear();
                        self.pending_user_name.clear();
                        self.pending_user_display_name.clear();
                    }
                    Tab::PreAuthKeys => {
                        self.input_mode = Some(CreateInputMode::PreAuthKey);
                        self.input_buffer.clear();
                        if let Some(user_name) = self
                            .users_view
                            .selected_user()
                            .map(|u| u.name.trim().to_string())
                            .filter(|name| !name.is_empty())
                        {
                            self.input_buffer = user_name;
                        }
                    }
                    Tab::ApiKeys => {
                        self.input_mode = Some(CreateInputMode::ApiKey);
                        self.input_buffer.clear();
                    }
                    Tab::Nodes => {}
                }
            }

            // Delete
            KeyCode::Char('d') => {
                match self.active_tab {
                    Tab::Users => {
                        if self.users_view.selected_user().is_some() {
                            self.users_view.confirm_delete = true;
                        }
                    }
                    Tab::Nodes => {
                        if self.nodes_view.selected_node().is_some() {
                            self.nodes_view.confirm_delete = true;
                        }
                    }
                    Tab::ApiKeys => {
                        if self.apikeys_view.selected_key().is_some() {
                            self.apikeys_view.confirm_delete = true;
                        }
                    }
                    _ => {}
                }
            }

            // Expire
            KeyCode::Char('e') => {
                match self.active_tab {
                    Tab::Nodes => {
                        if let Some(node) = self.nodes_view.selected_node() {
                            let id = node.id.clone();
                            return Some(cmd_expire_node(self.client.clone(), id));
                        }
                    }
                    Tab::PreAuthKeys => {
                        if let Some(key) = self.preauthkeys_view.selected_key() {
                            let user_name = key
                                .user
                                .as_ref()
                                .map(|u| {
                                    if u.name.trim().is_empty() {
                                        u.id.clone()
                                    } else {
                                        u.name.clone()
                                    }
                                })
                                .unwrap_or_default();
                            let key_str = key.key.clone();
                            return Some(cmd_expire_preauthkey(
                                self.client.clone(),
                                user_name,
                                key_str,
                            ));
                        }
                    }
                    Tab::ApiKeys => {
                        if let Some(key) = self.apikeys_view.selected_key() {
                            let prefix = key.prefix.clone();
                            return Some(cmd_expire_apikey(self.client.clone(), prefix));
                        }
                    }
                    _ => {}
                }
            }

            _ => {}
        }

        None
    }

    fn handle_create_input(&mut self, key: &KeyMsg) -> Option<Cmd> {
        let mode = match self.input_mode {
            Some(mode) => mode,
            None => return None,
        };

        match key.key {
            KeyCode::Esc => {
                self.input_mode = None;
                self.input_buffer.clear();
                self.pending_user_name.clear();
                self.pending_user_display_name.clear();
                self.status_message = Some("Creation canceled".to_string());
                None
            }
            KeyCode::Enter => {
                match mode {
                    CreateInputMode::UserName => {
                        let name = self.input_buffer.trim().to_string();
                        if name.is_empty() {
                            self.status_message = Some("User name cannot be empty".to_string());
                            return None;
                        }
                        self.pending_user_name = name;
                        self.input_mode = Some(CreateInputMode::UserDisplayName);
                        self.input_buffer.clear();
                        self.status_message = None;
                        None
                    }
                    CreateInputMode::UserDisplayName => {
                        self.pending_user_display_name = self.input_buffer.trim().to_string();
                        self.input_mode = Some(CreateInputMode::UserEmail);
                        self.input_buffer.clear();
                        self.status_message = None;
                        None
                    }
                    CreateInputMode::UserEmail => {
                        let email = self.input_buffer.trim().to_string();
                        let name = self.pending_user_name.trim().to_string();
                        let display_name = self.pending_user_display_name.trim().to_string();

                        self.input_mode = None;
                        self.input_buffer.clear();
                        self.pending_user_name.clear();
                        self.pending_user_display_name.clear();
                        self.users_view.loading = true;
                        self.status_message = None;

                        let display_name = if display_name.is_empty() {
                            None
                        } else {
                            Some(display_name)
                        };
                        let email = if email.is_empty() { None } else { Some(email) };

                        Some(cmd_create_user(self.client.clone(), name, display_name, email))
                    }
                    CreateInputMode::PreAuthKey => {
                        let input = self.input_buffer.trim().to_string();
                        match parse_preauth_create_input(&input) {
                            Ok(req) => {
                                self.input_mode = None;
                                self.input_buffer.clear();
                                self.preauthkeys_view.loading = true;
                                self.status_message = None;
                                Some(cmd_create_preauthkey(self.client.clone(), req))
                            }
                            Err(e) => {
                                self.status_message = Some(e);
                                None
                            }
                        }
                    }
                    CreateInputMode::ApiKey => {
                        let expiration = self.input_buffer.trim();
                        let expiration = if expiration.is_empty() {
                            None
                        } else {
                            Some(expiration.to_string())
                        };
                        self.input_mode = None;
                        self.input_buffer.clear();
                        self.apikeys_view.loading = true;
                        self.status_message = None;
                        Some(cmd_create_apikey(self.client.clone(), expiration))
                    }
                }
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
                None
            }
            KeyCode::Char(ch) => {
                if !ch.is_control() {
                    self.input_buffer.push(ch);
                }
                None
            }
            _ => None,
        }
    }

    fn input_prompt(&self, mode: CreateInputMode) -> &'static str {
        match mode {
            CreateInputMode::UserName => "Create user (step 1/3): login name",
            CreateInputMode::UserDisplayName => "Create user (step 2/3): display name (can be empty)",
            CreateInputMode::UserEmail => "Create user (step 3/3): email (can be empty)",
            CreateInputMode::PreAuthKey => {
                "Create preauth key: <user> [reusable] [ephemeral] [expiration=<RFC3339>] [tags=tag1,tag2]"
            }
            CreateInputMode::ApiKey => {
                "Create API key: optional expiration RFC3339 (empty = server default)"
            }
        }
    }

    fn handle_confirm_delete_user(&mut self, key: &KeyMsg) -> Option<Cmd> {
        match key.key {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Some(user) = self.users_view.selected_user() {
                    let id = user.id.clone();
                    self.users_view.confirm_delete = false;
                    return Some(cmd_delete_user(self.client.clone(), id));
                }
                self.users_view.confirm_delete = false;
            }
            _ => {
                self.users_view.confirm_delete = false;
            }
        }
        None
    }

    fn handle_confirm_delete_node(&mut self, key: &KeyMsg) -> Option<Cmd> {
        match key.key {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Some(node) = self.nodes_view.selected_node() {
                    let id = node.id.clone();
                    self.nodes_view.confirm_delete = false;
                    return Some(cmd_delete_node(self.client.clone(), id));
                }
                self.nodes_view.confirm_delete = false;
            }
            _ => {
                self.nodes_view.confirm_delete = false;
            }
        }
        None
    }

    fn handle_confirm_delete_apikey(&mut self, key: &KeyMsg) -> Option<Cmd> {
        match key.key {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Some(key) = self.apikeys_view.selected_key() {
                    let prefix = key.prefix.clone();
                    self.apikeys_view.confirm_delete = false;
                    return Some(cmd_delete_apikey(self.client.clone(), prefix));
                }
                self.apikeys_view.confirm_delete = false;
            }
            _ => {
                self.apikeys_view.confirm_delete = false;
            }
        }
        None
    }

    fn load_current_tab(&mut self) -> Option<Cmd> {
        match self.active_tab {
            Tab::Users => {
                self.users_view.loading = true;
                Some(cmd_load_users(self.client.clone()))
            }
            Tab::Nodes => {
                self.nodes_view.loading = true;
                Some(cmd_load_nodes(self.client.clone()))
            }
            Tab::PreAuthKeys => {
                self.preauthkeys_view.loading = true;
                Some(cmd_load_preauthkeys(self.client.clone()))
            }
            Tab::ApiKeys => {
                self.apikeys_view.loading = true;
                Some(cmd_load_apikeys(self.client.clone()))
            }
        }
    }

    fn load_current_tab_if_empty(&mut self) -> Option<Cmd> {
        let needs_load = match self.active_tab {
            Tab::Users => self.users_view.users.is_empty() && self.users_view.error.is_none(),
            Tab::Nodes => self.nodes_view.nodes.is_empty() && self.nodes_view.error.is_none(),
            Tab::PreAuthKeys => {
                self.preauthkeys_view.keys.is_empty() && self.preauthkeys_view.error.is_none()
            }
            Tab::ApiKeys => {
                self.apikeys_view.keys.is_empty() && self.apikeys_view.error.is_none()
            }
        };

        if needs_load {
            self.load_current_tab()
        } else {
            None
        }
    }

    fn render_tabs(&self) -> String {
        let tabs: Vec<String> = Tab::all()
            .iter()
            .map(|tab| {
                if *tab == self.active_tab {
                    Style::new()
                        .foreground(Color(theme::TEXT.to_string()))
                        .background(Color(theme::TAB_ACTIVE.to_string()))
                        .bold(true)
                        .padding_left(1)
                        .padding_right(1)
                        .render(tab.label())
                } else {
                    Style::new()
                        .foreground(Color(theme::TEXT_DIM.to_string()))
                        .background(Color(theme::TAB_INACTIVE.to_string()))
                        .padding_left(1)
                        .padding_right(1)
                        .render(tab.label())
                }
            })
            .collect();

        format!("  {}", tabs.join(" "))
    }

    fn copy_to_clipboard_cmd(&self, value: String, success_message: String) -> Cmd {
        Box::pin(async move {
            match copy_to_clipboard(&value) {
                Ok(()) => Some(Box::new(StatusMsg { message: success_message }) as Msg),
                Err(e) => Some(Box::new(StatusMsg {
                    message: format!("Copy failed: {}", e),
                }) as Msg),
            }
        })
    }

    fn api_key_value_for_copy(&self, selected_prefix: &str) -> (String, bool) {
        if let Some(full_key) = self.last_created_api_key.as_ref() {
            if full_key.starts_with(selected_prefix) {
                return (full_key.clone(), true);
            }
        }
        (selected_prefix.to_string(), false)
    }
}

fn parse_preauth_create_input(input: &str) -> Result<CreatePreAuthKeyRequest, String> {
    let mut parts = input.split_whitespace();
    let user = parts
        .next()
        .ok_or_else(|| "User is required, format: <user> [reusable] [ephemeral] [expiration=<RFC3339>] [tags=a,b]".to_string())?
        .to_string();

    let mut reusable = false;
    let mut ephemeral = false;
    let mut expiration: Option<String> = None;
    let mut acl_tags: Option<Vec<String>> = None;

    for token in parts {
        if token.eq_ignore_ascii_case("reusable") {
            reusable = true;
            continue;
        }
        if token.eq_ignore_ascii_case("ephemeral") {
            ephemeral = true;
            continue;
        }
        if let Some(value) = token.strip_prefix("expiration=") {
            if value.is_empty() {
                return Err("expiration= requires RFC3339 value".to_string());
            }
            expiration = Some(value.to_string());
            continue;
        }
        if let Some(value) = token.strip_prefix("tags=") {
            let tags: Vec<String> = value
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
            if tags.is_empty() {
                return Err("tags= requires comma-separated values".to_string());
            }
            acl_tags = Some(tags);
            continue;
        }
        return Err(format!("Unknown token '{}'", token));
    }

    Ok(CreatePreAuthKeyRequest {
        user,
        reusable,
        ephemeral,
        expiration,
        acl_tags,
    })
}

fn copy_to_clipboard(value: &str) -> Result<(), String> {
    let mut clipboard = arboard::Clipboard::new()
        .map_err(|e| format!("clipboard unavailable: {}", e))?;
    clipboard
        .set_text(value.to_string())
        .map_err(|e| format!("clipboard write failed: {}", e))
}
