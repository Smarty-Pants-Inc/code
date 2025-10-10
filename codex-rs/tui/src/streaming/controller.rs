<<<<<<< HEAD
use crate::history_cell;
use crate::history_cell::HistoryCell;
use codex_core::config::Config;
use ratatui::text::Line;

use super::HeaderEmitter;
use super::StreamState;

/// Sink for history insertions and animation control.
pub(crate) trait HistorySink {
    fn insert_history_cell(&self, cell: Box<dyn HistoryCell>);
    fn start_commit_animation(&self);
    fn stop_commit_animation(&self);
}

/// Concrete sink backed by `AppEventSender`.
pub(crate) struct AppEventHistorySink(pub(crate) crate::app_event_sender::AppEventSender);

impl HistorySink for AppEventHistorySink {
    fn insert_history_cell(&self, cell: Box<dyn crate::history_cell::HistoryCell>) {
        self.0
            .send(crate::app_event::AppEvent::InsertHistoryCell(cell))
    }
    fn start_commit_animation(&self) {
        self.0
            .send(crate::app_event::AppEvent::StartCommitAnimation)
    }
    fn stop_commit_animation(&self) {
        self.0.send(crate::app_event::AppEvent::StopCommitAnimation)
    }
}

type Lines = Vec<Line<'static>>;

=======
use crate::history_cell::HistoryCell;
use crate::history_cell::{self};
use codex_core::config::Config;
use ratatui::text::Line;

use super::StreamState;

>>>>>>> upstream/main
/// Controller that manages newline-gated streaming, header emission, and
/// commit animation across streams.
pub(crate) struct StreamController {
    config: Config,
<<<<<<< HEAD
    header: HeaderEmitter,
    state: StreamState,
    active: bool,
    finishing_after_drain: bool,
=======
    state: StreamState,
    finishing_after_drain: bool,
    header_emitted: bool,
>>>>>>> upstream/main
}

impl StreamController {
    pub(crate) fn new(config: Config, width: Option<usize>) -> Self {
        Self {
            config,
<<<<<<< HEAD
            header: HeaderEmitter::new(),
            state: StreamState::new(),
            active: false,
            finishing_after_drain: false,
        }
    }

    pub(crate) fn reset_headers_for_new_turn(&mut self) {
        self.header.reset_for_new_turn();
    }

    pub(crate) fn is_write_cycle_active(&self) -> bool {
        self.active
    }

    pub(crate) fn clear_all(&mut self) {
        self.state.clear();
        self.active = false;
        self.finishing_after_drain = false;
        // leave header state unchanged; caller decides when to reset
    }

    /// Begin an answer stream. Does not emit header yet; it is emitted on first commit.
    pub(crate) fn begin(&mut self, _sink: &impl HistorySink) {
        // Starting a new stream cancels any pending finish-from-previous-stream animation.
        if !self.active {
            self.header.reset_for_stream();
        }
        self.finishing_after_drain = false;
        self.active = true;
    }

