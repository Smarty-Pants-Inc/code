use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

use ratatui::buffer::Cell as BufferCell;
use ratatui::text::Line;

use crate::history::state::{HistoryId, HistoryRecord, HistoryState};
use crate::history_cell::{
    assistant_markdown_lines,
    compute_assistant_layout,
    explore_lines_from_record_with_force,
    diff_lines_from_record,
    exec_display_lines_from_record,
    merged_exec_lines_from_record,
    stream_lines_from_state,
    AssistantLayoutCache,
    AssistantMarkdownCell,
    HistoryCell,
};
use code_core::config::Config;
#[cfg(feature = "code-fork")]
use crate::foundation::wrapping::word_wrap_lines;
#[cfg(not(feature = "code-fork"))]
use crate::insert_history::word_wrap_lines;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Memoized layout data for history rendering.
pub(crate) struct HistoryRenderState {
    pub(crate) layout_cache: RefCell<HashMap<CacheKey, Rc<CachedLayout>>>,
    pub(crate) height_cache: RefCell<HashMap<CacheKey, u16>>,
    pub(crate) height_cache_last_width: Cell<u16>,
    pub(crate) prefix_sums: RefCell<Vec<u16>>,
    pub(crate) last_prefix_width: Cell<u16>,
    pub(crate) last_prefix_count: Cell<usize>,
    pub(crate) last_total_height: Cell<u16>,
    pub(crate) prefix_valid: Cell<bool>,
    // Row intervals that correspond to inter-cell spacing so we can avoid
    // landing the viewport on empty gaps when scrolling.
    spacing_ranges: RefCell<Vec<(u16, u16)>>,
}

impl HistoryRenderState {
    pub(crate) fn new() -> Self {
        Self {
            layout_cache: RefCell::new(HashMap::new()),
            height_cache: RefCell::new(HashMap::new()),
            height_cache_last_width: Cell::new(0),
            prefix_sums: RefCell::new(Vec::new()),
            last_prefix_width: Cell::new(0),
            last_prefix_count: Cell::new(0),
            last_total_height: Cell::new(0),
            prefix_valid: Cell::new(false),
            spacing_ranges: RefCell::new(Vec::new()),
        }
    }

    pub(crate) fn invalidate_height_cache(&self) {
        self.layout_cache.borrow_mut().clear();
        self.height_cache.borrow_mut().clear();
        self.prefix_sums.borrow_mut().clear();
        self.last_total_height.set(0);
        self.prefix_valid.set(false);
        self.spacing_ranges.borrow_mut().clear();
    }

    pub(crate) fn handle_width_change(&self, width: u16) {
        if self.height_cache_last_width.get() != width {
            self.layout_cache
                .borrow_mut()
                .retain(|key, _| key.width == width);
            self.height_cache
                .borrow_mut()
                .retain(|key, _| key.width == width);
            self.prefix_sums.borrow_mut().clear();
            self.last_total_height.set(0);
            self.prefix_valid.set(false);
            self.height_cache_last_width.set(width);
            self.spacing_ranges.borrow_mut().clear();
        }
    }

    pub(crate) fn invalidate_history_id(&self, id: HistoryId) {
        if id == HistoryId::ZERO {
            return;
        }
        self.layout_cache
            .borrow_mut()
            .retain(|key, _| key.history_id != id);
        self.height_cache
            .borrow_mut()
            .retain(|key, _| key.history_id != id);
        self.prefix_sums.borrow_mut().clear();
        self.last_total_height.set(0);
        self.prefix_valid.set(false);
        self.spacing_ranges.borrow_mut().clear();
    }

    pub(crate) fn invalidate_all(&self) {
        self.layout_cache.borrow_mut().clear();
        self.height_cache.borrow_mut().clear();
        self.prefix_sums.borrow_mut().clear();
        self.last_total_height.set(0);
        self.prefix_valid.set(false);
        self.spacing_ranges.borrow_mut().clear();
    }

    pub(crate) fn should_rebuild_prefix(&self, width: u16, count: usize) -> bool {
        if !self.prefix_valid.get() {
            return true;
        }
        if self.last_prefix_width.get() != width {
            return true;
        }
        if self.last_prefix_count.get() != count {
            return true;
        }
        false
    }

    pub(crate) fn update_prefix_cache(
        &self,
        width: u16,
        prefix: Vec<u16>,
        total_height: u16,
        count: usize,
    ) {
        {
            let mut ps = self.prefix_sums.borrow_mut();
            *ps = prefix;
        }
        self.last_prefix_width.set(width);
        self.last_prefix_count.set(count);
        self.last_total_height.set(total_height);
        self.prefix_valid.set(true);
    }

