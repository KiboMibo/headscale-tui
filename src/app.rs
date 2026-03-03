use bubbletea_rs::{Cmd, Model, Msg, KeyMsg, WindowSizeMsg, batch, quit};
use crossterm::event::KeyCode;
use lipgloss_extras::lipgloss::{Color, Style};

use crate::api::client::*;
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
        };

        let load_cmd = cmd_load_users(client);
        Some(load_cmd);

        let client2 = app.client.clone();
        let init_cmd: Cmd = Box::pin(async move {
            Some(Box::new(InitRenderMsg) as Msg)
        });

        (app, Some(batch(vec![cmd_load_users(client2), init_cmd])))
    }

    fn update(&mut self, msg: Msg) -> Option<Cmd> {
        // Window resize
        if let Some(size) = msg.downcast_ref::<WindowSizeMsg>() {
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
            return None;
        }

        // API responses
        if let Some(m) = msg.downcast_ref::<UsersLoadedMsg>() {
            self.users_view.users = m.users.clone();
            self.users_view.loading = false;
            self.users_view.error = None;
            return None;
        }
        if let Some(m) = msg.downcast_ref::<NodesLoadedMsg>() {
            self.nodes_view.nodes = m.nodes.clone();
            self.nodes_view.loading = false;
            self.nodes_view.error = None;
            return None;
        }
        if let Some(m) = msg.downcast_ref::<PreAuthKeysLoadedMsg>() {
            self.preauthkeys_view.keys = m.keys.clone();
            self.preauthkeys_view.loading = false;
            self.preauthkeys_view.error = None;
            return None;
        }
        if let Some(m) = msg.downcast_ref::<ApiKeysLoadedMsg>() {
            self.apikeys_view.keys = m.keys.clone();
            self.apikeys_view.loading = false;
            self.apikeys_view.error = None;
            return None;
        }

        // API errors
        if let Some(m) = msg.downcast_ref::<ApiErrorMsg>() {
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

        // Mutation success messages → refresh
        if msg.downcast_ref::<UserCreatedMsg>().is_some()
            || msg.downcast_ref::<UserDeletedMsg>().is_some()
        {
            self.status_message = Some("Operation successful".to_string());
            self.users_view.loading = true;
            self.users_view.confirm_delete = false;
            return Some(cmd_load_users(self.client.clone()));
        }
        if msg.downcast_ref::<NodeDeletedMsg>().is_some()
            || msg.downcast_ref::<NodeExpiredMsg>().is_some()
            || msg.downcast_ref::<NodeRenamedMsg>().is_some()
        {
            self.status_message = Some("Operation successful".to_string());
            self.nodes_view.loading = true;
            self.nodes_view.confirm_delete = false;
            return Some(cmd_load_nodes(self.client.clone()));
        }
        if msg.downcast_ref::<PreAuthKeyCreatedMsg>().is_some()
            || msg.downcast_ref::<PreAuthKeyExpiredMsg>().is_some()
        {
            self.status_message = Some("Operation successful".to_string());
            self.preauthkeys_view.loading = true;
            return Some(cmd_load_preauthkeys(self.client.clone()));
        }
        if msg.downcast_ref::<ApiKeyCreatedMsg>().is_some()
            || msg.downcast_ref::<ApiKeyExpiredMsg>().is_some()
            || msg.downcast_ref::<ApiKeyDeletedMsg>().is_some()
        {
            self.status_message = Some("Operation successful".to_string());
            self.apikeys_view.loading = true;
            self.apikeys_view.confirm_delete = false;
            return Some(cmd_load_apikeys(self.client.clone()));
        }

        // Refresh
        if msg.downcast_ref::<RefreshMsg>().is_some() {
            return self.load_current_tab();
        }

        // Key events
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
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

        output
    }
}

impl App {
    fn handle_key(&mut self, key: &KeyMsg) -> Option<Cmd> {
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
                            let user_name = key.user.as_ref().map(|u| u.name.clone()).unwrap_or_default();
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
}