    /// Push a delta; if it contains a newline, commit completed lines and start animation.
    pub(crate) fn push_and_maybe_commit(&mut self, delta: &str, sink: &impl HistorySink) {
        if !self.active {
            return;
        }
        let cfg = self.config.clone();
        let state = &mut self.state;
        // Record that at least one delta was received for this stream
        if !delta.is_empty() {
            state.has_seen_delta = true;
        }
=======
            state: StreamState::new(width),
            finishing_after_drain: false,
            header_emitted: false,
        }
    }

    /// Push a delta; if it contains a newline, commit completed lines and start animation.
    pub(crate) fn push(&mut self, delta: &str) -> bool {
        let cfg = self.config.clone();
        let state = &mut self.state;
        if !delta.is_empty() {
            state.has_seen_delta = true;
        }
>>>>>>> upstream/main
        state.collector.push_delta(delta);
        if delta.contains('\n') {
            let newly_completed = state.collector.commit_complete_lines(&cfg);
            if !newly_completed.is_empty() {
                state.enqueue(newly_completed);
<<<<<<< HEAD
                sink.start_commit_animation();
            }
        }
    }

    /// Finalize the active stream. If `flush_immediately` is true, drain and emit now.
    pub(crate) fn finalize(&mut self, flush_immediately: bool, sink: &impl HistorySink) -> bool {
        if !self.active {
            return false;
        }
        let cfg = self.config.clone();
        // Finalize collector first.
        let remaining = {
            let state = &mut self.state;
            state.collector.finalize_and_drain(&cfg)
        };
        if flush_immediately {
            // Collect all output first to avoid emitting headers when there is no content.
            let mut out_lines: Lines = Vec::new();
            {
                let state = &mut self.state;
                if !remaining.is_empty() {
                    state.enqueue(remaining);
                }
                let step = state.drain_all();
                out_lines.extend(step.history);
            }
            if !out_lines.is_empty() {
                // Insert as a HistoryCell so display drops the header while transcript keeps it.
                sink.insert_history_cell(Box::new(history_cell::AgentMessageCell::new(
                    out_lines,
                    self.header.maybe_emit_header(),
                )));
            }

            // Cleanup
            self.state.clear();
            // Allow a subsequent block in this turn to emit its header.
            self.header.allow_reemit_in_turn();
            // Also clear the per-stream emitted flag so the header can render again.
            self.header.reset_for_stream();
            self.active = false;
            self.finishing_after_drain = false;
            true
        } else {
            if !remaining.is_empty() {
                let state = &mut self.state;
                state.enqueue(remaining);
            }
            // Spacer animated out
            self.state.enqueue(vec![Line::from("")]);
            self.finishing_after_drain = true;
            sink.start_commit_animation();
            false
        }
    }

    /// Step animation: commit at most one queued line and handle end-of-drain cleanup.
    pub(crate) fn on_commit_tick(&mut self, sink: &impl HistorySink) -> bool {
        if !self.active {
            return false;
        }
        let step = { self.state.step() };
        if !step.history.is_empty() {
            sink.insert_history_cell(Box::new(history_cell::AgentMessageCell::new(
                step.history,
                self.header.maybe_emit_header(),
            )));
        }

        let is_idle = self.state.is_idle();
        if is_idle {
            sink.stop_commit_animation();
            if self.finishing_after_drain {
                // Reset and notify
                self.state.clear();
                // Allow a subsequent block in this turn to emit its header.
                self.header.allow_reemit_in_turn();
                // Also clear the per-stream emitted flag so the header can render again.
                self.header.reset_for_stream();
                self.active = false;
                self.finishing_after_drain = false;
=======
>>>>>>> upstream/main
                return true;
            }
        }
        false
    }

