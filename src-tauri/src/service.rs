use crate::{
    cursor::{CursorAction, CursorQuery},
    native,
};
use anyhow::Result;
use figfig_core::{
    runtime,
    terminal::{tracker_hint, CursorBackend},
    CompletionRequest, CompletionResponse, Engine, ServerMessage, SessionState, Settings,
    PROTOCOL_VERSION,
};
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};
use tauri::{Emitter, Manager};
use tokio::sync::{broadcast, Semaphore};

pub const POPUP_WIDTH: f64 = 320.0;
pub fn popup_height(candidate_count: usize) -> f64 {
    // Matches styles.css: a 20px footer and a 1px border on each edge.
    const CHROME_HEIGHT: f64 = 22.0;
    const ROW_HEIGHT: f64 = 20.0;
    CHROME_HEIGHT + candidate_count as f64 * ROW_HEIGHT
}

struct Inner {
    settings: Settings,
    paused: bool,
    session: SessionState,
    popup: CompletionResponse,
    caret: Option<native::Caret>,
    error: Option<String>,
    cursor_query: Option<CursorQuery>,
    cursor_sequence: u64,
    terminal_pid: i32,
}

pub struct Service {
    inner: Mutex<Inner>,
    pub events: broadcast::Sender<ServerMessage>,
    engine: Arc<Engine>,
    workers: Arc<Semaphore>,
}

impl Service {
    pub fn new() -> Result<Arc<Self>> {
        let (events, _) = broadcast::channel(64);
        Ok(Arc::new(Self {
            inner: Mutex::new(Inner {
                settings: runtime::load_settings()?,
                paused: false,
                session: SessionState::default(),
                popup: CompletionResponse::default(),
                caret: None,
                error: None,
                cursor_query: None,
                cursor_sequence: 0,
                terminal_pid: 0,
            }),
            events,
            engine: Arc::new(Engine::new()),
            workers: Arc::new(Semaphore::new(2)),
        }))
    }

