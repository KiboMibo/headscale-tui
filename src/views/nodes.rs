use crate::api::types::Node;
use crate::theme;
use lipgloss_extras::lipgloss::{thick_border, Color, Style};
use lipgloss_extras::table::{Table, HEADER_ROW};

#[derive(Debug, Clone)]
pub struct NodesView {
    pub nodes: Vec<Node>,
    pub selected: usize,
    pub loading: bool,
    pub error: Option<String>,
    pub width: u16,
    pub confirm_delete: bool,
}

impl NodesView {
    pub fn new() -> Self {
        NodesView {
            nodes: Vec::new(),
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
        if !self.nodes.is_empty() && self.selected < self.nodes.len() - 1 {
            self.selected += 1;
        }
    }

    pub fn selected_node(&self) -> Option<&Node> {
        self.nodes.get(self.selected)
    }

    pub fn view(&self) -> String {
        if self.loading {
            return Style::new()
                .foreground(Color(theme::MUTED.to_string()))
                .padding_left(2)
                .padding_top(1)
                .render("  Loading nodes...");
        }

        if let Some(ref err) = self.error {
            return Style::new()
                .foreground(Color(theme::DANGER.to_string()))
                .padding_left(2)
                .padding_top(1)
                .render(&format!("  Error: {}", err));
        }

        if self.nodes.is_empty() {
            return Style::new()
                .foreground(Color(theme::MUTED.to_string()))
                .padding_left(2)
                .padding_top(1)
                .render("  No nodes found.");
        }

        let headers = vec![
            "ID".to_string(),
            "Name".to_string(),
            "User".to_string(),
            "IP".to_string(),
            "Online".to_string(),
            "Last Seen".to_string(),
            "Tags".to_string(),
        ];

        let rows: Vec<Vec<String>> = self
            .nodes
            .iter()
            .map(|n| {
                let status = if n.online { "●" } else { "○" };
                let user_name = n.user.as_ref().map(|u| u.name.clone()).unwrap_or_default();
                let ips = n.ip_addresses.join(", ");
                let tags = n.tags.join(", ");

                vec![
                    n.id.to_string(),
                    n.given_name.clone(),
                    user_name,
                    ips,
                    status.to_string(),
                    format_time(&n.last_seen),
                    tags,
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
            if let Some(node) = self.nodes.get(self.selected) {
                output.push_str(&format!(
                    "\n  {}",
                    Style::new()
                        .foreground(Color(theme::DANGER.to_string()))
                        .bold(true)
                        .render(&format!("Delete node '{}'? (y/n)", node.given_name))
                ));
            }
        }

        let help = Style::new()
            .foreground(Color(theme::MUTED.to_string()))
            .render("  ↑/↓ navigate • d delete • e expire • r refresh • q quit");

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