<<<<<<< HEAD
    /// Apply a full final answer: replace queued content with only the remaining tail,
    /// then finalize immediately and notify completion.
    pub(crate) fn apply_final_answer(&mut self, message: &str, sink: &impl HistorySink) -> bool {
        self.apply_full_final(message, sink)
    }

    fn apply_full_final(&mut self, message: &str, sink: &impl HistorySink) -> bool {
        self.begin(sink);

        {
            let state = &mut self.state;
            // Only inject the final full message if we have not seen any deltas for this stream.
            // If deltas were received, rely on the collector's existing buffer to avoid duplication.
            if !state.has_seen_delta && !message.is_empty() {
                // normalize to end with newline
                let mut msg = message.to_owned();
                if !msg.ends_with('\n') {
                    msg.push('\n');
                }

                // replace while preserving already committed count
                let committed = state.collector.committed_count();
                state
                    .collector
                    .replace_with_and_mark_committed(&msg, committed);
=======
    /// Finalize the active stream. Drain and emit now.
    pub(crate) fn finalize(&mut self) -> Option<Box<dyn HistoryCell>> {
        let cfg = self.config.clone();
        // Finalize collector first.
        let remaining = {
            let state = &mut self.state;
            state.collector.finalize_and_drain(&cfg)
        };
        // Collect all output first to avoid emitting headers when there is no content.
        let mut out_lines = Vec::new();
        {
            let state = &mut self.state;
            if !remaining.is_empty() {
                state.enqueue(remaining);
>>>>>>> upstream/main
            }
            let step = state.drain_all();
            out_lines.extend(step);
        }
<<<<<<< HEAD
        self.finalize(true, sink)
=======

        // Cleanup
        self.state.clear();
        self.finishing_after_drain = false;
        self.emit(out_lines)
    }

    /// Step animation: commit at most one queued line and handle end-of-drain cleanup.
    pub(crate) fn on_commit_tick(&mut self) -> (Option<Box<dyn HistoryCell>>, bool) {
        let step = self.state.step();
        (self.emit(step), self.state.is_idle())
    }

    fn emit(&mut self, lines: Vec<Line<'static>>) -> Option<Box<dyn HistoryCell>> {
        if lines.is_empty() {
            return None;
        }
        Some(Box::new(history_cell::AgentMessageCell::new(lines, {
            let header_emitted = self.header_emitted;
            self.header_emitted = true;
            !header_emitted
        })))
>>>>>>> upstream/main
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_core::config::Config;
    use codex_core::config::ConfigOverrides;

    async fn test_config() -> Config {
        let overrides = ConfigOverrides {
            cwd: std::env::current_dir().ok(),
            ..Default::default()
        };
<<<<<<< HEAD
        match Config::load_with_cli_overrides(vec![], overrides) {
            Ok(c) => c,
            Err(e) => panic!("load test config: {e}"),
        }
    }

    struct TestSink {
        pub lines: RefCell<Vec<Vec<Line<'static>>>>,
    }
    impl TestSink {
        fn new() -> Self {
            Self {
                lines: RefCell::new(Vec::new()),
            }
        }
    }
    impl HistorySink for TestSink {
        fn insert_history_cell(&self, cell: Box<dyn crate::history_cell::HistoryCell>) {
            // For tests, store the transcript representation of the cell.
            self.lines.borrow_mut().push(cell.transcript_lines());
        }
        fn start_commit_animation(&self) {}
        fn stop_commit_animation(&self) {}
=======
        Config::load_with_cli_overrides(vec![], overrides)
            .await
            .expect("load test config")
>>>>>>> upstream/main
    }

    fn lines_to_plain_strings(lines: &[ratatui::text::Line<'_>]) -> Vec<String> {
        lines
            .iter()
            .map(|l| {
                l.spans
                    .iter()
                    .map(|s| s.content.clone())
                    .collect::<Vec<_>>()
                    .join("")
            })
            .collect()
    }

    #[tokio::test]
    async fn controller_loose_vs_tight_with_commit_ticks_matches_full() {
        let cfg = test_config().await;
        let mut ctrl = StreamController::new(cfg.clone(), None);
        let mut lines = Vec::new();

        // Exact deltas from the session log (section: Loose vs. tight list items)
        let deltas = vec![
            "\n\n",
            "Loose",
            " vs",
            ".",
            " tight",
            " list",
            " items",
            ":\n",
            "1",
            ".",
            " Tight",
            " item",
            "\n",
            "2",
            ".",
            " Another",
            " tight",
            " item",
            "\n\n",
            "1",
            ".",
            " Loose",
            " item",
            " with",
            " its",
            " own",
            " paragraph",
            ".\n\n",
            "  ",
            " This",
            " paragraph",
            " belongs",
            " to",
            " the",
            " same",
            " list",
            " item",
            ".\n\n",
            "2",
            ".",
            " Second",
            " loose",
            " item",
            " with",
            " a",
            " nested",
            " list",
            " after",
            " a",
            " blank",
            " line",
            ".\n\n",
            "  ",
            " -",
            " Nested",
            " bullet",
            " under",
            " a",
            " loose",
            " item",
            "\n",
            "  ",
            " -",
            " Another",
            " nested",
            " bullet",
            "\n\n",
        ];

        // Simulate streaming with a commit tick attempt after each delta.
        for d in deltas.iter() {
            ctrl.push(d);
            while let (Some(cell), idle) = ctrl.on_commit_tick() {
                lines.extend(cell.transcript_lines());
                if idle {
                    break;
                }
            }
        }
        // Finalize and flush remaining lines now.
        if let Some(cell) = ctrl.finalize() {
            lines.extend(cell.transcript_lines());
        }

        let mut flat = lines;
        // Drop leading blank and header line if present.
        if !flat.is_empty() && lines_to_plain_strings(&[flat[0].clone()])[0].is_empty() {
            flat.remove(0);
        }
        if !flat.is_empty() {
            let s0 = lines_to_plain_strings(&[flat[0].clone()])[0].clone();
            if s0 == "codex" {
                flat.remove(0);
            }
        }
        let streamed = lines_to_plain_strings(&flat);

        // Full render of the same source
        let source: String = deltas.iter().copied().collect();
        let mut rendered: Vec<ratatui::text::Line<'static>> = Vec::new();
        crate::markdown::append_markdown(&source, None, &mut rendered, &cfg);
        let rendered_strs = lines_to_plain_strings(&rendered);

        assert_eq!(streamed, rendered_strs);

        // Also assert exact expected plain strings for clarity.
        let expected = vec![
            "Loose vs. tight list items:".to_string(),
            "".to_string(),
            "1. Tight item".to_string(),
            "2. Another tight item".to_string(),
            "3. Loose item with its own paragraph.".to_string(),
            "".to_string(),
            "   This paragraph belongs to the same list item.".to_string(),
            "4. Second loose item with a nested list after a blank line.".to_string(),
            "    - Nested bullet under a loose item".to_string(),
            "    - Another nested bullet".to_string(),
        ];
        assert_eq!(
            streamed, expected,
            "expected exact rendered lines for loose/tight section"
        );
    }
}
