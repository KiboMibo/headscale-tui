use crate::api::types::PreAuthKey;
use crate::theme;
use lipgloss_extras::lipgloss::{thick_border, Color, Style};
use lipgloss_extras::table::{Table, HEADER_ROW};

#[derive(Debug, Clone)]
pub struct PreAuthKeysView {
    pub keys: Vec<PreAuthKey>,
    pub selected: usize,
    pub loading: bool,
    pub error: Option<String>,
    pub width: u16,
}

impl PreAuthKeysView {
    pub fn new() -> Self {
        PreAuthKeysView {
            keys: Vec::new(),
            selected: 0,
            loading: true,
            error: None,
            width: 160,
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if !self.keys.is_empty() && self.selected < self.keys.len() - 1 {
            self.selected += 1;
        }
    }

    pub fn selected_key(&self) -> Option<&PreAuthKey> {
        self.keys.get(self.selected)
    }

    pub fn view(&self) -> String {
        if self.loading {
            return Style::new()
                .foreground(Color(theme::MUTED.to_string()))
                .padding_left(2)
                .padding_top(1)
                .render("  Loading pre-auth keys...");
        }

        if let Some(ref err) = self.error {
            return Style::new()
                .foreground(Color(theme::DANGER.to_string()))
                .padding_left(2)
                .padding_top(1)
                .render(&format!("  Error: {}", err));
        }

        if self.keys.is_empty() {
            return Style::new()
                .foreground(Color(theme::MUTED.to_string()))
                .padding_left(2)
                .padding_top(1)
                .render("  No pre-auth keys found. Press 'c' to create one.");
        }

        let headers = vec![
            "ID".to_string(),
            "Key".to_string(),
            "User".to_string(),
            "Reusable".to_string(),
            "Ephemeral".to_string(),
            "Used".to_string(),
            "Expiration".to_string(),
            "ACL Tags".to_string(),
        ];

        let rows: Vec<Vec<String>> = self
            .keys
            .iter()
            .map(|k| {
                let user_name = k
                    .user
                    .as_ref()
                    .map(|u| u.name.clone())
                    .unwrap_or_else(|| "N/A".to_string());
                let key_preview = if k.key.len() > 12 {
                    format!("{}...", &k.key[..12])
                } else {
                    k.key.clone()
                };
                // Limit tags display to prevent table overflow
                let tags = if k.acl_tags.len() > 3 {
                    format!("{} tags...", k.acl_tags.len())
                } else {
                    k.acl_tags.join(", ")
                };

                vec![
                    k.id.to_string(),
                    key_preview,
                    user_name,
                    bool_symbol(k.reusable),
                    bool_symbol(k.ephemeral),
                    bool_symbol(k.used),
                    format_time(&k.expiration),
                    tags,
                ]
            })
            .collect();

        let selected = self.selected;
        let w = (self.width as i32 - 4).max(40);

        let output = Table::new()
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

        let help = Style::new()
            .foreground(Color(theme::MUTED.to_string()))
            .render("  ↑/↓ navigate • c create • e expire • r refresh • q quit");

        format!("{}\n{}", output, help)
    }
}

fn bool_symbol(v: bool) -> String {
    if v {
        "✓".to_string()
    } else {
        "✗".to_string()
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
