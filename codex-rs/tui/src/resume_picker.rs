<<<<<<< HEAD
use std::path::Path;
use std::path::PathBuf;
=======
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
>>>>>>> upstream/main

use chrono::DateTime;
use chrono::Utc;
use codex_core::ConversationItem;
use codex_core::ConversationsPage;
use codex_core::Cursor;
<<<<<<< HEAD
=======
use codex_core::INTERACTIVE_SESSION_SOURCES;
>>>>>>> upstream/main
use codex_core::RolloutRecorder;
use color_eyre::eyre::Result;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Stylize as _;
use ratatui::text::Line;
<<<<<<< HEAD
use tokio_stream::StreamExt;

=======
use ratatui::text::Span;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;
use unicode_width::UnicodeWidthStr;

use crate::key_hint;
>>>>>>> upstream/main
use crate::text_formatting::truncate_text;
use crate::tui::FrameRequester;
use crate::tui::Tui;
use crate::tui::TuiEvent;
use codex_protocol::models::ContentItem;
use codex_protocol::models::ResponseItem;
use codex_protocol::protocol::InputMessageKind;
use codex_protocol::protocol::USER_MESSAGE_BEGIN;

const PAGE_SIZE: usize = 25;
<<<<<<< HEAD
=======
const LOAD_NEAR_THRESHOLD: usize = 5;
>>>>>>> upstream/main

#[derive(Debug, Clone)]
pub enum ResumeSelection {
    StartFresh,
    Resume(PathBuf),
    Exit,
}

<<<<<<< HEAD
=======
#[derive(Clone)]
struct PageLoadRequest {
    codex_home: PathBuf,
    cursor: Option<Cursor>,
    request_token: usize,
    search_token: Option<usize>,
}

type PageLoader = Arc<dyn Fn(PageLoadRequest) + Send + Sync>;

enum BackgroundEvent {
    PageLoaded {
        request_token: usize,
        search_token: Option<usize>,
        page: std::io::Result<ConversationsPage>,
    },
}

>>>>>>> upstream/main
/// Interactive session picker that lists recorded rollout files with simple
/// search and pagination. Shows the first user input as the preview, relative
/// time (e.g., "5 seconds ago"), and the absolute path.
pub async fn run_resume_picker(tui: &mut Tui, codex_home: &Path) -> Result<ResumeSelection> {
    let alt = AltScreenGuard::enter(tui);
<<<<<<< HEAD
    let mut state = PickerState::new(codex_home.to_path_buf(), alt.tui.frame_requester());
    state.load_page(None).await?;
    state.request_frame();

    let mut events = alt.tui.event_stream();
    while let Some(ev) = events.next().await {
        match ev {
            TuiEvent::Key(key) => {
                if matches!(key.kind, KeyEventKind::Release) {
                    continue;
                }
                if let Some(sel) = state.handle_key(key).await? {
                    return Ok(sel);
                }
            }
            TuiEvent::Draw => {
                draw_picker(alt.tui, &state)?;
            }
            // Ignore paste and attach-image in picker
            _ => {}
=======
    let (bg_tx, bg_rx) = mpsc::unbounded_channel();

    let loader_tx = bg_tx.clone();
    let page_loader: PageLoader = Arc::new(move |request: PageLoadRequest| {
        let tx = loader_tx.clone();
        tokio::spawn(async move {
            let page = RolloutRecorder::list_conversations(
                &request.codex_home,
                PAGE_SIZE,
                request.cursor.as_ref(),
                INTERACTIVE_SESSION_SOURCES,
            )
            .await;
            let _ = tx.send(BackgroundEvent::PageLoaded {
                request_token: request.request_token,
                search_token: request.search_token,
                page,
            });
        });
    });

    let mut state = PickerState::new(
        codex_home.to_path_buf(),
        alt.tui.frame_requester(),
        page_loader,
    );
    state.load_initial_page().await?;
    state.request_frame();

    let mut tui_events = alt.tui.event_stream().fuse();
    let mut background_events = UnboundedReceiverStream::new(bg_rx).fuse();

    loop {
        tokio::select! {
            Some(ev) = tui_events.next() => {
                match ev {
                    TuiEvent::Key(key) => {
                        if matches!(key.kind, KeyEventKind::Release) {
                            continue;
                        }
                        if let Some(sel) = state.handle_key(key).await? {
                            return Ok(sel);
                        }
                    }
                    TuiEvent::Draw => {
                        if let Ok(size) = alt.tui.terminal.size() {
                            let list_height = size.height.saturating_sub(4) as usize;
                            state.update_view_rows(list_height);
                            state.ensure_minimum_rows_for_view(list_height);
                        }
                        draw_picker(alt.tui, &state)?;
                    }
                    _ => {}
                }
            }
            Some(event) = background_events.next() => {
                state.handle_background_event(event)?;
            }
            else => break,
>>>>>>> upstream/main
        }
    }

    // Fallback – treat as cancel/new
    Ok(ResumeSelection::StartFresh)
}

/// RAII guard that ensures we leave the alt-screen on scope exit.
struct AltScreenGuard<'a> {
    tui: &'a mut Tui,
}

impl<'a> AltScreenGuard<'a> {
    fn enter(tui: &'a mut Tui) -> Self {
        let _ = tui.enter_alt_screen();
        Self { tui }
    }
}

impl Drop for AltScreenGuard<'_> {
    fn drop(&mut self) {
        let _ = self.tui.leave_alt_screen();
    }
}

struct PickerState {
    codex_home: PathBuf,
    requester: FrameRequester,
<<<<<<< HEAD
    // pagination
    pagination: Pagination,
    // data
    all_rows: Vec<Row>, // unfiltered rows for current page
    filtered_rows: Vec<Row>,
    selected: usize,
    // search
    query: String,
}