    pub fn request_completion(
        self: &Arc<Self>,
        app: &tauri::AppHandle,
        request: CompletionRequest,
    ) {
        {
            let mut inner = self.inner.lock().unwrap();
            if !inner.session.begin(&request) {
                return;
            }
            inner.cursor_query = None;
            inner.terminal_pid = request.terminal_pid;
            inner.error = None;
        }
        let service = self.clone();
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            let Ok(_permit) = service.workers.clone().acquire_owned().await else {
                return;
            };
            let limit = {
                let inner = service.inner.lock().unwrap();
                if !inner.session.matches(&request.session_id, request.revision) {
                    return;
                }
                inner.settings.max_candidates
            };
            let engine = service.engine.clone();
            let work_request = request.clone();
            if let Ok(response) =
                tauri::async_runtime::spawn_blocking(move || engine.complete(&work_request, limit))
                    .await
            {
                let callback_app = app.clone();
                let _ = app.run_on_main_thread(move || {
                    service.present(&callback_app, &request, response);
                });
            }
        });
    }

    pub fn select(&self, app: &tauri::AppHandle, session_id: &str, revision: u64, selected: usize) {
        let popup = {
            let mut inner = self.inner.lock().unwrap();
            if !inner.session.matches(session_id, revision)
                || selected >= inner.popup.candidates.len()
            {
                return;
            }
            inner.popup.selected = selected;
            inner.popup.clone()
        };
        let _ = app.emit_to("popup", "completion", popup);
    }

    pub fn hide_session(&self, app: &tauri::AppHandle, session_id: &str, revision: u64) {
        let should_hide = self
            .inner
            .lock()
            .unwrap()
            .session
            .hide(session_id, revision);
        if should_hide {
            self.hide(app);
        }
    }

    pub fn disconnect(&self, app: &tauri::AppHandle, session_id: &str) {
        let should_hide = self.inner.lock().unwrap().session.disconnect(session_id);
        if should_hide {
            self.hide(app);
        }
    }

    pub fn status(&self) -> ServerMessage {
        let pid = native::frontmost_pid();
        let foreground = native::terminal_host(pid);
        let caret = native::caret(pid);
        let (paused, current_error, showing_zed_popup) = {
            let inner = self.inner.lock().unwrap();
            let current_error = if inner.terminal_pid == pid {
                inner.error.clone()
            } else {
                None
            };
            let showing_zed_popup = inner.caret.is_some_and(|anchor| {
                anchor.pid == pid
                    && matches!(anchor.source, native::PositionSource::ZedWindow { .. })
            });
            (inner.paused, current_error, showing_zed_popup)
        };

        let uses_tracker =
            foreground.is_some_and(|host| host.backend == CursorBackend::InputMethod);
        let electron_caret =
            foreground.is_some_and(|host| host.family == "electron") && caret.is_ok();
        let tracker_error = if uses_tracker && !electron_caret {
            native::tracker_ready(pid).err()
        } else {
            None
        };
        let zed_window_popup = foreground.is_some_and(|host| host.family == "zed")
            && (tracker_error.is_some() || showing_zed_popup)
            && native::zed_window_anchor(pid, None).is_ok();
        let terminal = foreground
            .or_else(|| {
                caret
                    .as_ref()
                    .ok()
                    .and_then(|caret| native::terminal_host(caret.pid))
            })
            .map(|host| host.name.clone());
        let error = if zed_window_popup || (uses_tracker && electron_caret) {
            None
        } else if uses_tracker {
            tracker_error.or(current_error)
        } else {
            caret.err().or(current_error)
        };

        ServerMessage::Status {
            version: PROTOCOL_VERSION,
            paused,
            accessibility: native::trusted(),
            terminal,
            zed_window_popup,
            error,
        }
    }

    pub fn reload_settings(&self, app: &tauri::AppHandle) -> ServerMessage {
        match runtime::load_settings() {
            Ok(settings) => {
                self.inner.lock().unwrap().settings = settings;
                self.hide(app);
                let _ = self.events.send(self.preferences());
                ServerMessage::Ack
            }
            Err(error) => ServerMessage::Error {
                message: error.to_string(),
            },
        }
    }

    pub fn preferences(&self) -> ServerMessage {
        let inner = self.inner.lock().unwrap();
        ServerMessage::Preferences {
            settings: inner.settings.clone(),
            paused: inner.paused,
        }
    }

    pub fn hide(&self, app: &tauri::AppHandle) {
        let popup = {
            let mut inner = self.inner.lock().unwrap();
            inner.popup.visible = false;
            inner.caret = None;
            inner.cursor_query = None;
            inner.popup.clone()
        };
        if let Some(window) = app.get_webview_window("popup") {
            let _ = window.hide();
        }
        let _ = app.emit_to("popup", "completion", &popup);
        let _ = self
            .events
            .send(ServerMessage::Completion { response: popup });
    }

    pub fn set_paused(&self, app: &tauri::AppHandle, paused: bool) {
        self.inner.lock().unwrap().paused = paused;
        self.hide(app);
        let _ = self.events.send(self.preferences());
    }

    fn present(
        &self,
        app: &tauri::AppHandle,
        request: &CompletionRequest,
        response: CompletionResponse,
    ) {
        let mut inner = self.inner.lock().unwrap();
        if !inner
            .session
            .matches(&request.session_id, response.revision)
        {
            return;
        }
        let enabled =
            inner.settings.popup_enabled && !inner.paused && !response.candidates.is_empty();
        let host = if enabled {
            native::terminal_host(request.terminal_pid)
        } else {
            None
        };
        if host.is_some_and(|host| host.family == "electron") {
            if let Ok(caret) = native::caret(request.terminal_pid) {
                drop(inner);
                self.present_at(app, request, response, Some(Ok(caret)));
                return;
            }
        }
        if host.is_some_and(|host| host.backend == CursorBackend::InputMethod) {
            if native::frontmost_pid() != request.terminal_pid {
                drop(inner);
                self.present_at(
                    app,
                    request,
                    response,
                    Some(Err("The shell's terminal is not focused".into())),
                );
                return;
            }
            inner.popup.visible = false;
            inner.caret = None;
            if let Some(window) = app.get_webview_window("popup") {
                let _ = window.hide();
            }
            let id = Self::queue_cursor(
                &mut inner,
                request.terminal_pid,
                CursorAction::Complete(request.clone(), response),
            );
            drop(inner);
            self.dispatch_cursor(app, &id, request.terminal_pid);
            return;
        }
        drop(inner);
        self.present_at(app, request, response, None);
    }

    fn queue_cursor(inner: &mut Inner, terminal_pid: i32, action: CursorAction) -> String {
        inner.cursor_sequence += 1;
        let id = format!("{}-{}", std::process::id(), inner.cursor_sequence);
        inner.cursor_query = Some(CursorQuery {
            id: id.clone(),
            terminal_pid,
            started: Instant::now(),
            action,
        });
        id
    }

    fn dispatch_cursor(&self, app: &tauri::AppHandle, id: &str, terminal_pid: i32) {
        if let Err(error) = native::request_tracker_caret(id, terminal_pid) {
            self.cursor_failed(app, error);
        }
    }

    fn cursor_failed(&self, app: &tauri::AppHandle, error: String) {
        let query = self.inner.lock().unwrap().cursor_query.take();
        if let Some(query) = query {
            match query.action {
                CursorAction::Complete(request, response) => {
                    let anchor = if native::terminal_host(request.terminal_pid)
                        .is_some_and(|host| host.family == "zed")
                    {
                        native::zed_window_anchor(request.terminal_pid, request.terminal_size)
                    } else {
                        Err(error)
                    };
                    self.present_at(app, &request, response, Some(anchor))
                }
                CursorAction::Track { .. } => {
                    self.inner.lock().unwrap().error = Some(error);
                    self.hide(app);
                }
            }
        }
    }

    pub fn cursor_position(&self, app: &tauri::AppHandle, position: figfig_core::CursorPosition) {
        let query = {
            let mut inner = self.inner.lock().unwrap();
            if !inner
                .cursor_query
                .as_ref()
                .is_some_and(|q| q.matches(&position, Instant::now()))
            {
                return;
            }
            inner.cursor_query.take().unwrap()
        };
        let caret = if native::frontmost_pid() == position.terminal_pid {
            native::tracker_caret(&position)
        } else {
            Err("The shell's terminal is not focused".into())
        };
        match query.action {
            CursorAction::Complete(request, response) => {
                self.present_at(app, &request, response, Some(caret))
            }
            CursorAction::Track {
                caret: old,
                session_id,
                revision,
            } => {
                let mut inner = self.inner.lock().unwrap();
                if !inner.session.matches(&session_id, revision) || inner.caret != Some(old) {
                    return;
                }
                // A different location may be another pane or the editor. Wait for a
                // new shell request rather than moving stale suggestions there.
                if caret.as_ref().ok() != Some(&old) {
                    inner.session.dismissed = true;
                    inner.error = caret.err();
                    drop(inner);
                    self.hide(app);
                }
            }
        }
    }

    fn present_at(
        &self,
        app: &tauri::AppHandle,
        request: &CompletionRequest,
        mut response: CompletionResponse,
        supplied_caret: Option<Result<native::Caret, String>>,
    ) {
        let mut inner = self.inner.lock().unwrap();
        if !inner
            .session
            .matches(&response.session_id, response.revision)
        {
            return;
        }
        let enabled =
            inner.settings.popup_enabled && !inner.paused && !response.candidates.is_empty();
        response.visible = false;
        if enabled {
            let caret = supplied_caret.unwrap_or_else(|| native::caret(request.terminal_pid));
            match caret {
                Ok(caret) if request.terminal_pid > 0 => {
                    if let Some(window) = app.get_webview_window("popup") {
                        let height = popup_height(response.candidates.len());
                        let _ = window.set_size(tauri::LogicalSize::new(POPUP_WIDTH, height));
                        match native::place(&window, caret, POPUP_WIDTH, height) {
                            Ok(()) => {
                                response.visible = true;
                                inner.caret = Some(caret);
                                inner.error = None;
                            }
                            Err(error) => inner.error = Some(error),
                        }
                    }
                }
                Ok(_) => {
                    inner.error = Some("Could not identify the shell's terminal process".into())
                }
                Err(error) => inner.error = Some(error),
            }
        }
        if !response.visible {
            inner.caret = None;
            if let Some(window) = app.get_webview_window("popup") {
                let _ = window.hide();
            }
        }
        inner.popup = response.clone();
        drop(inner);
        let _ = app.emit_to("popup", "completion", &response);
        let _ = self.events.send(ServerMessage::Completion { response });
    }

    pub fn track(&self, app: &tauri::AppHandle) {
        let (pending, caret) = {
            let inner = self.inner.lock().unwrap();
            let pending = inner
                .cursor_query
                .as_ref()
                .map(|query| (query.terminal_pid, query.expired(Instant::now())));
            let caret = inner.caret.filter(|_| inner.popup.visible);
            (pending, caret)
        };
        if let Some((pid, expired)) = pending {
            if expired {
                let host = native::terminal_host(pid);
                let name = host.map_or("", |host| host.name.as_str());
                self.cursor_failed(app, tracker_hint(name).into());
            }
            return;
        }
        let Some(old) = caret else {
            return;
        };

        match old.source {
            native::PositionSource::ZedWindow { terminal_size, .. } => {
                if native::zed_window_anchor(old.pid, terminal_size).ok() != Some(old) {
                    self.dismiss_for_focus_change(app);
                }
            }
            native::PositionSource::Tracker => self.track_with_tracker(app, old),
            native::PositionSource::Accessibility => self.track_accessibility(app, old),
        }
    }

    fn dismiss_for_focus_change(&self, app: &tauri::AppHandle) {
        self.inner.lock().unwrap().session.dismissed = true;
        self.hide(app);
    }

    fn track_with_tracker(&self, app: &tauri::AppHandle, old: native::Caret) {
        if native::frontmost_pid() != old.pid {
            self.hide(app);
            return;
        }
        let id = {
            let mut inner = self.inner.lock().unwrap();
            let session_id = inner.popup.session_id.clone();
            let revision = inner.popup.revision;
            if !inner.session.matches(&session_id, revision) {
                return;
            }
            Self::queue_cursor(
                &mut inner,
                old.pid,
                CursorAction::Track {
                    caret: old,
                    session_id,
                    revision,
                },
            )
        };
        self.dispatch_cursor(app, &id, old.pid);
    }

    fn track_accessibility(&self, app: &tauri::AppHandle, old: native::Caret) {
        let Ok(current) = native::caret(old.pid) else {
            self.dismiss_for_focus_change(app);
            return;
        };
        if current.focus != old.focus {
            self.dismiss_for_focus_change(app);
        } else if current != old {
            let count = self.inner.lock().unwrap().popup.candidates.len();
            if let Some(window) = app.get_webview_window("popup") {
                if native::place(&window, current, POPUP_WIDTH, popup_height(count)).is_ok() {
                    self.inner.lock().unwrap().caret = Some(current);
                } else {
                    self.hide(app);
                }
            }
        }
    }
}

#[tauri::command]
pub fn get_completion(service: tauri::State<'_, Arc<Service>>) -> CompletionResponse {
    service.inner.lock().unwrap().popup.clone()
}
