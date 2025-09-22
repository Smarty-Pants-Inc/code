//! Exec-layer overlays. Encapsulates non-upstream behavior so future merges
//! stay localized to this crate.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, OnceLock};

use codex_protocol::protocol::{Event, EventMsg, TaskCompleteEvent};

/// Default number of events to retain in the observer buffer.
pub const DEFAULT_BUFFER_CAPACITY: usize = 512;

/// Wrapper type that allows the SDK to attach additional state to the upstream
/// executor without modifying its source.
#[derive(Debug, Default, Clone)]
pub struct ExecOverlayState {
    pub last_agent_message: Option<String>,
}

impl ExecOverlayState {
    pub fn clear(&mut self) {
        self.last_agent_message = None;
    }

    fn update_from_msg(&mut self, msg: &EventMsg) {
        if let EventMsg::TaskComplete(TaskCompleteEvent { last_agent_message }) = msg {
            self.last_agent_message = last_agent_message.clone();
        }
    }
}

/// Lightweight, clonable representation of an event suitable for buffering.
#[derive(Debug, Clone)]
pub struct BufferedEvent {
    pub id: String,
    pub msg: EventMsg,
}

impl BufferedEvent {
    pub fn new(id: String, msg: EventMsg) -> Self {
        Self { id, msg }
    }
}

impl From<&Event> for BufferedEvent {
    fn from(event: &Event) -> Self {
        Self {
            id: event.id.clone(),
            msg: event.msg.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct EventObserver {
    inner: Arc<Mutex<EventObserverInner>>,
}

#[derive(Debug)]
struct EventObserverInner {
    capacity: usize,
    events: VecDeque<BufferedEvent>,
    state: ExecOverlayState,
}

impl EventObserver {
    pub fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        Self {
            inner: Arc::new(Mutex::new(EventObserverInner {
                capacity,
                events: VecDeque::with_capacity(capacity),
                state: ExecOverlayState::default(),
            })),
        }
    }

    pub fn record(&self, event: &Event) {
        self.record_buffered(BufferedEvent::from(event));
    }

    pub fn record_buffered(&self, event: BufferedEvent) {
        let mut guard = self.inner.lock().expect("overlay observer poisoned");
        guard.state.update_from_msg(&event.msg);
        if guard.events.len() == guard.capacity {
            guard.events.pop_front();
        }
        guard.events.push_back(event);
    }

    pub fn snapshot(&self) -> Vec<BufferedEvent> {
        let guard = self.inner.lock().expect("overlay observer poisoned");
        guard.events.iter().cloned().collect()
    }

    pub fn state(&self) -> ExecOverlayState {
        let guard = self.inner.lock().expect("overlay observer poisoned");
        guard.state.clone()
    }

    pub fn len(&self) -> usize {
        let guard = self.inner.lock().expect("overlay observer poisoned");
        guard.events.len()
    }

    pub fn capacity(&self) -> usize {
        let guard = self.inner.lock().expect("overlay observer poisoned");
        guard.capacity
    }

    pub fn clear(&self) {
        let mut guard = self.inner.lock().expect("overlay observer poisoned");
        guard.events.clear();
        guard.state.clear();
    }
}

impl Default for EventObserver {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_BUFFER_CAPACITY)
    }
}

static GLOBAL_OBSERVER: OnceLock<EventObserver> = OnceLock::new();

pub mod observer {
    use super::*;

    pub fn ensure_global_observer(capacity: usize) -> EventObserver {
        GLOBAL_OBSERVER
            .get_or_init(|| EventObserver::with_capacity(capacity))
            .clone()
    }

    pub fn global_observer() -> Option<EventObserver> {
        GLOBAL_OBSERVER.get().cloned()
    }

    pub fn record(event: &Event) {
        if let Some(observer) = GLOBAL_OBSERVER.get() {
            observer.record(event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_protocol::protocol::{BackgroundEventEvent, TaskCompleteEvent};

    fn background_event(id: &str, message: &str) -> Event {
        Event {
            id: id.to_owned(),
            msg: EventMsg::BackgroundEvent(BackgroundEventEvent {
                message: message.to_owned(),
            }),
        }
    }

    fn task_complete(id: &str, last: Option<&str>) -> Event {
        Event {
            id: id.to_owned(),
            msg: EventMsg::TaskComplete(TaskCompleteEvent {
                last_agent_message: last.map(|s| s.to_owned()),
            }),
        }
    }

    #[test]
    fn observer_trims_to_capacity() {
        let observer = EventObserver::with_capacity(3);
        observer.record(&background_event("1", "a"));
        observer.record(&background_event("2", "b"));
        observer.record(&background_event("3", "c"));
        observer.record(&background_event("4", "d"));

        let snapshot = observer.snapshot();
        let ids: Vec<_> = snapshot.iter().map(|ev| ev.id.as_str()).collect();
        assert_eq!(ids, vec!["2", "3", "4"]);
        assert_eq!(observer.len(), 3);
    }

    #[test]
    fn observer_tracks_last_agent_message() {
        let observer = EventObserver::default();
        observer.record(&task_complete("1", Some("done")));
        let state = observer.state();
        assert_eq!(state.last_agent_message.as_deref(), Some("done"));

        observer.record(&task_complete("2", None));
        let state = observer.state();
        assert!(state.last_agent_message.is_none());
    }

    #[test]
    fn observer_clear_resets_state() {
        let observer = EventObserver::default();
        observer.record(&task_complete("1", Some("done")));
        observer.clear();
        assert_eq!(observer.len(), 0);
        assert!(observer.state().last_agent_message.is_none());
    }
}
