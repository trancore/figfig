use crate::native::Caret;
use figfig_core::{CompletionRequest, CompletionResponse, CursorPosition};
use std::time::{Duration, Instant};

pub const CURSOR_TIMEOUT: Duration = Duration::from_millis(500);
pub const SETUP_HINT: &str = "Zed cursor tracking is unavailable. Run `figfig setup zed`, allow figfig Cursor Tracking, and restart Zed.";

pub enum CursorAction {
    Complete(CompletionRequest, CompletionResponse),
    Track {
        caret: Caret,
        session_id: String,
        revision: u64,
    },
}

pub struct CursorQuery {
    pub id: String,
    pub terminal_pid: i32,
    pub started: Instant,
    pub action: CursorAction,
}

impl CursorQuery {
    pub fn expired(&self, now: Instant) -> bool {
        now.saturating_duration_since(self.started) >= CURSOR_TIMEOUT
    }

    pub fn matches(&self, position: &CursorPosition, now: Instant) -> bool {
        !self.expired(now)
            && self.id == position.request_id
            && self.terminal_pid == position.terminal_pid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_current_terminals_timely_reply_is_accepted() {
        let now = Instant::now();
        let query = CursorQuery {
            id: "latest".into(),
            terminal_pid: 42,
            started: now,
            action: CursorAction::Complete(
                CompletionRequest {
                    session_id: "session".into(),
                    revision: 1,
                    buffer: "git ch".into(),
                    cursor: 6,
                    cwd: "/".into(),
                    terminal_pid: 42,
                },
                CompletionResponse::default(),
            ),
        };
        let mut position = CursorPosition {
            request_id: "latest".into(),
            terminal_pid: 42,
            x: 100.0,
            y: 200.0,
            height: 20.0,
            focus: 1,
        };
        assert!(query.matches(&position, now));
        assert!(!query.matches(&position, now + CURSOR_TIMEOUT));
        position.request_id = "older".into();
        assert!(!query.matches(&position, now));
        position.request_id = "latest".into();
        position.terminal_pid = 43;
        assert!(!query.matches(&position, now));
    }
}
