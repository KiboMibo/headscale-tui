#![allow(dead_code)]

use lipgloss_extras::lipgloss::{Color, Style};

pub const PRIMARY: &str = "#7C3AED";
pub const SECONDARY: &str = "#06B6D4";
pub const ACCENT: &str = "#F59E0B";
pub const SUCCESS: &str = "#10B981";
pub const DANGER: &str = "#EF4444";
pub const WARNING: &str = "#F59E0B";
pub const MUTED: &str = "#6B7280";
pub const TEXT: &str = "#F9FAFB";
pub const TEXT_DIM: &str = "#9CA3AF";
pub const BG_DARK: &str = "#111827";
pub const BG_HIGHLIGHT: &str = "#1F2937";
pub const BORDER: &str = "#374151";
pub const TAB_ACTIVE: &str = "#7C3AED";
pub const TAB_INACTIVE: &str = "#374151";
pub const ONLINE: &str = "#10B981";
pub const OFFLINE: &str = "#EF4444";
pub fn title_style() -> Style {
    Style::new().foreground(Color::from(PRIMARY)).bold(true)
}

pub fn status_online() -> Style {
    Style::new().foreground(Color::from(ONLINE)).bold(true)
}

pub fn status_offline() -> Style {
    Style::new().foreground(Color::from(OFFLINE))
}

pub fn tab_active_style() -> Style {
    Style::new()
        .foreground(Color::from(TEXT))
        .background(Color::from(TAB_ACTIVE))
        .bold(true)
        .padding_left(2)
        .padding_right(2)
}

pub fn tab_inactive_style() -> Style {
    Style::new()
        .foreground(Color::from(TEXT_DIM))
        .background(Color::from(TAB_INACTIVE))
        .padding_left(2)
        .padding_right(2)
}

pub fn help_style() -> Style {
    Style::new().foreground(Color::from(MUTED))
}

pub fn error_style() -> Style {
    Style::new().foreground(Color::from(DANGER)).bold(true)
}

pub fn success_style() -> Style {
    Style::new().foreground(Color::from(SUCCESS))
}

pub fn header_style() -> Style {
    Style::new().foreground(Color::from(SECONDARY)).bold(true)
}
