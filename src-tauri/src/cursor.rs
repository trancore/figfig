use crate::native::Caret;
use figfig_core::{CompletionRequest, CompletionResponse, CursorPosition};
use std::time::{Duration, Instant};

pub const CURSOR_TIMEOUT: Duration = Duration::from_millis(500);

#[derive(Debug, PartialEq, Eq)]
pub enum TrackerState {
    NotRegistered,
    Disabled,
    Inactive,
    Active,
}

impl TrackerState {
    pub fn from_flags(registered: bool, enabled: bool, selected: bool) -> Self {
        match (registered, enabled, selected) {
            (false, _, _) => Self::NotRegistered,
            (_, false, _) => Self::Disabled,
            (_, _, false) => Self::Inactive,
            _ => Self::Active,
        }
    }

    pub fn require_active(self) -> Result<(), String> {
        match self {
            Self::NotRegistered => Err("Cursor Tracking is not registered. Run `figfig setup terminal`. After a first installation, log out of macOS and back in, then retry setup.".into()),
            Self::Disabled => Err("Cursor Tracking is registered but disabled by macOS. Log out of macOS and back in (restarting the editor alone is not sufficient), then run `figfig setup terminal` again. Accessibility permission does not enable this palette input method.".into()),
            Self::Inactive => Err("Cursor Tracking is enabled but not selected. Run `figfig setup terminal` again, then restart your editor and open a new terminal.".into()),
            Self::Active => Ok(()),
        }
    }
}

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
    fn cursor_setup_diagnoses_registration_enablement_and_selection_separately() {
        for (registered, enabled, selected, expected) in [
            (false, false, false, TrackerState::NotRegistered),
            (true, false, false, TrackerState::Disabled),
            (true, false, true, TrackerState::Disabled),
            (true, true, false, TrackerState::Inactive),
            (true, true, true, TrackerState::Active),
        ] {
            assert_eq!(
                TrackerState::from_flags(registered, enabled, selected),
                expected
            );
        }
        assert!(TrackerState::Active.require_active().is_ok());
        let error = TrackerState::Disabled.require_active().unwrap_err();
        assert!(error.contains("registered but disabled"));
        assert!(error.contains("Log out of macOS"));
        assert!(!error.contains("Allow"));
    }

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
                    terminal_size: None,
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
            coordinate_system: Default::default(),
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