    pub(crate) fn update_spacing_ranges(&self, ranges: Vec<(u16, u16)>) {
        *self.spacing_ranges.borrow_mut() = ranges;
    }

    pub(crate) fn adjust_scroll_to_content(&self, mut scroll_pos: u16) -> u16 {
        if scroll_pos == 0 {
            return scroll_pos;
        }
        let ranges = self.spacing_ranges.borrow();
        if ranges.is_empty() {
            return scroll_pos;
        }
        // Walk backwards until we hit a true cell row or run out of history.
        loop {
            let mut adjusted = false;
            for &(start, end) in ranges.iter() {
                if start == 0 {
                    continue;
                }
                if scroll_pos >= start && scroll_pos < end {
                    scroll_pos = start.saturating_sub(1);
                    adjusted = true;
                    break;
                }
            }
            if !adjusted || scroll_pos == 0 {
                break;
            }
        }
        scroll_pos
    }

    #[cfg(test)]
    pub(crate) fn spacing_ranges_for_test(&self) -> Vec<(u16, u16)> {
        self.spacing_ranges.borrow().clone()
    }

    pub(crate) fn last_total_height(&self) -> u16 {
        self.last_total_height.get()
    }

    pub(crate) fn visible_cells<'a>(
        &self,
        history_state: &HistoryState,
        requests: &[RenderRequest<'a>],
        settings: RenderSettings,
    ) -> Vec<VisibleCell<'a>> {
        requests
            .iter()
            .map(|req| {
                let assistant_plan = if settings.width == 0 {
                    None
                } else if let Some(assistant_cell) = req.assistant {
                    Some(assistant_cell.ensure_layout(settings.width))
                } else if let RenderRequestKind::Assistant { id } = req.kind {
                    history_state
                        .record(id)
                        .and_then(|record| match record {
                            HistoryRecord::AssistantMessage(state) => Some(Rc::new(
                                compute_assistant_layout(state, req.config, settings.width),
                            )),
                            _ => None,
                        })
                } else {
                    None
                };

                let has_custom_render = req
                    .cell
                    .map(|cell| cell.has_custom_render())
                    .unwrap_or(false);

                let prohibit_cache = matches!(req.kind, RenderRequestKind::Streaming { .. });
                let use_cache = req.use_cache && !prohibit_cache;

                let layout = if has_custom_render {
                    None
                } else if settings.width == 0 {
                    None
                } else if assistant_plan.is_some() {
                    None
                } else if use_cache && req.history_id != HistoryId::ZERO {
                    Some(self.render_cached(req.history_id, settings, || {
                        req.build_lines(history_state)
                    }))
                } else {
                    Some(self.render_adhoc(settings.width, || {
                        req.build_lines(history_state)
                    }))
                };

                let use_height_cache = use_cache && req.history_id != HistoryId::ZERO;
                let cached_height = if use_height_cache {
                    let key = CacheKey::new(req.history_id, settings);
                    self.height_cache
                        .borrow()
                        .get(&key)
                        .copied()
                        .map(|h| (h, HeightSource::Cached, None))
                } else {
                    None
                };

                let (height, height_source, height_measure_ns) = if settings.width == 0 {
                    (0, HeightSource::ZeroWidth, None)
                } else if let Some(plan) = assistant_plan.as_ref() {
                    (plan.total_rows(), HeightSource::AssistantPlan, None)
                } else if let Some(layout_ref) = layout.as_ref() {
                    (
                        layout_ref
                            .line_count()
                            .min(u16::MAX as usize) as u16,
                        HeightSource::Layout,
                        None,
                    )
                } else if let Some((h, src, measure)) = cached_height {
                    (h, src, measure)
                } else if let Some(cell) = req.cell {
                    if cell.has_custom_render() {
                        let start = Instant::now();
                        let computed = cell.desired_height(settings.width);
                        let elapsed = start.elapsed().as_nanos();
                        if use_height_cache {
                            let key = CacheKey::new(req.history_id, settings);
                            self.height_cache.borrow_mut().insert(key, computed);
                        }
                        (
                            computed,
                            HeightSource::DesiredHeight,
                            Some(elapsed),
                        )
                    } else if let Some(lines) = req.fallback_lines.as_ref() {
                        let wrapped = word_wrap_lines(lines, settings.width);
                        let height = wrapped.len().min(u16::MAX as usize) as u16;
                        if use_height_cache {
                            let key = CacheKey::new(req.history_id, settings);
                            self.height_cache.borrow_mut().insert(key, height);
                        }
                        (height, HeightSource::FallbackLines, None)
                    } else {
                        let start = Instant::now();
                        let computed = cell.desired_height(settings.width);
                        let elapsed = start.elapsed().as_nanos();
                        if use_height_cache {
                            let key = CacheKey::new(req.history_id, settings);
                            self.height_cache.borrow_mut().insert(key, computed);
                        }
                        (
                            computed,
                            HeightSource::DesiredHeight,
                            Some(elapsed),
                        )
                    }
                } else if let Some(lines) = req.fallback_lines.as_ref() {
                    let wrapped = word_wrap_lines(lines, settings.width);
                    let height = wrapped.len().min(u16::MAX as usize) as u16;
                    if use_height_cache {
                        let key = CacheKey::new(req.history_id, settings);
                        self.height_cache.borrow_mut().insert(key, height);
                    }
                    (height, HeightSource::FallbackLines, None)
                } else {
                    (0, HeightSource::Unknown, None)
                };

                VisibleCell {
                    cell: req.cell,
                    assistant_plan,
                    layout,
                    height,
                    height_source,
                    height_measure_ns,
                }
            })
            .collect()
    }

    fn render_cached<F>(&self, history_id: HistoryId, settings: RenderSettings, build_lines: F) -> LayoutRef
    where
        F: FnOnce() -> Vec<Line<'static>>,
    {
        if settings.width == 0 {
            return LayoutRef::empty();
        }

        let key = CacheKey::new(history_id, settings);
        if let Some(layout) = self.layout_cache.borrow().get(&key).cloned() {
            return LayoutRef { data: layout };
        }

        let layout = Rc::new(build_cached_layout(build_lines(), settings.width));
        self.layout_cache
            .borrow_mut()
            .insert(key, Rc::clone(&layout));
        LayoutRef { data: layout }
    }

    fn render_adhoc<F>(&self, width: u16, build_lines: F) -> LayoutRef
    where
        F: FnOnce() -> Vec<Line<'static>>,
    {
        if width == 0 {
            return LayoutRef::empty();
        }
        LayoutRef {
            data: Rc::new(build_cached_layout(build_lines(), width)),
        }
    }
}

