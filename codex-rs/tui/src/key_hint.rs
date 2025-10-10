<<<<<<< HEAD
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::text::Span;
use std::fmt::Display;

#[cfg(test)]
const ALT_PREFIX: &str = "⌥";
#[cfg(all(not(test), target_os = "macos"))]
const ALT_PREFIX: &str = "⌥";
#[cfg(all(not(test), not(target_os = "macos")))]
const ALT_PREFIX: &str = "Alt+";

#[cfg(test)]
const CTRL_PREFIX: &str = "⌃";
#[cfg(all(not(test), target_os = "macos"))]
const CTRL_PREFIX: &str = "⌃";
#[cfg(all(not(test), not(target_os = "macos")))]
const CTRL_PREFIX: &str = "Ctrl+";

#[cfg(test)]
const SHIFT_PREFIX: &str = "⇧";
#[cfg(all(not(test), target_os = "macos"))]
const SHIFT_PREFIX: &str = "⇧";
#[cfg(all(not(test), not(target_os = "macos")))]
const SHIFT_PREFIX: &str = "Shift+";

fn key_hint_style() -> Style {
    Style::default().fg(Color::Cyan)
}

fn modifier_span(prefix: &str, key: impl Display) -> Span<'static> {
    Span::styled(format!("{prefix}{key}"), key_hint_style())
}

pub(crate) fn ctrl(key: impl Display) -> Span<'static> {
    modifier_span(CTRL_PREFIX, key)
}

pub(crate) fn alt(key: impl Display) -> Span<'static> {
    modifier_span(ALT_PREFIX, key)
}

pub(crate) fn shift(key: impl Display) -> Span<'static> {
    modifier_span(SHIFT_PREFIX, key)
}

pub(crate) fn plain(key: impl Display) -> Span<'static> {
    Span::styled(format!("{key}"), key_hint_style())
=======
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::text::Span;

const ALT_PREFIX: &str = "alt + ";
const CTRL_PREFIX: &str = "ctrl + ";
const SHIFT_PREFIX: &str = "shift + ";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KeyBinding {
    key: KeyCode,
    modifiers: KeyModifiers,
}

impl KeyBinding {
    pub(crate) const fn new(key: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { key, modifiers }
    }

    pub fn is_press(&self, event: KeyEvent) -> bool {
        self.key == event.code
            && self.modifiers == event.modifiers
            && (event.kind == KeyEventKind::Press || event.kind == KeyEventKind::Repeat)
    }
}

pub(crate) const fn plain(key: KeyCode) -> KeyBinding {
    KeyBinding::new(key, KeyModifiers::NONE)
}

pub(crate) const fn alt(key: KeyCode) -> KeyBinding {
    KeyBinding::new(key, KeyModifiers::ALT)
}

pub(crate) const fn shift(key: KeyCode) -> KeyBinding {
    KeyBinding::new(key, KeyModifiers::SHIFT)
}

pub(crate) const fn ctrl(key: KeyCode) -> KeyBinding {
    KeyBinding::new(key, KeyModifiers::CONTROL)
}

fn modifiers_to_string(modifiers: KeyModifiers) -> String {
    let mut result = String::new();
    if modifiers.contains(KeyModifiers::CONTROL) {
        result.push_str(CTRL_PREFIX);
    }
    if modifiers.contains(KeyModifiers::SHIFT) {
        result.push_str(SHIFT_PREFIX);
    }
    if modifiers.contains(KeyModifiers::ALT) {
        result.push_str(ALT_PREFIX);
    }
    result
}

impl From<KeyBinding> for Span<'static> {
    fn from(binding: KeyBinding) -> Self {
        (&binding).into()
    }
}
impl From<&KeyBinding> for Span<'static> {
    fn from(binding: &KeyBinding) -> Self {
        let KeyBinding { key, modifiers } = binding;
        let modifiers = modifiers_to_string(*modifiers);
        let key = match key {
            KeyCode::Enter => "enter".to_string(),
            KeyCode::Up => "↑".to_string(),
            KeyCode::Down => "↓".to_string(),
            KeyCode::Left => "←".to_string(),
            KeyCode::Right => "→".to_string(),
            KeyCode::PageUp => "pgup".to_string(),
            KeyCode::PageDown => "pgdn".to_string(),
            _ => format!("{key}").to_ascii_lowercase(),
        };
        Span::styled(format!("{modifiers}{key}"), key_hint_style())
    }
}

fn key_hint_style() -> Style {
    Style::default().dim()
>>>>>>> upstream/main
}