#[derive(Debug, Clone)]
struct Pagination {
    current_anchor: Option<Cursor>,
    backstack: Vec<Option<Cursor>>, // track previous anchors for ←/a
    next_cursor: Option<Cursor>,
    page_index: usize,
=======
    pagination: PaginationState,
    all_rows: Vec<Row>,
    filtered_rows: Vec<Row>,
    seen_paths: HashSet<PathBuf>,
    selected: usize,
    scroll_top: usize,
    query: String,
    search_state: SearchState,
    next_request_token: usize,
    next_search_token: usize,
    page_loader: PageLoader,
    view_rows: Option<usize>,
}

struct PaginationState {
    next_cursor: Option<Cursor>,
    num_scanned_files: usize,
    reached_scan_cap: bool,
    loading: LoadingState,
}

#[derive(Clone, Copy, Debug)]
enum LoadingState {
    Idle,
    Pending(PendingLoad),
}

#[derive(Clone, Copy, Debug)]
struct PendingLoad {
    request_token: usize,
    search_token: Option<usize>,
}

#[derive(Clone, Copy, Debug)]
enum SearchState {
    Idle,
    Active { token: usize },
}

enum LoadTrigger {
    Scroll,
    Search { token: usize },
}

impl LoadingState {
    fn is_pending(&self) -> bool {
        matches!(self, LoadingState::Pending(_))
    }
}

impl SearchState {
    fn active_token(&self) -> Option<usize> {
        match self {
            SearchState::Idle => None,
            SearchState::Active { token } => Some(*token),
        }
    }

    fn is_active(&self) -> bool {
        self.active_token().is_some()
    }
>>>>>>> upstream/main
}

#[derive(Clone)]
struct Row {
    path: PathBuf,
    preview: String,
<<<<<<< HEAD
    ts: Option<DateTime<Utc>>,
}

impl PickerState {
    fn new(codex_home: PathBuf, requester: FrameRequester) -> Self {
        Self {
            codex_home,
            requester,
            pagination: Pagination {
                current_anchor: None,
                backstack: vec![None],
                next_cursor: None,
                page_index: 0,
            },
            all_rows: Vec::new(),
            filtered_rows: Vec::new(),
            selected: 0,
            query: String::new(),
=======
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
}

impl PickerState {
    fn new(codex_home: PathBuf, requester: FrameRequester, page_loader: PageLoader) -> Self {
        Self {
            codex_home,
            requester,
            pagination: PaginationState {
                next_cursor: None,
                num_scanned_files: 0,
                reached_scan_cap: false,
                loading: LoadingState::Idle,
            },
            all_rows: Vec::new(),
            filtered_rows: Vec::new(),
            seen_paths: HashSet::new(),
            selected: 0,
            scroll_top: 0,
            query: String::new(),
            search_state: SearchState::Idle,
            next_request_token: 0,
            next_search_token: 0,
            page_loader,
            view_rows: None,
>>>>>>> upstream/main
        }
    }

    fn request_frame(&self) {
        self.requester.schedule_frame();
    }

    async fn handle_key(&mut self, key: KeyEvent) -> Result<Option<ResumeSelection>> {
        match key.code {
            KeyCode::Esc => return Ok(Some(ResumeSelection::StartFresh)),
            KeyCode::Char('c')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                return Ok(Some(ResumeSelection::Exit));
            }
            KeyCode::Enter => {
                if let Some(row) = self.filtered_rows.get(self.selected) {
                    return Ok(Some(ResumeSelection::Resume(row.path.clone())));
                }
            }
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
<<<<<<< HEAD
=======
                    self.ensure_selected_visible();
>>>>>>> upstream/main
                }
                self.request_frame();
            }
            KeyCode::Down => {
                if self.selected + 1 < self.filtered_rows.len() {
                    self.selected += 1;
<<<<<<< HEAD
                }
                self.request_frame();
            }
            KeyCode::Left | KeyCode::Char('a') => {
                self.prev_page().await?;
            }
            KeyCode::Right | KeyCode::Char('d') => {
                self.next_page().await?;
            }
            KeyCode::Backspace => {
                self.query.pop();
                self.apply_filter();
=======
                    self.ensure_selected_visible();
                }
                self.maybe_load_more_for_scroll();
                self.request_frame();
            }
            KeyCode::PageUp => {
                let step = self.view_rows.unwrap_or(10).max(1);
                if self.selected > 0 {
                    self.selected = self.selected.saturating_sub(step);
                    self.ensure_selected_visible();
                    self.request_frame();
                }
            }
            KeyCode::PageDown => {
                if !self.filtered_rows.is_empty() {
                    let step = self.view_rows.unwrap_or(10).max(1);
                    let max_index = self.filtered_rows.len().saturating_sub(1);
                    self.selected = (self.selected + step).min(max_index);
                    self.ensure_selected_visible();
                    self.maybe_load_more_for_scroll();
                    self.request_frame();
                }
            }
            KeyCode::Backspace => {
                let mut new_query = self.query.clone();
                new_query.pop();
                self.set_query(new_query);
>>>>>>> upstream/main
            }
            KeyCode::Char(c) => {
                // basic text input for search
                if !key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                    && !key.modifiers.contains(crossterm::event::KeyModifiers::ALT)
                {
<<<<<<< HEAD
                    self.query.push(c);
                    self.apply_filter();
=======
                    let mut new_query = self.query.clone();
                    new_query.push(c);
                    self.set_query(new_query);
>>>>>>> upstream/main
                }
            }
            _ => {}
        }
        Ok(None)
    }

