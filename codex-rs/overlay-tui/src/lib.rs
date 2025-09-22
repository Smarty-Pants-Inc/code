//! TUI overlay helpers. Provides seam points for branding and SDK-specific UI
//! tweaks so we can keep upstream widgets untouched.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;

/// Styling primitives for the welcome banner. Text segments stay identical to
/// upstream; only presentation varies per overlay.
#[derive(Debug, Clone, Copy)]
pub struct WelcomeBannerStyles {
    pub prefix: Style,
    pub product: Style,
    pub suffix: Style,
}

impl WelcomeBannerStyles {
    pub fn default() -> Self {
        let accent = Style::default().fg(Color::LightCyan);
        Self {
            prefix: accent,
            product: accent.add_modifier(Modifier::BOLD),
            suffix: Style::default(),
        }
    }
}

pub fn welcome_banner_styles() -> WelcomeBannerStyles {
    WelcomeBannerStyles::default()
}

/// Styled span for the footer context percentage indicator.
pub fn context_footer_span(percent_remaining: u8) -> Span<'static> {
    let style = if percent_remaining < 20 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().add_modifier(Modifier::DIM)
    };
    Span::styled(format!("{percent_remaining}% context left"), style)
}
