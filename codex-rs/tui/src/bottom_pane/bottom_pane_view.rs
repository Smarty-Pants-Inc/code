use crate::bottom_pane::ApprovalRequest;
use crate::render::renderable::Renderable;
use crossterm::event::KeyEvent;
use ratatui::layout::Rect;

use super::CancellationEvent;

/// Trait implemented by every view that can be shown in the bottom pane.
<<<<<<< HEAD
pub(crate) trait BottomPaneView {
    /// Handle a key event while the view is active. A redraw is always
    /// scheduled after this call.
    fn handle_key_event(&mut self, _pane: &mut BottomPane, _key_event: KeyEvent) {}
=======
pub(crate) trait BottomPaneView: Renderable {
    /// Handle a key event while the view is active. A redraw is always
    /// scheduled after this call.
    fn handle_key_event(&mut self, _key_event: KeyEvent) {}
>>>>>>> upstream/main

    /// Return `true` if the view has finished and should be removed.
    fn is_complete(&self) -> bool {
        false
    }

    /// Handle Ctrl-C while this view is active.
<<<<<<< HEAD
    fn on_ctrl_c(&mut self, _pane: &mut BottomPane) -> CancellationEvent {
        CancellationEvent::NotHandled
    }

    /// Return the desired height of the view.
    fn desired_height(&self, width: u16) -> u16;

    /// Render the view: this will be displayed in place of the composer.
    fn render(&self, area: Rect, buf: &mut Buffer);
=======
    fn on_ctrl_c(&mut self) -> CancellationEvent {
        CancellationEvent::NotHandled
    }

    /// Optional paste handler. Return true if the view modified its state and
    /// needs a redraw.
    fn handle_paste(&mut self, _pasted: String) -> bool {
        false
    }

    /// Cursor position when this view is active.
    fn cursor_pos(&self, _area: Rect) -> Option<(u16, u16)> {
        None
    }
>>>>>>> upstream/main

    /// Try to handle approval request; return the original value if not
    /// consumed.
    fn try_consume_approval_request(
        &mut self,
        request: ApprovalRequest,
    ) -> Option<ApprovalRequest> {
        Some(request)
    }
}
