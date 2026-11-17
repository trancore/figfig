use crate::native;
use anyhow::Result;
use figfig_core::{
    runtime, CompletionRequest, CompletionResponse, Engine, ServerMessage, SessionState, Settings,
    PROTOCOL_VERSION,
};
use std::sync::{Arc, Mutex};
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
        if !self.inner.lock().unwrap().session.begin(&request) {
            return;
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
        let caret = native::caret(0);
        let inner = self.inner.lock().unwrap();
        ServerMessage::Status {
            version: PROTOCOL_VERSION,
            paused: inner.paused,
            accessibility: native::trusted(),
            terminal: caret
                .as_ref()
                .ok()
                .and_then(|caret| native::terminal_name(caret.pid)),
            error: caret.err().or_else(|| inner.error.clone()),
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
        mut response: CompletionResponse,
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
        let caret = if enabled {
            native::caret(request.terminal_pid)
        } else {
            Err("No active suggestions".into())
        };
        response.visible = false;
        if enabled {
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
        let old = {
            let inner = self.inner.lock().unwrap();
            if !inner.popup.visible {
                return;
            }
            inner.caret
        };
        let Some(old) = old else {
            return;
        };
        match native::caret(old.pid) {
            Ok(current) if current.focus == old.focus => {
                if current != old {
                    let count = self.inner.lock().unwrap().popup.candidates.len();
                    if let Some(window) = app.get_webview_window("popup") {
                        if native::place(&window, current, POPUP_WIDTH, popup_height(count)).is_ok()
                        {
                            self.inner.lock().unwrap().caret = Some(current);
                        } else {
                            self.hide(app);
                        }
                    }
                }
            }
            _ => {
                self.inner.lock().unwrap().session.dismissed = true;
                self.hide(app);
            }
        }
    }
}

#[tauri::command]
pub fn get_completion(service: tauri::State<'_, Arc<Service>>) -> CompletionResponse {
    service.inner.lock().unwrap().popup.clone()
}
