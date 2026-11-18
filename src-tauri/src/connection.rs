use crate::{native, service::Service};
use anyhow::{bail, Context, Result};
use figfig_core::{
    runtime,
    transport::{write_frame, FrameReader},
    ClientMessage, ServerMessage,
};
use std::{
    ops::ControlFlow,
    os::unix::fs::{FileTypeExt, PermissionsExt},
    sync::Arc,
};
use tokio::{
    io::BufReader,
    net::{unix::OwnedWriteHalf, UnixListener, UnixStream},
    sync::broadcast,
};

pub fn bind_socket() -> Result<std::os::unix::net::UnixListener> {
    let path = runtime::socket_path()?;
    if let Ok(metadata) = std::fs::symlink_metadata(&path) {
        if !metadata.file_type().is_socket() {
            bail!("Refusing to replace a non-socket at {}", path.display());
        }
        if std::os::unix::net::UnixStream::connect(&path).is_ok() {
            bail!("figfig is already running");
        }
        std::fs::remove_file(&path).context("Remove stale socket")?;
    }
    let listener = std::os::unix::net::UnixListener::bind(&path)?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    listener.set_nonblocking(true)?;
    Ok(listener)
}

pub async fn serve(
    listener: std::os::unix::net::UnixListener,
    service: Arc<Service>,
    app: tauri::AppHandle,
) -> Result<()> {
    let listener = UnixListener::from_std(listener)?;
    loop {
        let (stream, _) = listener.accept().await?;
        let service = service.clone();
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(error) = connection(stream, service, app).await {
                // Never log input buffers, cwd, or history.
                eprintln!("figfig: connection closed: {error}");
            }
        });
    }
}

async fn connection(
    stream: UnixStream,
    service: Arc<Service>,
    app: tauri::AppHandle,
) -> Result<()> {
    let mut session_id = None;
    let result = receive_messages(stream, &service, &app, &mut session_id).await;
    // Run cleanup on EOF, invalid input, read failures, and write failures alike.
    if let Some(id) = session_id {
        service.disconnect(&app, &id);
    }
    result
}

async fn receive_messages(
    stream: UnixStream,
    service: &Arc<Service>,
    app: &tauri::AppHandle,
    session_id: &mut Option<String>,
) -> Result<()> {
    let (read, mut write) = stream.into_split();
    let mut reader = FrameReader::new(BufReader::new(read));
    let mut events = service.events.subscribe();
    write_frame(&mut write, &service.preferences()).await?;
    loop {
        tokio::select! {
            frame = reader.next_frame() => {
                let Some(frame) = frame? else { break; };
                let message = serde_json::from_slice(&frame).context("Invalid protocol message")?;
                if handle_message(message, &mut write, service, app, session_id).await?.is_break() {
                    break;
                }
            }
            event = events.recv() => {
                let message = match event {
                    Ok(message) => message,
                    // A desynchronized bridge must reconnect before accepting candidates.
                    Err(broadcast::error::RecvError::Lagged(_)) => break,
                    Err(broadcast::error::RecvError::Closed) => break,
                };
                let relevant = match &message {
                    ServerMessage::Completion { response } => session_id.as_ref() == Some(&response.session_id),
                    _ => true,
                };
                if relevant {
                    write_frame(&mut write, &message).await?;
                }
            }
        }
    }
    Ok(())
}

async fn handle_message(
    message: ClientMessage,
    writer: &mut OwnedWriteHalf,
    service: &Arc<Service>,
    app: &tauri::AppHandle,
    session_id: &mut Option<String>,
) -> Result<ControlFlow<()>> {
    let response = match message {
        ClientMessage::SetupCursor {
            bundle_path,
            enabled,
        } => {
            let (sender, receiver) = tokio::sync::oneshot::channel();
            app.run_on_main_thread(move || {
                let _ = sender.send(native::setup_cursor(&bundle_path, enabled));
            })?;
            match receiver.await? {
                Ok(()) => ServerMessage::Ack,
                Err(message) => ServerMessage::Error { message },
            }
        }
        ClientMessage::CursorPosition { position } => {
            let service = service.clone();
            let callback_app = app.clone();
            app.run_on_main_thread(move || service.cursor_position(&callback_app, position))?;
            return Ok(ControlFlow::Continue(()));
        }
        ClientMessage::Complete { request } => {
            if request.buffer.len() > 64 * 1024
                || request.session_id.len() > 200
                || request.cwd.len() > 4096
            {
                bail!("Request exceeds limits");
            }
            if session_id
                .as_ref()
                .is_some_and(|id| *id != request.session_id)
            {
                bail!("Session changed on an existing connection");
            }
            *session_id = Some(request.session_id.clone());
            service.request_completion(app, request);
            return Ok(ControlFlow::Continue(()));
        }
        ClientMessage::Select {
            session_id,
            revision,
            selected,
        } => {
            service.select(app, &session_id, revision, selected);
            return Ok(ControlFlow::Continue(()));
        }
        ClientMessage::Hide {
            session_id,
            revision,
        } => {
            service.hide_session(app, &session_id, revision);
            return Ok(ControlFlow::Continue(()));
        }
        ClientMessage::Status => {
            // Status can inspect AppKit/TIS state, just like cursor setup.
            let (sender, receiver) = tokio::sync::oneshot::channel();
            let service = service.clone();
            app.run_on_main_thread(move || {
                let _ = sender.send(service.status());
            })?;
            receiver.await?
        }
        ClientMessage::ReloadSettings => service.reload_settings(app),
        ClientMessage::SetPaused { paused } => {
            service.set_paused(app, paused);
            ServerMessage::Ack
        }
        ClientMessage::RequestAccessibility => {
            let granted = native::request_permission();
            if !granted {
                let _ = std::process::Command::new("/usr/bin/open")
                    .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
                    .spawn();
            }
            ServerMessage::Accessibility { granted }
        }
        ClientMessage::Shutdown => {
            write_frame(writer, &ServerMessage::Ack).await?;
            app.exit(0);
            return Ok(ControlFlow::Break(()));
        }
    };
    write_frame(writer, &response).await?;
    Ok(ControlFlow::Continue(()))
}
