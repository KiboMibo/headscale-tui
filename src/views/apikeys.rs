use crate::api::types::ApiKey;
use crate::theme;
use lipgloss_extras::lipgloss::{thick_border, Color, Style};
use lipgloss_extras::table::{Table, HEADER_ROW};

#[derive(Debug, Clone)]
pub struct ApiKeysView {
    pub keys: Vec<ApiKey>,
    pub selected: usize,
    pub loading: bool,
    pub error: Option<String>,
    pub width: u16,
    pub confirm_delete: bool,
}

impl ApiKeysView {
    pub fn new() -> Self {
        ApiKeysView {
            keys: Vec::new(),
            selected: 0,
            loading: true,
            error: None,
            width: 160,
            confirm_delete: false,
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

    pub fn selected_key(&self) -> Option<&ApiKey> {
        self.keys.get(self.selected)
    }

    pub fn view(&self) -> String {
        if self.loading {
            return Style::new()
                .foreground(Color(theme::MUTED.to_string()))
                .padding_left(2)
                .padding_top(1)
                .render("  Loading API keys...");
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
                .render("  No API keys found. Press 'c' to create one.");
        }

        let headers = vec![
            "ID".to_string(),
            "Prefix".to_string(),
            "Expiration".to_string(),
            "Created".to_string(),
            "Last Seen".to_string(),
        ];

        let rows: Vec<Vec<String>> = self
            .keys
            .iter()
            .map(|k| {
                vec![
                    k.id.to_string(),
                    k.prefix.clone(),
                    format_time(k.expiration.as_deref()),
                    format_time(k.created_at.as_deref()),
                    format_time(k.last_seen.as_deref()),
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
            if let Some(key) = self.keys.get(self.selected) {
                output.push_str(&format!(
                    "\n  {}",
                    Style::new()
                        .foreground(Color(theme::DANGER.to_string()))
                        .bold(true)
                        .render(&format!("Delete API key '{}'? (y/n)", key.prefix))
                ));
            }
        }

        let help = Style::new()
            .foreground(Color(theme::MUTED.to_string()))
            .render("  ↑/↓ navigate • Enter copy prefix • c create • d delete • e expire • r refresh • q quit");

        format!("{}\n{}", output, help)
    }
}

fn format_time(ts: Option<&str>) -> String {
    let Some(ts) = ts else {
        return "-".to_string();
    };
    if ts.is_empty() {
        return "-".to_string();
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
        dt.format("%Y-%m-%d %H:%M").to_string()
    } else {
        ts.chars().take(16).collect()
    }
}
