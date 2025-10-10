use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
<<<<<<< HEAD
#[allow(unused_imports)]
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
=======
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Clear;
>>>>>>> upstream/main
use ratatui::widgets::Paragraph;
use ratatui::widgets::WidgetRef;
use ratatui::widgets::Wrap;

<<<<<<< HEAD
use crate::frames::FRAME_TICK_DEFAULT;
use crate::frames::FRAMES_DEFAULT;
=======
use crate::ascii_animation::AsciiAnimation;
use crate::onboarding::onboarding_screen::KeyboardHandler;
>>>>>>> upstream/main
use crate::onboarding::onboarding_screen::StepStateProvider;
use crate::tui::FrameRequester;

use super::onboarding_screen::StepState;
use std::time::Duration;
use std::time::Instant;

const FRAME_TICK: Duration = FRAME_TICK_DEFAULT;
const MIN_ANIMATION_HEIGHT: u16 = 20;
const MIN_ANIMATION_WIDTH: u16 = 60;

const MIN_ANIMATION_HEIGHT: u16 = 20;
const MIN_ANIMATION_WIDTH: u16 = 60;

pub(crate) struct WelcomeWidget {
    pub is_logged_in: bool,
<<<<<<< HEAD
    pub request_frame: FrameRequester,
    pub start: Instant,
=======
    animation: AsciiAnimation,
}

impl KeyboardHandler for WelcomeWidget {
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if key_event.kind == KeyEventKind::Press
            && key_event.code == KeyCode::Char('.')
            && key_event.modifiers.contains(KeyModifiers::CONTROL)
        {
            tracing::warn!("Welcome background to press '.'");
            let _ = self.animation.pick_random_variant();
        }
    }
}

impl WelcomeWidget {
    pub(crate) fn new(is_logged_in: bool, request_frame: FrameRequester) -> Self {
        Self {
            is_logged_in,
            animation: AsciiAnimation::new(request_frame),
        }
    }
>>>>>>> upstream/main
}

impl WidgetRef for &WelcomeWidget {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
<<<<<<< HEAD
        let elapsed_ms = self.start.elapsed().as_millis();

        // Align next draw to the next FRAME_TICK boundary to reduce jitter.
        {
            let tick_ms = FRAME_TICK.as_millis();
            let rem_ms = elapsed_ms % tick_ms;
            let delay_ms = if rem_ms == 0 {
                tick_ms
            } else {
                tick_ms - rem_ms
            };
            // Safe cast: delay_ms < tick_ms and FRAME_TICK is small.
            self.request_frame
                .schedule_frame_in(Duration::from_millis(delay_ms as u64));
        }

        let frames = &FRAMES_DEFAULT;
        let idx = ((elapsed_ms / FRAME_TICK.as_millis()) % frames.len() as u128) as usize;
        // Skip the animation entirely when the viewport is too small so we don't clip frames.
        let show_animation =
            area.height >= MIN_ANIMATION_HEIGHT && area.width >= MIN_ANIMATION_WIDTH;

        let mut lines: Vec<Line> = Vec::new();
        if show_animation {
            let frame_line_count = frames[idx].lines().count();
            lines.reserve(frame_line_count + 2);
            lines.extend(frames[idx].lines().map(|l| l.into()));
            lines.push("".into());
        }
        #[cfg(feature = "smarty-sdk")]
        let banner_styles = smarty_sdk_overlay_tui::welcome_banner_styles();
        #[cfg(feature = "smarty-sdk")]
        let (prefix_style, product_style, suffix_style) = (
            banner_styles.prefix,
            banner_styles.product,
            banner_styles.suffix,
        );
        #[cfg(not(feature = "smarty-sdk"))]
        let (prefix_style, product_style, suffix_style) = (
            Style::default(),
            Style::default().add_modifier(Modifier::BOLD),
            Style::default(),
        );

        lines.push(Line::from(vec![
            Span::styled("  Welcome to ", prefix_style),
            Span::styled("Codex", product_style),
            Span::styled(", OpenAI's command-line coding agent", suffix_style),
=======
        Clear.render(area, buf);
        self.animation.schedule_next_frame();

        // Skip the animation entirely when the viewport is too small so we don't clip frames.
        let show_animation =
            area.height >= MIN_ANIMATION_HEIGHT && area.width >= MIN_ANIMATION_WIDTH;

        let mut lines: Vec<Line> = Vec::new();
        if show_animation {
            let frame = self.animation.current_frame();
            // let frame_line_count = frame.lines().count();
            // lines.reserve(frame_line_count + 2);
            lines.extend(frame.lines().map(Into::into));
            lines.push("".into());
        }
        lines.push(Line::from(vec![
            "  ".into(),
            "Welcome to ".into(),
            "Codex".bold(),
            ", OpenAI's command-line coding agent".into(),
>>>>>>> upstream/main
        ]));

        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }
}

impl StepStateProvider for WelcomeWidget {
    fn get_step_state(&self) -> StepState {
        match self.is_logged_in {
            true => StepState::Hidden,
            false => StepState::Complete,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
<<<<<<< HEAD

    /// A number of things break down if FRAME_TICK is zero.
    #[test]
    fn frame_tick_must_be_nonzero() {
        assert!(FRAME_TICK.as_millis() > 0);
=======
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    static VARIANT_A: [&str; 1] = ["frame-a"];
    static VARIANT_B: [&str; 1] = ["frame-b"];
    static VARIANTS: [&[&str]; 2] = [&VARIANT_A, &VARIANT_B];

    #[test]
    fn welcome_renders_animation_on_first_draw() {
        let widget = WelcomeWidget::new(false, FrameRequester::test_dummy());
        let area = Rect::new(0, 0, MIN_ANIMATION_WIDTH, MIN_ANIMATION_HEIGHT);
        let mut buf = Buffer::empty(area);
        (&widget).render(area, &mut buf);

        let mut found = false;
        let mut last_non_empty: Option<u16> = None;
        for y in 0..area.height {
            for x in 0..area.width {
                if !buf[(x, y)].symbol().trim().is_empty() {
                    found = true;
                    last_non_empty = Some(y);
                    break;
                }
            }
        }

        assert!(found, "expected welcome animation to render characters");
        let measured_rows = last_non_empty.map(|v| v + 2).unwrap_or(0);
        assert!(
            measured_rows >= MIN_ANIMATION_HEIGHT,
            "expected measurement to report at least {MIN_ANIMATION_HEIGHT} rows, got {measured_rows}"
        );
    }

    #[test]
    fn ctrl_dot_changes_animation_variant() {
        let mut widget = WelcomeWidget {
            is_logged_in: false,
            animation: AsciiAnimation::with_variants(FrameRequester::test_dummy(), &VARIANTS, 0),
        };

        let before = widget.animation.current_frame();
        widget.handle_key_event(KeyEvent::new(KeyCode::Char('.'), KeyModifiers::CONTROL));
        let after = widget.animation.current_frame();

        assert_ne!(
            before, after,
            "expected ctrl+. to switch welcome animation variant"
        );
>>>>>>> upstream/main
    }
}