<<<<<<< HEAD
    async fn prev_page(&mut self) -> Result<()> {
        if self.pagination.page_index == 0 {
            return Ok(());
        }
        // current_anchor points to the page we just loaded; backstack[page_index-1] is the anchor to reload
        if self.pagination.page_index > 0 {
            self.pagination.page_index -= 1;
            let anchor = self
                .pagination
                .backstack
                .get(self.pagination.page_index)
                .cloned()
                .flatten();
            self.pagination.current_anchor = anchor.clone();
            self.load_page(anchor.as_ref()).await?;
        }
        Ok(())
    }

    async fn next_page(&mut self) -> Result<()> {
        if let Some(next) = self.pagination.next_cursor.clone() {
            // Record the anchor for the page we are moving to at index new_index
            let new_index = self.pagination.page_index + 1;
            if self.pagination.backstack.len() <= new_index {
                self.pagination.backstack.resize(new_index + 1, None);
            }
            self.pagination.backstack[new_index] = Some(next.clone());
            self.pagination.current_anchor = Some(next.clone());
            self.pagination.page_index = new_index;
            let anchor = self.pagination.current_anchor.clone();
            self.load_page(anchor.as_ref()).await?;
        }
        Ok(())
    }

    async fn load_page(&mut self, anchor: Option<&Cursor>) -> Result<()> {
        let page = RolloutRecorder::list_conversations(&self.codex_home, PAGE_SIZE, anchor).await?;
        self.pagination.next_cursor = page.next_cursor.clone();
        self.all_rows = to_rows(page);
        self.apply_filter();
        // reset selection on new page
        self.selected = 0;
        Ok(())
    }

=======
    async fn load_initial_page(&mut self) -> Result<()> {
        let page = RolloutRecorder::list_conversations(
            &self.codex_home,
            PAGE_SIZE,
            None,
            INTERACTIVE_SESSION_SOURCES,
        )
        .await?;
        self.reset_pagination();
        self.all_rows.clear();
        self.filtered_rows.clear();
        self.seen_paths.clear();
        self.search_state = SearchState::Idle;
        self.selected = 0;
        self.ingest_page(page);
        Ok(())
    }

    fn handle_background_event(&mut self, event: BackgroundEvent) -> Result<()> {
        match event {
            BackgroundEvent::PageLoaded {
                request_token,
                search_token,
                page,
            } => {
                let pending = match self.pagination.loading {
                    LoadingState::Pending(pending) => pending,
                    LoadingState::Idle => return Ok(()),
                };
                if pending.request_token != request_token {
                    return Ok(());
                }
                self.pagination.loading = LoadingState::Idle;
                let page = page.map_err(color_eyre::Report::from)?;
                self.ingest_page(page);
                let completed_token = pending.search_token.or(search_token);
                self.continue_search_if_token_matches(completed_token);
            }
        }
        Ok(())
    }

    fn reset_pagination(&mut self) {
        self.pagination.next_cursor = None;
        self.pagination.num_scanned_files = 0;
        self.pagination.reached_scan_cap = false;
        self.pagination.loading = LoadingState::Idle;
    }

    fn ingest_page(&mut self, page: ConversationsPage) {
        if let Some(cursor) = page.next_cursor.clone() {
            self.pagination.next_cursor = Some(cursor);
        } else {
            self.pagination.next_cursor = None;
        }
        self.pagination.num_scanned_files = self
            .pagination
            .num_scanned_files
            .saturating_add(page.num_scanned_files);
        if page.reached_scan_cap {
            self.pagination.reached_scan_cap = true;
        }

        let rows = rows_from_items(page.items);
        for row in rows {
            if self.seen_paths.insert(row.path.clone()) {
                self.all_rows.push(row);
            }
        }

        self.apply_filter();
    }

>>>>>>> upstream/main
    fn apply_filter(&mut self) {
        if self.query.is_empty() {
            self.filtered_rows = self.all_rows.clone();
        } else {
            let q = self.query.to_lowercase();
            self.filtered_rows = self
                .all_rows
                .iter()
                .filter(|r| r.preview.to_lowercase().contains(&q))
                .cloned()
                .collect();
        }
        if self.selected >= self.filtered_rows.len() {
            self.selected = self.filtered_rows.len().saturating_sub(1);
        }
<<<<<<< HEAD
        self.request_frame();
    }
}

fn to_rows(page: ConversationsPage) -> Vec<Row> {
    page.items.into_iter().map(|it| head_to_row(&it)).collect()
}