#[derive(Clone)]
pub(crate) struct LayoutRef {
    pub(crate) data: Rc<CachedLayout>,
}

impl LayoutRef {
    fn empty() -> Self {
        LayoutRef {
            data: Rc::new(CachedLayout {
                lines: Vec::new(),
                rows: Vec::new(),
            }),
        }
    }

    pub(crate) fn layout(&self) -> Rc<CachedLayout> {
        Rc::clone(&self.data)
    }

    pub(crate) fn line_count(&self) -> usize {
        self.data.lines.len()
    }
}

impl Default for HistoryRenderState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub(crate) struct CachedLayout {
    pub(crate) lines: Vec<Line<'static>>,
    pub(crate) rows: Vec<Box<[BufferCell]>>,
}

fn build_cached_layout(lines: Vec<Line<'static>>, width: u16) -> CachedLayout {
    let wrapped = if lines.is_empty() {
        Vec::new()
    } else {
        word_wrap_lines(&lines, width)
    };
    let rows = build_cached_rows(&wrapped, width);
    CachedLayout { lines: wrapped, rows }
}

fn build_cached_rows(lines: &[Line<'static>], width: u16) -> Vec<Box<[BufferCell]>> {
    let target_width = width as usize;
    lines
        .iter()
        .map(|line| build_cached_row(line, target_width))
        .collect()
}

fn build_cached_row(line: &Line<'static>, target_width: usize) -> Box<[BufferCell]> {
    if target_width == 0 {
        return Box::new([]);
    }

    let mut cells = vec![BufferCell::default(); target_width];
    let mut x: u16 = 0;
    let mut remaining = target_width as u16;

    for span in &line.spans {
        if remaining == 0 {
            break;
        }
        let span_style = line.style.patch(span.style);
        for symbol in UnicodeSegmentation::graphemes(span.content.as_ref(), true) {
            if symbol.chars().any(|ch| ch.is_control()) {
                continue;
            }
            let symbol_width = UnicodeWidthStr::width(symbol) as u16;
            if symbol_width == 0 {
                continue;
            }
            if symbol_width > remaining {
                remaining = 0;
                break;
            }

            let idx = x as usize;
            if idx >= target_width {
                remaining = 0;
                break;
            }

            cells[idx].set_symbol(symbol).set_style(span_style);

            let next_symbol = x.saturating_add(symbol_width);
            x = x.saturating_add(1);
            while x < next_symbol {
                let fill_idx = x as usize;
                if fill_idx >= target_width {
                    remaining = 0;
                    break;
                }
                cells[fill_idx].reset();
                x = x.saturating_add(1);
            }
            if remaining == 0 {
                break;
            }
            if x >= target_width as u16 {
                remaining = 0;
                break;
            }
            remaining = target_width as u16 - x;
            if remaining == 0 {
                break;
            }
        }
        if remaining == 0 {
            break;
        }
    }

    cells.into_boxed_slice()
}

/// Settings that affect layout caching. Any change to these fields invalidates
/// the cached `CachedLayout` entries keyed by `(HistoryId, width, theme_epoch,
/// reasoning_visible)`.
#[derive(Clone, Copy)]
pub(crate) struct RenderSettings {
    pub width: u16,
    pub theme_epoch: u64,
    pub reasoning_visible: bool,
}

impl RenderSettings {
    pub fn new(width: u16, theme_epoch: u64, reasoning_visible: bool) -> Self {
        Self {
            width,
            theme_epoch,
            reasoning_visible,
        }
    }
}

/// A rendering input assembled by `ChatWidget::draw_history` for a single
/// history record. We keep both the legacy `HistoryCell` (if one exists) and a
/// semantic fallback so the renderer can rebuild layouts directly from
/// `HistoryRecord` data when needed.
pub(crate) struct RenderRequest<'a> {
    pub history_id: HistoryId,
    pub cell: Option<&'a dyn HistoryCell>,
    pub assistant: Option<&'a AssistantMarkdownCell>,
    pub use_cache: bool,
    pub fallback_lines: Option<Vec<Line<'static>>>,
    pub kind: RenderRequestKind,
    pub config: &'a Config,
}

