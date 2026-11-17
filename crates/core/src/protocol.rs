use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 1;
pub const MAX_FRAME: usize = 256 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionRequest {
    pub session_id: String,
    pub revision: u64,
    pub buffer: String,
    /// Unicode scalar offset, matching ZLE's CURSOR in a UTF-8 locale.
    pub cursor: usize,
    pub cwd: String,
    #[serde(default)]
    pub terminal_pid: i32,
}

/// Geometry only, reported by the optional macOS cursor input method.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorPosition {
    pub request_id: String,
    pub terminal_pid: i32,
    pub x: f64,
    /// AppKit screen coordinates, with a bottom-left origin.
    pub y: f64,
    pub height: f64,
    pub focus: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub label: String,
    pub description: String,
    pub insert_text: String,
    pub kind: String,
    pub append_space: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionResponse {
    pub session_id: String,
    pub revision: u64,
    pub replace_start: usize,
    pub replace_end: usize,
    pub candidates: Vec<Candidate>,
    pub selected: usize,
    /// Only true when the native popup can actually be shown.
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ClientMessage {
    SetupCursor {
        bundle_path: String,
        enabled: bool,
    },
    CursorPosition {
        position: CursorPosition,
    },
    Complete {
        request: CompletionRequest,
    },
    Select {
        session_id: String,
        revision: u64,
        selected: usize,
    },
    Hide {
        session_id: String,
        revision: u64,
    },
    Status,
    ReloadSettings,
    SetPaused {
        paused: bool,
    },
    RequestAccessibility,
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ServerMessage {
    Completion {
        response: CompletionResponse,
    },
    Preferences {
        settings: Settings,
        paused: bool,
    },
    Status {
        version: u32,
        paused: bool,
        accessibility: bool,
        terminal: Option<String>,
        error: Option<String>,
    },
    Ack,
    Accessibility {
        granted: bool,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub popup_enabled: bool,
    pub history_enabled: bool,
    pub max_candidates: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            popup_enabled: true,
            history_enabled: true,
            max_candidates: 8,
        }
    }
}

impl Settings {
    pub fn normalize(mut self) -> Self {
        self.max_candidates = self.max_candidates.clamp(3, 12);
        self
    }
}

/// Keeps late results, hide messages, and other sessions from replacing live UI.
#[derive(Default)]
pub struct SessionState {
    pub session_id: String,
    pub revision: u64,
    pub dismissed: bool,
}

impl SessionState {
    pub fn begin(&mut self, request: &CompletionRequest) -> bool {
        if self.session_id == request.session_id && request.revision <= self.revision {
            return false;
        }
        self.session_id.clone_from(&request.session_id);
        self.revision = request.revision;
        self.dismissed = false;
        true
    }

    pub fn matches(&self, session_id: &str, revision: u64) -> bool {
        !self.dismissed && self.session_id == session_id && self.revision == revision
    }

    pub fn disconnect(&mut self, session_id: &str) -> bool {
        self.hide(session_id, self.revision)
    }

    pub fn hide(&mut self, session_id: &str, revision: u64) -> bool {
        if self.session_id == session_id && revision >= self.revision {
            self.revision = revision;
            self.dismissed = true;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn late_results_cannot_reopen_dismissed_popup() {
        let mut state = SessionState::default();
        let request = CompletionRequest {
            session_id: "shell".into(),
            revision: 1,
            buffer: "git".into(),
            cursor: 3,
            cwd: "/".into(),
            terminal_pid: 0,
        };
        assert!(state.begin(&request));
        assert!(!state.begin(&request));
        assert!(state.hide("shell", 2));
        assert!(!state.matches("shell", 1));
        assert!(!state.begin(&request));
    }

    #[test]
    fn disconnect_invalidates_pending_results_only_for_the_active_session() {
        let mut state = SessionState {
            session_id: "active".into(),
            revision: 7,
            dismissed: false,
        };
        assert!(!state.disconnect("previous"));
        assert!(state.matches("active", 7));
        assert!(state.disconnect("active"));
        assert!(!state.matches("active", 7));
        assert_eq!(state.revision, 7);
    }

    #[test]
    fn another_session_cannot_hide_the_active_one() {
        let mut state = SessionState {
            session_id: "new".into(),
            revision: 1,
            dismissed: false,
        };
        assert!(!state.hide("old", 100));
        assert!(state.matches("new", 1));
    }
}