fn head_to_row(item: &ConversationItem) -> Row {
    let mut ts: Option<DateTime<Utc>> = None;
    if let Some(first) = item.head.first()
        && let Some(t) = first.get("timestamp").and_then(|v| v.as_str())
        && let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(t)
    {
        ts = Some(parsed.with_timezone(&Utc));
    }
=======
        if self.filtered_rows.is_empty() {
            self.scroll_top = 0;
        }
        self.ensure_selected_visible();
        self.request_frame();
    }

    fn set_query(&mut self, new_query: String) {
        if self.query == new_query {
            return;
        }
        self.query = new_query;
        self.selected = 0;
        self.apply_filter();
        if self.query.is_empty() {
            self.search_state = SearchState::Idle;
            return;
        }
        if !self.filtered_rows.is_empty() {
            self.search_state = SearchState::Idle;
            return;
        }
        if self.pagination.reached_scan_cap || self.pagination.next_cursor.is_none() {
            self.search_state = SearchState::Idle;
            return;
        }
        let token = self.allocate_search_token();
        self.search_state = SearchState::Active { token };
        self.load_more_if_needed(LoadTrigger::Search { token });
    }

    fn continue_search_if_needed(&mut self) {
        let Some(token) = self.search_state.active_token() else {
            return;
        };
        if !self.filtered_rows.is_empty() {
            self.search_state = SearchState::Idle;
            return;
        }
        if self.pagination.reached_scan_cap || self.pagination.next_cursor.is_none() {
            self.search_state = SearchState::Idle;
            return;
        }
        self.load_more_if_needed(LoadTrigger::Search { token });
    }

    fn continue_search_if_token_matches(&mut self, completed_token: Option<usize>) {
        let Some(active) = self.search_state.active_token() else {
            return;
        };
        if let Some(token) = completed_token
            && token != active
        {
            return;
        }
        self.continue_search_if_needed();
    }

    fn ensure_selected_visible(&mut self) {
        if self.filtered_rows.is_empty() {
            self.scroll_top = 0;
            return;
        }
        let capacity = self.view_rows.unwrap_or(self.filtered_rows.len()).max(1);

        if self.selected < self.scroll_top {
            self.scroll_top = self.selected;
        } else {
            let last_visible = self.scroll_top.saturating_add(capacity - 1);
            if self.selected > last_visible {
                self.scroll_top = self.selected.saturating_sub(capacity - 1);
            }
        }

        let max_start = self.filtered_rows.len().saturating_sub(capacity);
        if self.scroll_top > max_start {
            self.scroll_top = max_start;
        }
    }

    fn ensure_minimum_rows_for_view(&mut self, minimum_rows: usize) {
        if minimum_rows == 0 {
            return;
        }
        if self.filtered_rows.len() >= minimum_rows {
            return;
        }
        if self.pagination.loading.is_pending() || self.pagination.next_cursor.is_none() {
            return;
        }
        if let Some(token) = self.search_state.active_token() {
            self.load_more_if_needed(LoadTrigger::Search { token });
        } else {
            self.load_more_if_needed(LoadTrigger::Scroll);
        }
    }

    fn update_view_rows(&mut self, rows: usize) {
        self.view_rows = if rows == 0 { None } else { Some(rows) };
        self.ensure_selected_visible();
    }

    fn maybe_load_more_for_scroll(&mut self) {
        if self.pagination.loading.is_pending() {
            return;
        }
        if self.pagination.next_cursor.is_none() {
            return;
        }
        if self.filtered_rows.is_empty() {
            return;
        }
        let remaining = self.filtered_rows.len().saturating_sub(self.selected + 1);
        if remaining <= LOAD_NEAR_THRESHOLD {
            self.load_more_if_needed(LoadTrigger::Scroll);
        }
    }

    fn load_more_if_needed(&mut self, trigger: LoadTrigger) {
        if self.pagination.loading.is_pending() {
            return;
        }
        let Some(cursor) = self.pagination.next_cursor.clone() else {
            return;
        };
        let request_token = self.allocate_request_token();
        let search_token = match trigger {
            LoadTrigger::Scroll => None,
            LoadTrigger::Search { token } => Some(token),
        };
        self.pagination.loading = LoadingState::Pending(PendingLoad {
            request_token,
            search_token,
        });
        self.request_frame();

        (self.page_loader)(PageLoadRequest {
            codex_home: self.codex_home.clone(),
            cursor: Some(cursor),
            request_token,
            search_token,
        });
    }

    fn allocate_request_token(&mut self) -> usize {
        let token = self.next_request_token;
        self.next_request_token = self.next_request_token.wrapping_add(1);
        token
    }

    fn allocate_search_token(&mut self) -> usize {
        let token = self.next_search_token;
        self.next_search_token = self.next_search_token.wrapping_add(1);
        token
    }
}

fn rows_from_items(items: Vec<ConversationItem>) -> Vec<Row> {
    items.into_iter().map(|item| head_to_row(&item)).collect()
}

fn head_to_row(item: &ConversationItem) -> Row {
    let created_at = item
        .created_at
        .as_deref()
        .and_then(parse_timestamp_str)
        .or_else(|| item.head.first().and_then(extract_timestamp));
    let updated_at = item
        .updated_at
        .as_deref()
        .and_then(parse_timestamp_str)
        .or(created_at);
>>>>>>> upstream/main

    let preview = preview_from_head(&item.head)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| String::from("(no message yet)"));

    Row {
        path: item.path.clone(),
        preview,
<<<<<<< HEAD
        ts,
    }
}

=======
        created_at,
        updated_at,
    }
}

fn parse_timestamp_str(ts: &str) -> Option<DateTime<Utc>> {
    chrono::DateTime::parse_from_rfc3339(ts)
        .map(|dt| dt.with_timezone(&Utc))
        .ok()
}

fn extract_timestamp(value: &serde_json::Value) -> Option<DateTime<Utc>> {
    value
        .get("timestamp")
        .and_then(|v| v.as_str())
        .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
        .map(|dt| dt.with_timezone(&Utc))
}

>>>>>>> upstream/main
fn preview_from_head(head: &[serde_json::Value]) -> Option<String> {
    head.iter()
        .filter_map(|value| serde_json::from_value::<ResponseItem>(value.clone()).ok())
        .find_map(|item| match item {
            ResponseItem::Message { content, .. } => {
                // Find the actual user message (as opposed to user instructions or ide context)
                let preview = content
                    .into_iter()
                    .filter_map(|content| match content {
                        ContentItem::InputText { text }
                            if matches!(
                                InputMessageKind::from(("user", text.as_str())),
                                InputMessageKind::Plain
                            ) =>
                        {
                            // Strip ide context.
                            let text = match text.find(USER_MESSAGE_BEGIN) {
                                Some(idx) => {
                                    text[idx + USER_MESSAGE_BEGIN.len()..].trim().to_string()
                                }
                                None => text,
                            };
                            Some(text)
                        }
                        _ => None,
                    })
                    .collect::<String>();

                if preview.is_empty() {
                    None
                } else {
                    Some(preview)
                }
            }
            _ => None,
        })
}

fn draw_picker(tui: &mut Tui, state: &PickerState) -> std::io::Result<()> {
    // Render full-screen overlay
    let height = tui.terminal.size()?.height;
    tui.draw(height, |frame| {
        let area = frame.area();
<<<<<<< HEAD
        let [header, search, list, hint] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(area.height.saturating_sub(3)),
=======
        let [header, search, columns, list, hint] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(area.height.saturating_sub(4)),
>>>>>>> upstream/main
            Constraint::Length(1),
        ])
        .areas(area);

        // Header
        frame.render_widget_ref(
            Line::from(vec!["Resume a previous session".bold().cyan()]),
            header,
        );

        // Search line
        let q = if state.query.is_empty() {
            "Type to search".dim().to_string()
        } else {
            format!("Search: {}", state.query)
        };
        frame.render_widget_ref(Line::from(q), search);

<<<<<<< HEAD
        // List
        render_list(frame, list, state);

        // Hint line
        let hint_line: Line = vec![
            "Enter".bold(),
            " to resume  ".into(),
            "Esc".bold(),
            " to start new  ".into(),
            "Ctrl+C".into(),
            " to quit  ".dim(),
            "←/a".into(),
            " prev  ".dim(),
            "→/d".into(),
            " next".dim(),
=======
        let metrics = calculate_column_metrics(&state.filtered_rows);

        // Column headers and list
        render_column_headers(frame, columns, &metrics);
        render_list(frame, list, state, &metrics);

        // Hint line
        let hint_line: Line = vec![
            key_hint::plain(KeyCode::Enter).into(),
            " to resume ".dim(),
            "    ".dim(),
            key_hint::plain(KeyCode::Esc).into(),
            " to start new ".dim(),
            "    ".dim(),
            key_hint::ctrl(KeyCode::Char('c')).into(),
            " to quit ".dim(),
            "    ".dim(),
            key_hint::plain(KeyCode::Up).into(),
            "/".dim(),
            key_hint::plain(KeyCode::Down).into(),
            " to browse".dim(),
>>>>>>> upstream/main
        ]
        .into();
        frame.render_widget_ref(hint_line, hint);
    })
}

