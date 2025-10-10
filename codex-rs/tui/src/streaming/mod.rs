<<<<<<< HEAD
use crate::markdown_stream::AnimatedLineStreamer;
=======
use std::collections::VecDeque;

use ratatui::text::Line;

>>>>>>> upstream/main
use crate::markdown_stream::MarkdownStreamCollector;
pub(crate) mod controller;

pub(crate) struct StreamState {
    pub(crate) collector: MarkdownStreamCollector,
    queued_lines: VecDeque<Line<'static>>,
    pub(crate) has_seen_delta: bool,
}

impl StreamState {
    pub(crate) fn new(width: Option<usize>) -> Self {
        Self {
            collector: MarkdownStreamCollector::new(width),
            queued_lines: VecDeque::new(),
            has_seen_delta: false,
        }
    }
    pub(crate) fn clear(&mut self) {
        self.collector.clear();
        self.queued_lines.clear();
        self.has_seen_delta = false;
    }
    pub(crate) fn step(&mut self) -> Vec<Line<'static>> {
        self.queued_lines.pop_front().into_iter().collect()
    }
    pub(crate) fn drain_all(&mut self) -> Vec<Line<'static>> {
        self.queued_lines.drain(..).collect()
    }
    pub(crate) fn is_idle(&self) -> bool {
        self.queued_lines.is_empty()
    }
<<<<<<< HEAD
    pub(crate) fn enqueue(&mut self, lines: Vec<ratatui::text::Line<'static>>) {
        self.streamer.enqueue(lines)
    }
}

pub(crate) struct HeaderEmitter {
    emitted_this_turn: bool,
    emitted_in_stream: bool,
}

impl HeaderEmitter {
    pub(crate) fn new() -> Self {
        Self {
            emitted_this_turn: false,
            emitted_in_stream: false,
        }
    }

    pub(crate) fn reset_for_new_turn(&mut self) {
        self.emitted_this_turn = false;
        self.emitted_in_stream = false;
    }

    pub(crate) fn reset_for_stream(&mut self) {
        self.emitted_in_stream = false;
    }

    /// Allow emitting the header again within the current turn after a finalize.
    pub(crate) fn allow_reemit_in_turn(&mut self) {
        self.emitted_this_turn = false;
    }

    pub(crate) fn maybe_emit_header(&mut self) -> bool {
        if !self.emitted_in_stream && !self.emitted_this_turn {
            self.emitted_in_stream = true;
            self.emitted_this_turn = true;
            true
        } else {
            false
        }
=======
    pub(crate) fn enqueue(&mut self, lines: Vec<Line<'static>>) {
        self.queued_lines.extend(lines);
>>>>>>> upstream/main
    }
}
