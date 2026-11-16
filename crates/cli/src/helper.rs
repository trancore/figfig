use anyhow::{bail, Context, Result};
use figfig_core::{
    runtime,
    transport::{write_frame, FrameReader},
    ClientMessage, ServerMessage,
};
use std::{
    os::unix::process::CommandExt,
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};
use tokio::{io::BufReader, net::UnixStream};

fn app_location() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("FIGFIG_APP") {
        return Ok(path.into());
    }
    let exe = std::env::current_exe()?.canonicalize()?;
    let bin = exe.parent().context("No executable directory")?;
    let candidates = [
        bin.join("../libexec/figfig.app"),
        bin.join("bundle/macos/figfig.app"),
        bin.join("figfig-desktop"),
    ];
    candidates
        .into_iter()
        .find(|p| p.exists())
        .context("Popup helper not found. Build with pnpm run build:release, or reinstall figfig.")
}

fn spawn_helper() -> Result<std::process::Child> {
    let path = app_location()?;
    let executable = if path.extension().is_some_and(|s| s == "app") {
        path.join("Contents/MacOS/figfig-desktop")
    } else {
        path
    };
    Command::new(executable)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .context("Could not start the popup helper")
}

pub async fn ensure_running() -> Result<()> {
    let path = runtime::socket_path()?;
    if UnixStream::connect(&path).await.is_ok() {
        return Ok(());
    }
    let mut child = spawn_helper()?;
    for _ in 0..100 {
        if UnixStream::connect(&path).await.is_ok() {
            return Ok(());
        }
        if let Some(status) = child.try_wait()? {
            // A simultaneous startup may have won the single-instance lock.
            if !status.success() {
                bail!("Popup helper exited with {status}");
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let _ = child.kill();
    let _ = child.wait();
    bail!("Popup helper did not start within 5 seconds")
}

pub fn require_ack(message: ServerMessage) -> Result<()> {
    match message {
        ServerMessage::Ack => Ok(()),
        ServerMessage::Error { message } => bail!("{message}"),
        _ => bail!("Unexpected helper response"),
    }
}

pub async fn exchange(message: ClientMessage) -> Result<ServerMessage> {
    tokio::time::timeout(Duration::from_secs(2), async {
        let mut stream = UnixStream::connect(runtime::socket_path()?)
            .await
            .context("Popup helper is not running. It starts automatically in zsh; use figfig start to check it manually.")?;
        write_frame(&mut stream, &message).await?;
        let mut reader = FrameReader::new(BufReader::new(stream));
        loop {
            let line = reader
                .next_frame()
                .await?
                .context("Popup helper closed the connection")?;
            let message = serde_json::from_slice(&line)?;
            if !matches!(message, ServerMessage::Preferences { .. }) {
                return Ok(message);
            }
        }
    })
    .await
    .context("Popup helper did not respond within 2 seconds")?
}
