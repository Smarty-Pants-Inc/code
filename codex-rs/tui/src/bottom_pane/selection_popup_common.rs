use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
// Note: Table-based layout previously used Constraint; the manual renderer
// below no longer requires it.
use ratatui::style::Color;
<<<<<<< HEAD
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Block;
use ratatui::widgets::BorderType;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;
=======
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::text::Span;
>>>>>>> upstream/main
use ratatui::widgets::Widget;
use unicode_width::UnicodeWidthChar;

use crate::key_hint::KeyBinding;

use super::scroll_state::ScrollState;

/// A generic representation of a display row for selection popups.
pub(crate) struct GenericDisplayRow {
    pub name: String,
    pub display_shortcut: Option<KeyBinding>,
    pub match_indices: Option<Vec<usize>>, // indices to bold (char positions)
    pub is_current: bool,
    pub description: Option<String>, // optional grey text after the name
}

/// Compute a shared description-column start based on the widest visible name
/// plus two spaces of padding. Ensures at least one column is left for the
/// description.
fn compute_desc_col(
    rows_all: &[GenericDisplayRow],
    start_idx: usize,
    visible_items: usize,
    content_width: u16,
) -> usize {
    let visible_range = start_idx..(start_idx + visible_items);
    let max_name_width = rows_all
        .iter()
        .enumerate()
        .filter(|(i, _)| visible_range.contains(i))
        .map(|(_, r)| Line::from(r.name.clone()).width())
        .max()
        .unwrap_or(0);
    let mut desc_col = max_name_width.saturating_add(2);
    if (desc_col as u16) >= content_width {
        desc_col = content_width.saturating_sub(1) as usize;
    }
    desc_col
}

/// Build the full display line for a row with the description padded to start
/// at `desc_col`. Applies fuzzy-match bolding when indices are present and
/// dims the description.
fn build_full_line(row: &GenericDisplayRow, desc_col: usize) -> Line<'static> {
    // Enforce single-line name: allow at most desc_col - 2 cells for name,
    // reserving two spaces before the description column.
    let name_limit = desc_col.saturating_sub(2);

    let mut name_spans: Vec<Span> = Vec::with_capacity(row.name.len());
    let mut used_width = 0usize;
    let mut truncated = false;

    if let Some(idxs) = row.match_indices.as_ref() {
        let mut idx_iter = idxs.iter().peekable();
        for (char_idx, ch) in row.name.chars().enumerate() {
            let ch_w = UnicodeWidthChar::width(ch).unwrap_or(0);
            if used_width + ch_w > name_limit {
                truncated = true;
                break;
            }
            used_width += ch_w;

            if idx_iter.peek().is_some_and(|next| **next == char_idx) {
                idx_iter.next();
                name_spans.push(ch.to_string().bold());
            } else {
                name_spans.push(ch.to_string().into());
            }
        }
    } else {
        for ch in row.name.chars() {
            let ch_w = UnicodeWidthChar::width(ch).unwrap_or(0);
            if used_width + ch_w > name_limit {
                truncated = true;
                break;
            }
            used_width += ch_w;
            name_spans.push(ch.to_string().into());
        }
    }

    if truncated {
        // If there is at least one cell available, add an ellipsis.
        // When name_limit is 0, we still show an ellipsis to indicate truncation.
        name_spans.push("…".into());
    }

    let this_name_width = Line::from(name_spans.clone()).width();
    let mut full_spans: Vec<Span> = name_spans;
    if let Some(display_shortcut) = row.display_shortcut {
        full_spans.push(" ".into());
        full_spans.push(display_shortcut.into());
    }
    if let Some(desc) = row.description.as_ref() {
        let gap = desc_col.saturating_sub(this_name_width);
        if gap > 0 {
            full_spans.push(" ".repeat(gap).into());
        }
        full_spans.push(desc.clone().dim());
    }
    Line::from(full_spans)
}

/// Compute a shared description-column start based on the widest visible name
/// plus two spaces of padding. Ensures at least one column is left for the
/// description.
fn compute_desc_col(
    rows_all: &[GenericDisplayRow],
    start_idx: usize,
    visible_items: usize,
    content_width: u16,
) -> usize {
    let visible_range = start_idx..(start_idx + visible_items);
    let max_name_width = rows_all
        .iter()
        .enumerate()
        .filter(|(i, _)| visible_range.contains(i))
        .map(|(_, r)| Line::from(r.name.clone()).width())
        .max()
        .unwrap_or(0);
    let mut desc_col = max_name_width.saturating_add(2);
    if (desc_col as u16) >= content_width {
        desc_col = content_width.saturating_sub(1) as usize;
    }
    desc_col
}