impl<'a> RenderRequest<'a> {
    /// Returns the best-effort lines for this record. We prefer the existing
    /// `HistoryCell` cache (which may include per-cell layout bridges) and fall
    /// back to semantic lines derived from the record state.
    fn build_lines(&self, history_state: &HistoryState) -> Vec<Line<'static>> {
        if let RenderRequestKind::Exec { id } = self.kind {
            if let Some(HistoryRecord::Exec(record)) = history_state.record(id) {
                return exec_display_lines_from_record(record);
            }
        }

        if let RenderRequestKind::MergedExec { id } = self.kind {
            if let Some(HistoryRecord::MergedExec(record)) = history_state.record(id) {
                return merged_exec_lines_from_record(record);
            }
        }

        if let RenderRequestKind::Explore { id, hold_header } = self.kind {
            if let Some(HistoryRecord::Explore(record)) = history_state.record(id) {
                return explore_lines_from_record_with_force(record, hold_header);
            }
        }

        if let RenderRequestKind::Diff { id } = self.kind {
            if let Some(HistoryRecord::Diff(record)) = history_state.record(id) {
                return diff_lines_from_record(record);
            }
        }

        if let RenderRequestKind::Streaming { id } = self.kind {
            if let Some(HistoryRecord::AssistantStream(record)) = history_state.record(id) {
                return stream_lines_from_state(record, self.config, record.in_progress);
            }
        }

        if let RenderRequestKind::Assistant { id } = self.kind {
            if let Some(HistoryRecord::AssistantMessage(record)) = history_state.record(id) {
                return assistant_markdown_lines(record, self.config);
            }
        }

        if let Some(cell) = self.cell {
            return cell.display_lines_trimmed();
        }

        if let Some(lines) = &self.fallback_lines {
            return lines.clone();
        }
        Vec::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Identifies the source for `RenderRequest` line construction.
/// Exec variants always rebuild lines from `HistoryState`, ensuring the
/// shared renderer cache is the single source of truth for layout data.
pub(crate) enum RenderRequestKind {
    Legacy,
    Exec { id: HistoryId },
    MergedExec { id: HistoryId },
    Explore { id: HistoryId, hold_header: bool },
    Diff { id: HistoryId },
    Streaming { id: HistoryId },
    Assistant { id: HistoryId },
}

/// Output from `HistoryRenderState::visible_cells()`. Contains the resolved
/// layout (if any), plus the optional `HistoryCell` pointer so the caller can
/// reuse existing caches.
pub(crate) struct VisibleCell<'a> {
    pub cell: Option<&'a dyn HistoryCell>,
    pub assistant_plan: Option<Rc<AssistantLayoutCache>>,
    pub layout: Option<LayoutRef>,
    pub height: u16,
    pub height_source: HeightSource,
    pub height_measure_ns: Option<u128>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HeightSource {
    AssistantPlan,
    Layout,
    Cached,
    DesiredHeight,
    FallbackLines,
    ZeroWidth,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct CacheKey {
    history_id: HistoryId,
    width: u16,
    theme_epoch: u64,
    reasoning_visible: bool,
}

impl CacheKey {
    fn new(history_id: HistoryId, settings: RenderSettings) -> Self {
        Self {
            history_id,
            width: settings.width,
            theme_epoch: settings.theme_epoch,
            reasoning_visible: settings.reasoning_visible,
        }
    }
}
