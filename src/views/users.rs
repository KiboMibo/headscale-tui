use crate::api::types::User;
use crate::theme;
use lipgloss_extras::lipgloss::{thick_border, Color, Style};
use lipgloss_extras::table::{Table, HEADER_ROW};

#[derive(Debug, Clone)]
pub struct UsersView {
    pub users: Vec<User>,
    pub selected: usize,
    pub loading: bool,
    pub error: Option<String>,
    pub width: u16,
    pub confirm_delete: bool,
}

impl UsersView {
    pub fn new() -> Self {
        UsersView {
            users: Vec::new(),
            selected: 0,
            loading: true,
            error: None,
            width: 80,
            confirm_delete: false,
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if !self.users.is_empty() && self.selected < self.users.len() - 1 {
            self.selected += 1;
        }
    }

    pub fn selected_user(&self) -> Option<&User> {
        self.users.get(self.selected)
    }

    pub fn view(&self) -> String {
        if self.loading {
            return Style::new()
                .foreground(Color(theme::MUTED.to_string()))
                .padding_left(2)
                .padding_top(1)
                .render("  Loading users...");
        }

        if let Some(ref err) = self.error {
            return Style::new()
                .foreground(Color(theme::DANGER.to_string()))
                .padding_left(2)
                .padding_top(1)
                .render(&format!("  Error: {}", err));
        }

        if self.users.is_empty() {
            return Style::new()
                .foreground(Color(theme::MUTED.to_string()))
                .padding_left(2)
                .padding_top(1)
                .render("  No users found. Press 'c' to create one.");
        }

        let headers = vec![
            "ID".to_string(),
            "Name".to_string(),
            "Display Name".to_string(),
            "Email".to_string(),
            "Provider".to_string(),
            "Created".to_string(),
        ];

        let rows: Vec<Vec<String>> = self
            .users
            .iter()
            .map(|u| {
                vec![
                    u.id.to_string(),
                    u.name.clone(),
                    u.display_name.clone(),
                    u.email.clone(),
                    u.provider.clone(),
                    format_time(&u.created_at),
                ]
            })
            .collect();

        let selected = self.selected;
        let w = (self.width as i32 - 4).max(40);

        let mut output = Table::new()
            .headers(headers)
            .rows(rows)
            .border(thick_border())
            .width(w)
            .style_func_boxed(Box::new(move |row, _col| {
                if row == HEADER_ROW {
                    Style::new()
                        .foreground(Color(theme::PRIMARY.to_string()))
                        .bold(true)
                } else if (row as usize).wrapping_sub(1) == selected {
                    Style::new()
                        .foreground(Color(theme::TEXT.to_string()))
                        .background(Color(theme::BG_HIGHLIGHT.to_string()))
                        .bold(true)
                } else {
                    Style::new().foreground(Color(theme::TEXT_DIM.to_string()))
                }
            }))
            .render();

        if self.confirm_delete {
            if let Some(user) = self.users.get(self.selected) {
                output.push_str(&format!(
                    "\n  {}",
                    Style::new()
                        .foreground(Color(theme::DANGER.to_string()))
                        .bold(true)
                        .render(&format!("Delete user '{}'? (y/n)", user.name))
                ));
            }
        }

        let help = Style::new()
            .foreground(Color(theme::MUTED.to_string()))
            .render("  ↑/↓ navigate • c create • d delete • r refresh • q quit");

        format!("{}\n{}", output, help)
    }
}

fn format_time(ts: &str) -> String {
    if ts.is_empty() {
        return "-".to_string();
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
        dt.format("%Y-%m-%d %H:%M").to_string()
    } else {
        ts.chars().take(16).collect()
    }
}