/// Build the full display line for a row with the description padded to start
/// at `desc_col`. Applies fuzzy-match bolding when indices are present and
/// dims the description.
fn build_full_line(row: &GenericDisplayRow, desc_col: usize) -> Line<'static> {
    let mut name_spans: Vec<Span> = Vec::with_capacity(row.name.len());
    if let Some(idxs) = row.match_indices.as_ref() {
        let mut idx_iter = idxs.iter().peekable();
        for (char_idx, ch) in row.name.chars().enumerate() {
            if idx_iter.peek().is_some_and(|next| **next == char_idx) {
                idx_iter.next();
                name_spans.push(ch.to_string().bold());
            } else {
                name_spans.push(ch.to_string().into());
            }
        }
    } else {
        name_spans.push(row.name.clone().into());
    }

    let this_name_width = Line::from(name_spans.clone()).width();
    let mut full_spans: Vec<Span> = name_spans;
    if let Some(desc) = row.description.as_ref() {
        let gap = desc_col.saturating_sub(this_name_width);
        if gap > 0 {
            full_spans.push(" ".repeat(gap).into());
        }
        full_spans.push(desc.clone().dim());
    }
    Line::from(full_spans)
}

/// Render a list of rows using the provided ScrollState, with shared styling
/// and behavior for selection popups.
pub(crate) fn render_rows(
    area: Rect,
    buf: &mut Buffer,
    rows_all: &[GenericDisplayRow],
    state: &ScrollState,
    max_results: usize,
<<<<<<< HEAD
    _dim_non_selected: bool,
    empty_message: &str,
) {
    // Always draw a dim left border to match other popups.
    let block = Block::default()
        .borders(Borders::LEFT)
        .border_type(BorderType::QuadrantOutside)
        .border_style(Style::default().add_modifier(Modifier::DIM));
    block.render(area, buf);

    // Content renders to the right of the border.
    let content_area = Rect {
        x: area.x.saturating_add(1),
        y: area.y,
        width: area.width.saturating_sub(1),
        height: area.height,
    };

    if rows_all.is_empty() {
        if content_area.height > 0 {
            let para = Paragraph::new(Line::from(empty_message.dim().italic()));
            para.render(
                Rect {
                    x: content_area.x,
                    y: content_area.y,
                    width: content_area.width,
                    height: 1,
                },
                buf,
            );
=======
    empty_message: &str,
) {
    if rows_all.is_empty() {
        if area.height > 0 {
            Line::from(empty_message.dim().italic()).render(area, buf);
>>>>>>> upstream/main
        }
        return;
    }

    // Determine which logical rows (items) are visible given the selection and
    // the max_results clamp. Scrolling is still item-based for simplicity.
<<<<<<< HEAD
    let max_rows_from_area = content_area.height as usize;
    let visible_items = max_results
        .min(rows_all.len())
        .min(max_rows_from_area.max(1));
=======
    let visible_items = max_results
        .min(rows_all.len())
        .min(area.height.max(1) as usize);
>>>>>>> upstream/main

    let mut start_idx = state.scroll_top.min(rows_all.len().saturating_sub(1));
    if let Some(sel) = state.selected_idx {
        if sel < start_idx {
            start_idx = sel;
        } else if visible_items > 0 {
            let bottom = start_idx + visible_items - 1;
            if sel > bottom {
                start_idx = sel + 1 - visible_items;
            }
        }
    }

<<<<<<< HEAD
    let desc_col = compute_desc_col(rows_all, start_idx, visible_items, content_area.width);

    // Render items, wrapping descriptions and aligning wrapped lines under the
    // shared description column. Stop when we run out of vertical space.
    let mut cur_y = content_area.y;
=======
    let desc_col = compute_desc_col(rows_all, start_idx, visible_items, area.width);

    // Render items, wrapping descriptions and aligning wrapped lines under the
    // shared description column. Stop when we run out of vertical space.
    let mut cur_y = area.y;
>>>>>>> upstream/main
    for (i, row) in rows_all
        .iter()
        .enumerate()
        .skip(start_idx)
        .take(visible_items)
    {
<<<<<<< HEAD
        if cur_y >= content_area.y + content_area.height {
=======
        if cur_y >= area.y + area.height {
>>>>>>> upstream/main
            break;
        }

        let GenericDisplayRow {
            name,
            match_indices,
<<<<<<< HEAD
=======
            display_shortcut,
>>>>>>> upstream/main
            is_current: _is_current,
            description,
        } = row;

<<<<<<< HEAD
        let full_line = build_full_line(
            &GenericDisplayRow {
                name: name.clone(),
                match_indices: match_indices.clone(),
=======
        let mut full_line = build_full_line(
            &GenericDisplayRow {
                name: name.clone(),
                match_indices: match_indices.clone(),
                display_shortcut: *display_shortcut,
>>>>>>> upstream/main
                is_current: *_is_current,
                description: description.clone(),
            },
            desc_col,
        );
<<<<<<< HEAD
=======
        if Some(i) == state.selected_idx {
            // Match previous behavior: cyan + bold for the selected row.
            full_line.spans.iter_mut().for_each(|span| {
                span.style = span.style.fg(Color::Cyan).bold();
            });
        }
>>>>>>> upstream/main

        // Wrap with subsequent indent aligned to the description column.
        use crate::wrapping::RtOptions;
        use crate::wrapping::word_wrap_line;
<<<<<<< HEAD
        let options = RtOptions::new(content_area.width as usize)
=======
        let options = RtOptions::new(area.width as usize)
>>>>>>> upstream/main
            .initial_indent(Line::from(""))
            .subsequent_indent(Line::from(" ".repeat(desc_col)));
        let wrapped = word_wrap_line(&full_line, options);

        // Render the wrapped lines.
<<<<<<< HEAD
        for mut line in wrapped {
            if cur_y >= content_area.y + content_area.height {
                break;
            }
            if Some(i) == state.selected_idx {
                // Match previous behavior: cyan + bold for the selected row.
                line.style = Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD);
            }
            let para = Paragraph::new(line);
            para.render(
                Rect {
                    x: content_area.x,
                    y: cur_y,
                    width: content_area.width,
=======
        for line in wrapped {
            if cur_y >= area.y + area.height {
                break;
            }
            line.render(
                Rect {
                    x: area.x,
                    y: cur_y,
                    width: area.width,
>>>>>>> upstream/main
                    height: 1,
                },
                buf,
            );
            cur_y = cur_y.saturating_add(1);
        }
    }
}

/// Compute the number of terminal rows required to render up to `max_results`
/// items from `rows_all` given the current scroll/selection state and the
/// available `width`. Accounts for description wrapping and alignment so the
/// caller can allocate sufficient vertical space.
pub(crate) fn measure_rows_height(
    rows_all: &[GenericDisplayRow],
    state: &ScrollState,
    max_results: usize,
    width: u16,
) -> u16 {
    if rows_all.is_empty() {
        return 1; // placeholder "no matches" line
    }

    let content_width = width.saturating_sub(1).max(1);

    let visible_items = max_results.min(rows_all.len());
    let mut start_idx = state.scroll_top.min(rows_all.len().saturating_sub(1));
    if let Some(sel) = state.selected_idx {
        if sel < start_idx {
            start_idx = sel;
        } else if visible_items > 0 {
            let bottom = start_idx + visible_items - 1;
            if sel > bottom {
                start_idx = sel + 1 - visible_items;
            }
        }
    }

    let desc_col = compute_desc_col(rows_all, start_idx, visible_items, content_width);

    use crate::wrapping::RtOptions;
    use crate::wrapping::word_wrap_line;
    let mut total: u16 = 0;
    for row in rows_all
        .iter()
        .enumerate()
        .skip(start_idx)
        .take(visible_items)
        .map(|(_, r)| r)
    {
        let full_line = build_full_line(row, desc_col);
        let opts = RtOptions::new(content_width as usize)
            .initial_indent(Line::from(""))
            .subsequent_indent(Line::from(" ".repeat(desc_col)));
        total = total.saturating_add(word_wrap_line(&full_line, opts).len() as u16);
    }
    total.max(1)
}