<<<<<<< HEAD
fn render_list(frame: &mut crate::custom_terminal::Frame, area: Rect, state: &PickerState) {
    let rows = &state.filtered_rows;
    if rows.is_empty() {
        frame.render_widget_ref(Line::from("No sessions found".italic().dim()), area);
        return;
    }

    // Compute how many rows fit (1 line per item)
    let capacity = area.height as usize;
    let start = state.selected.saturating_sub(capacity.saturating_sub(1));
    let visible = &rows[start..rows.len().min(start + capacity)];

    let mut y = area.y;
    for (idx, row) in visible.iter().enumerate() {
        let is_sel = start + idx == state.selected;
        let marker = if is_sel { "> ".bold() } else { "  ".into() };
        let ts = row
            .ts
            .map(human_time_ago)
            .unwrap_or_else(|| "".to_string())
            .dim();
        let max_cols = area.width.saturating_sub(6) as usize;
        let preview = truncate_text(&row.preview, max_cols);

        let line: Line = vec![marker, ts, "  ".into(), preview.into()].into();
=======
fn render_list(
    frame: &mut crate::custom_terminal::Frame,
    area: Rect,
    state: &PickerState,
    metrics: &ColumnMetrics,
) {
    if area.height == 0 {
        return;
    }

    let rows = &state.filtered_rows;
    if rows.is_empty() {
        let message = render_empty_state_line(state);
        frame.render_widget_ref(message, area);
        return;
    }

    let capacity = area.height as usize;
    let start = state.scroll_top.min(rows.len().saturating_sub(1));
    let end = rows.len().min(start + capacity);
    let labels = &metrics.labels;
    let mut y = area.y;

    let max_created_width = metrics.max_created_width;
    let max_updated_width = metrics.max_updated_width;

    for (idx, (row, (created_label, updated_label))) in rows[start..end]
        .iter()
        .zip(labels[start..end].iter())
        .enumerate()
    {
        let is_sel = start + idx == state.selected;
        let marker = if is_sel { "> ".bold() } else { "  ".into() };
        let marker_width = 2usize;
        let created_span = if max_created_width == 0 {
            None
        } else {
            Some(Span::from(format!("{created_label:<max_created_width$}")).dim())
        };
        let updated_span = if max_updated_width == 0 {
            None
        } else {
            Some(Span::from(format!("{updated_label:<max_updated_width$}")).dim())
        };
        let mut preview_width = area.width as usize;
        preview_width = preview_width.saturating_sub(marker_width);
        if max_created_width > 0 {
            preview_width = preview_width.saturating_sub(max_created_width + 2);
        }
        if max_updated_width > 0 {
            preview_width = preview_width.saturating_sub(max_updated_width + 2);
        }
        let add_leading_gap = max_created_width == 0 && max_updated_width == 0;
        if add_leading_gap {
            preview_width = preview_width.saturating_sub(2);
        }
        let preview = truncate_text(&row.preview, preview_width);
        let mut spans: Vec<Span> = vec![marker];
        if let Some(created) = created_span {
            spans.push(created);
            spans.push("  ".into());
        }
        if let Some(updated) = updated_span {
            spans.push(updated);
            spans.push("  ".into());
        }
        if add_leading_gap {
            spans.push("  ".into());
        }
        spans.push(preview.into());

        let line: Line = spans.into();
>>>>>>> upstream/main
        let rect = Rect::new(area.x, y, area.width, 1);
        frame.render_widget_ref(line, rect);
        y = y.saturating_add(1);
    }
<<<<<<< HEAD
=======

    if state.pagination.loading.is_pending() && y < area.y.saturating_add(area.height) {
        let loading_line: Line = vec!["  ".into(), "Loading older sessions…".italic().dim()].into();
        let rect = Rect::new(area.x, y, area.width, 1);
        frame.render_widget_ref(loading_line, rect);
    }
}

fn render_empty_state_line(state: &PickerState) -> Line<'static> {
    if !state.query.is_empty() {
        if state.search_state.is_active()
            || (state.pagination.loading.is_pending() && state.pagination.next_cursor.is_some())
        {
            return vec!["Searching…".italic().dim()].into();
        }
        if state.pagination.reached_scan_cap {
            let msg = format!(
                "Search scanned first {} sessions; more may exist",
                state.pagination.num_scanned_files
            );
            return vec![Span::from(msg).italic().dim()].into();
        }
        return vec!["No results for your search".italic().dim()].into();
    }

    if state.all_rows.is_empty() && state.pagination.num_scanned_files == 0 {
        return vec!["No sessions yet".italic().dim()].into();
    }

    if state.pagination.loading.is_pending() {
        return vec!["Loading older sessions…".italic().dim()].into();
    }

    vec!["No sessions yet".italic().dim()].into()
>>>>>>> upstream/main
}

fn human_time_ago(ts: DateTime<Utc>) -> String {
    let now = Utc::now();
    let delta = now - ts;
    let secs = delta.num_seconds();
    if secs < 60 {
        let n = secs.max(0);
        if n == 1 {
            format!("{n} second ago")
        } else {
            format!("{n} seconds ago")
        }
    } else if secs < 60 * 60 {
        let m = secs / 60;
        if m == 1 {
            format!("{m} minute ago")
        } else {
            format!("{m} minutes ago")
        }
    } else if secs < 60 * 60 * 24 {
        let h = secs / 3600;
        if h == 1 {
            format!("{h} hour ago")
        } else {
            format!("{h} hours ago")
        }
    } else {
        let d = secs / (60 * 60 * 24);
        if d == 1 {
            format!("{d} day ago")
        } else {
            format!("{d} days ago")
        }
    }
}

<<<<<<< HEAD
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
=======
fn format_created_label(row: &Row) -> String {
    row.created_at
        .map(human_time_ago)
        .unwrap_or_else(|| "-".to_string())
}

fn format_updated_label(row: &Row) -> String {
    match (row.updated_at, row.created_at) {
        (Some(updated), _) => human_time_ago(updated),
        (None, Some(created)) => human_time_ago(created),
        (None, None) => "-".to_string(),
    }
}

fn render_column_headers(
    frame: &mut crate::custom_terminal::Frame,
    area: Rect,
    metrics: &ColumnMetrics,
) {
    if area.height == 0 {
        return;
    }

    let mut spans: Vec<Span> = vec!["  ".into()];
    if metrics.max_created_width > 0 {
        let label = format!(
            "{text:<width$}",
            text = "Created",
            width = metrics.max_created_width
        );
        spans.push(Span::from(label).bold());
        spans.push("  ".into());
    }
    if metrics.max_updated_width > 0 {
        let label = format!(
            "{text:<width$}",
            text = "Updated",
            width = metrics.max_updated_width
        );
        spans.push(Span::from(label).bold());
        spans.push("  ".into());
    }
    spans.push("Conversation".bold());
    frame.render_widget_ref(Line::from(spans), area);
}

struct ColumnMetrics {
    max_created_width: usize,
    max_updated_width: usize,
    labels: Vec<(String, String)>,
}

fn calculate_column_metrics(rows: &[Row]) -> ColumnMetrics {
    let mut labels: Vec<(String, String)> = Vec::with_capacity(rows.len());
    let mut max_created_width = UnicodeWidthStr::width("Created");
    let mut max_updated_width = UnicodeWidthStr::width("Updated");

    for row in rows {
        let created = format_created_label(row);
        let updated = format_updated_label(row);
        max_created_width = max_created_width.max(UnicodeWidthStr::width(created.as_str()));
        max_updated_width = max_updated_width.max(UnicodeWidthStr::width(updated.as_str()));
        labels.push((created, updated));
    }

    ColumnMetrics {
        max_created_width,
        max_updated_width,
        labels,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;
    use insta::assert_snapshot;
    use serde_json::json;
    use std::future::Future;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::sync::Mutex;
>>>>>>> upstream/main

    fn head_with_ts_and_user_text(ts: &str, texts: &[&str]) -> Vec<serde_json::Value> {
        vec![
            json!({ "timestamp": ts }),
            json!({
                "type": "message",
                "role": "user",
                "content": texts
                    .iter()
                    .map(|t| json!({ "type": "input_text", "text": *t }))
                    .collect::<Vec<_>>()
            }),
        ]
    }

<<<<<<< HEAD
=======
    fn make_item(path: &str, ts: &str, preview: &str) -> ConversationItem {
        ConversationItem {
            path: PathBuf::from(path),
            head: head_with_ts_and_user_text(ts, &[preview]),
            tail: Vec::new(),
            created_at: Some(ts.to_string()),
            updated_at: Some(ts.to_string()),
        }
    }

    fn cursor_from_str(repr: &str) -> Cursor {
        serde_json::from_str::<Cursor>(&format!("\"{repr}\""))
            .expect("cursor format should deserialize")
    }

    fn page(
        items: Vec<ConversationItem>,
        next_cursor: Option<Cursor>,
        num_scanned_files: usize,
        reached_scan_cap: bool,
    ) -> ConversationsPage {
        ConversationsPage {
            items,
            next_cursor,
            num_scanned_files,
            reached_scan_cap,
        }
    }

    fn block_on_future<F: Future<Output = T>, T>(future: F) -> T {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(future)
    }

>>>>>>> upstream/main
    #[test]
    fn preview_uses_first_message_input_text() {
        let head = vec![
            json!({ "timestamp": "2025-01-01T00:00:00Z" }),
            json!({
                "type": "message",
                "role": "user",
                "content": [
                    { "type": "input_text", "text": "<user_instructions>hi</user_instructions>" },
                    { "type": "input_text", "text": "real question" },
                    { "type": "input_image", "image_url": "ignored" }
                ]
            }),
            json!({
                "type": "message",
                "role": "user",
                "content": [ { "type": "input_text", "text": "later text" } ]
            }),
        ];
        let preview = preview_from_head(&head);
        assert_eq!(preview.as_deref(), Some("real question"));
    }

    #[test]
<<<<<<< HEAD
    fn to_rows_preserves_backend_order() {
=======
    fn rows_from_items_preserves_backend_order() {
>>>>>>> upstream/main
        // Construct two items with different timestamps and real user text.
        let a = ConversationItem {
            path: PathBuf::from("/tmp/a.jsonl"),
            head: head_with_ts_and_user_text("2025-01-01T00:00:00Z", &["A"]),
<<<<<<< HEAD
=======
            tail: Vec::new(),
            created_at: Some("2025-01-01T00:00:00Z".into()),
            updated_at: Some("2025-01-01T00:00:00Z".into()),
>>>>>>> upstream/main
        };
        let b = ConversationItem {
            path: PathBuf::from("/tmp/b.jsonl"),
            head: head_with_ts_and_user_text("2025-01-02T00:00:00Z", &["B"]),
<<<<<<< HEAD
        };
        let rows = to_rows(ConversationsPage {
            items: vec![a, b],
            next_cursor: None,
            num_scanned_files: 0,
            reached_scan_cap: false,
        });
        assert_eq!(rows.len(), 2);
        // Preserve the given order; backend already provides newest-first
        assert!(rows[0].preview.contains('A'));
        assert!(rows[1].preview.contains('B'));
    }
=======
            tail: Vec::new(),
            created_at: Some("2025-01-02T00:00:00Z".into()),
            updated_at: Some("2025-01-02T00:00:00Z".into()),
        };
        let rows = rows_from_items(vec![a, b]);
        assert_eq!(rows.len(), 2);
        // Preserve the given order even if timestamps differ; backend already provides newest-first.
        assert!(rows[0].preview.contains('A'));
        assert!(rows[1].preview.contains('B'));
    }

    #[test]
    fn row_uses_tail_timestamp_for_updated_at() {
        let head = head_with_ts_and_user_text("2025-01-01T00:00:00Z", &["Hello"]);
        let tail = vec![json!({
            "timestamp": "2025-01-01T01:00:00Z",
            "type": "message",
            "role": "assistant",
            "content": [
                {
                    "type": "output_text",
                    "text": "hi",
                }
            ],
        })];
        let item = ConversationItem {
            path: PathBuf::from("/tmp/a.jsonl"),
            head,
            tail,
            created_at: Some("2025-01-01T00:00:00Z".into()),
            updated_at: Some("2025-01-01T01:00:00Z".into()),
        };

        let row = head_to_row(&item);
        let expected_created = chrono::DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let expected_updated = chrono::DateTime::parse_from_rfc3339("2025-01-01T01:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        assert_eq!(row.created_at, Some(expected_created));
        assert_eq!(row.updated_at, Some(expected_updated));
    }

    #[test]
    fn resume_table_snapshot() {
        use crate::custom_terminal::Terminal;
        use crate::test_backend::VT100Backend;
        use ratatui::layout::Constraint;
        use ratatui::layout::Layout;

        let loader: PageLoader = Arc::new(|_| {});
        let mut state =
            PickerState::new(PathBuf::from("/tmp"), FrameRequester::test_dummy(), loader);

        let now = Utc::now();
        let rows = vec![
            Row {
                path: PathBuf::from("/tmp/a.jsonl"),
                preview: String::from("Fix resume picker timestamps"),
                created_at: Some(now - Duration::minutes(16)),
                updated_at: Some(now - Duration::seconds(42)),
            },
            Row {
                path: PathBuf::from("/tmp/b.jsonl"),
                preview: String::from("Investigate lazy pagination cap"),
                created_at: Some(now - Duration::hours(1)),
                updated_at: Some(now - Duration::minutes(35)),
            },
            Row {
                path: PathBuf::from("/tmp/c.jsonl"),
                preview: String::from("Explain the codebase"),
                created_at: Some(now - Duration::hours(2)),
                updated_at: Some(now - Duration::hours(2)),
            },
        ];
        state.all_rows = rows.clone();
        state.filtered_rows = rows;
        state.view_rows = Some(3);
        state.selected = 1;
        state.scroll_top = 0;
        state.update_view_rows(3);

        let metrics = calculate_column_metrics(&state.filtered_rows);

        let width: u16 = 80;
        let height: u16 = 6;
        let backend = VT100Backend::new(width, height);
        let mut terminal = Terminal::with_options(backend).expect("terminal");
        terminal.set_viewport_area(Rect::new(0, 0, width, height));

        {
            let mut frame = terminal.get_frame();
            let area = frame.area();
            let segments =
                Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(area);
            render_column_headers(&mut frame, segments[0], &metrics);
            render_list(&mut frame, segments[1], &state, &metrics);
        }
        terminal.flush().expect("flush");

        let snapshot = terminal.backend().to_string();
        assert_snapshot!("resume_picker_table", snapshot);
    }

    #[test]
    fn pageless_scrolling_deduplicates_and_keeps_order() {
        let loader: PageLoader = Arc::new(|_| {});
        let mut state =
            PickerState::new(PathBuf::from("/tmp"), FrameRequester::test_dummy(), loader);

        state.reset_pagination();
        state.ingest_page(page(
            vec![
                make_item("/tmp/a.jsonl", "2025-01-03T00:00:00Z", "third"),
                make_item("/tmp/b.jsonl", "2025-01-02T00:00:00Z", "second"),
            ],
            Some(cursor_from_str(
                "2025-01-02T00-00-00|00000000-0000-0000-0000-000000000000",
            )),
            2,
            false,
        ));

        state.ingest_page(page(
            vec![
                make_item("/tmp/a.jsonl", "2025-01-03T00:00:00Z", "duplicate"),
                make_item("/tmp/c.jsonl", "2025-01-01T00:00:00Z", "first"),
            ],
            Some(cursor_from_str(
                "2025-01-01T00-00-00|00000000-0000-0000-0000-000000000001",
            )),
            2,
            false,
        ));

        state.ingest_page(page(
            vec![make_item(
                "/tmp/d.jsonl",
                "2024-12-31T23:00:00Z",
                "very old",
            )],
            None,
            1,
            false,
        ));

        let previews: Vec<_> = state
            .filtered_rows
            .iter()
            .map(|row| row.preview.as_str())
            .collect();
        assert_eq!(previews, vec!["third", "second", "first", "very old"]);

        let unique_paths = state
            .filtered_rows
            .iter()
            .map(|row| row.path.clone())
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(unique_paths.len(), 4);
    }

    #[test]
    fn ensure_minimum_rows_prefetches_when_underfilled() {
        let recorded_requests: Arc<Mutex<Vec<PageLoadRequest>>> = Arc::new(Mutex::new(Vec::new()));
        let request_sink = recorded_requests.clone();
        let loader: PageLoader = Arc::new(move |req: PageLoadRequest| {
            request_sink.lock().unwrap().push(req);
        });

        let mut state =
            PickerState::new(PathBuf::from("/tmp"), FrameRequester::test_dummy(), loader);
        state.reset_pagination();
        state.ingest_page(page(
            vec![
                make_item("/tmp/a.jsonl", "2025-01-01T00:00:00Z", "one"),
                make_item("/tmp/b.jsonl", "2025-01-02T00:00:00Z", "two"),
            ],
            Some(cursor_from_str(
                "2025-01-03T00-00-00|00000000-0000-0000-0000-000000000000",
            )),
            2,
            false,
        ));

        assert!(recorded_requests.lock().unwrap().is_empty());
        state.ensure_minimum_rows_for_view(10);
        let guard = recorded_requests.lock().unwrap();
        assert_eq!(guard.len(), 1);
        assert!(guard[0].search_token.is_none());
    }

    #[test]
    fn page_navigation_uses_view_rows() {
        let loader: PageLoader = Arc::new(|_| {});
        let mut state =
            PickerState::new(PathBuf::from("/tmp"), FrameRequester::test_dummy(), loader);

        let mut items = Vec::new();
        for idx in 0..20 {
            let ts = format!("2025-01-{:02}T00:00:00Z", idx + 1);
            let preview = format!("item-{idx}");
            let path = format!("/tmp/item-{idx}.jsonl");
            items.push(make_item(&path, &ts, &preview));
        }

        state.reset_pagination();
        state.ingest_page(page(items, None, 20, false));
        state.update_view_rows(5);

        assert_eq!(state.selected, 0);
        block_on_future(async {
            state
                .handle_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE))
                .await
                .unwrap();
        });
        assert_eq!(state.selected, 5);

        block_on_future(async {
            state
                .handle_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE))
                .await
                .unwrap();
        });
        assert_eq!(state.selected, 10);

        block_on_future(async {
            state
                .handle_key(KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE))
                .await
                .unwrap();
        });
        assert_eq!(state.selected, 5);
    }

    #[test]
    fn up_at_bottom_does_not_scroll_when_visible() {
        let loader: PageLoader = Arc::new(|_| {});
        let mut state =
            PickerState::new(PathBuf::from("/tmp"), FrameRequester::test_dummy(), loader);

        let mut items = Vec::new();
        for idx in 0..10 {
            let ts = format!("2025-02-{:02}T00:00:00Z", idx + 1);
            let preview = format!("item-{idx}");
            let path = format!("/tmp/item-{idx}.jsonl");
            items.push(make_item(&path, &ts, &preview));
        }

        state.reset_pagination();
        state.ingest_page(page(items, None, 10, false));
        state.update_view_rows(5);

        state.selected = state.filtered_rows.len().saturating_sub(1);
        state.ensure_selected_visible();

        let initial_top = state.scroll_top;
        assert_eq!(initial_top, state.filtered_rows.len().saturating_sub(5));

        block_on_future(async {
            state
                .handle_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE))
                .await
                .unwrap();
        });

        assert_eq!(state.scroll_top, initial_top);
        assert_eq!(state.selected, state.filtered_rows.len().saturating_sub(2));
    }

    #[test]
    fn set_query_loads_until_match_and_respects_scan_cap() {
        let recorded_requests: Arc<Mutex<Vec<PageLoadRequest>>> = Arc::new(Mutex::new(Vec::new()));
        let request_sink = recorded_requests.clone();
        let loader: PageLoader = Arc::new(move |req: PageLoadRequest| {
            request_sink.lock().unwrap().push(req);
        });

        let mut state =
            PickerState::new(PathBuf::from("/tmp"), FrameRequester::test_dummy(), loader);
        state.reset_pagination();
        state.ingest_page(page(
            vec![make_item(
                "/tmp/start.jsonl",
                "2025-01-01T00:00:00Z",
                "alpha",
            )],
            Some(cursor_from_str(
                "2025-01-02T00-00-00|00000000-0000-0000-0000-000000000000",
            )),
            1,
            false,
        ));
        recorded_requests.lock().unwrap().clear();

        state.set_query("target".to_string());
        let first_request = {
            let guard = recorded_requests.lock().unwrap();
            assert_eq!(guard.len(), 1);
            guard[0].clone()
        };

        state
            .handle_background_event(BackgroundEvent::PageLoaded {
                request_token: first_request.request_token,
                search_token: first_request.search_token,
                page: Ok(page(
                    vec![make_item("/tmp/beta.jsonl", "2025-01-02T00:00:00Z", "beta")],
                    Some(cursor_from_str(
                        "2025-01-03T00-00-00|00000000-0000-0000-0000-000000000001",
                    )),
                    5,
                    false,
                )),
            })
            .unwrap();

        let second_request = {
            let guard = recorded_requests.lock().unwrap();
            assert_eq!(guard.len(), 2);
            guard[1].clone()
        };
        assert!(state.search_state.is_active());
        assert!(state.filtered_rows.is_empty());

        state
            .handle_background_event(BackgroundEvent::PageLoaded {
                request_token: second_request.request_token,
                search_token: second_request.search_token,
                page: Ok(page(
                    vec![make_item(
                        "/tmp/match.jsonl",
                        "2025-01-03T00:00:00Z",
                        "target log",
                    )],
                    Some(cursor_from_str(
                        "2025-01-04T00-00-00|00000000-0000-0000-0000-000000000002",
                    )),
                    7,
                    false,
                )),
            })
            .unwrap();

        assert!(!state.filtered_rows.is_empty());
        assert!(!state.search_state.is_active());

        recorded_requests.lock().unwrap().clear();
        state.set_query("missing".to_string());
        let active_request = {
            let guard = recorded_requests.lock().unwrap();
            assert_eq!(guard.len(), 1);
            guard[0].clone()
        };

        state
            .handle_background_event(BackgroundEvent::PageLoaded {
                request_token: second_request.request_token,
                search_token: second_request.search_token,
                page: Ok(page(Vec::new(), None, 0, false)),
            })
            .unwrap();
        assert_eq!(recorded_requests.lock().unwrap().len(), 1);

        state
            .handle_background_event(BackgroundEvent::PageLoaded {
                request_token: active_request.request_token,
                search_token: active_request.search_token,
                page: Ok(page(Vec::new(), None, 3, true)),
            })
            .unwrap();

        assert!(state.filtered_rows.is_empty());
        assert!(!state.search_state.is_active());
        assert!(state.pagination.reached_scan_cap);
    }
>>>>>>> upstream/main
}
