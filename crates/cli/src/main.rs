mod bridge;
mod config;
mod cursor;
mod helper;
mod setup;

use anyhow::{bail, Context, Result};
use figfig_core::{runtime, ClientMessage, CompletionRequest, Engine, ServerMessage};
use helper::{ensure_running, exchange, require_ack};
use std::path::PathBuf;
use tokio::net::UnixStream;

const HELP: &str = "figfig — local terminal autocomplete

  figfig setup          Install zsh integration (backs up .zshrc)
  figfig setup terminal Install and enable integrated-terminal cursor tracking
  figfig setup terminal --remove
                        Disable and remove shared cursor tracking
  figfig setup zed      Compatibility alias for setup terminal
  figfig init zsh       Print zsh integration
  figfig config         Show configuration
  figfig config get KEY
  figfig config set KEY VALUE
                        Keys: popup, history, max-candidates
  figfig config path    Show configuration file path
  figfig pause          Pause suggestions temporarily
  figfig resume         Resume suggestions
  figfig permissions    Open the required macOS permission settings
  figfig doctor         Check helper connection and permissions
  figfig start          Ensure the background popup helper is running
  figfig stop           Stop the background helper
  figfig complete TEXT [DIR]  Inspect local completions

After setup, the helper starts automatically when you use zsh.
No settings window or menu bar application is required.";

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("figfig: {error:#}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--version" | "-V") => println!("figfig {}", env!("CARGO_PKG_VERSION")),
        Some("init") if args.get(1).map(String::as_str) == Some("zsh") => {
            let exe = std::env::current_exe()?;
            println!(
                "typeset -g _FIGFIG_BIN={}",
                figfig_core::parser::shell_quote(&exe.to_string_lossy())
            );
            print!("{}", include_str!("../../../shell/figfig.zsh"));
        }
        Some("setup") if args.len() == 1 => setup::run()?,
        Some("setup")
            if matches!(args.get(1).map(String::as_str), Some("terminal" | "zed"))
                && args.len() == 2 =>
        {
            cursor::setup(false).await?
        }
        Some("setup")
            if matches!(args.get(1).map(String::as_str), Some("terminal" | "zed"))
                && args.get(2).map(String::as_str) == Some("--remove")
                && args.len() == 3 =>
        {
            cursor::setup(true).await?
        }
        Some("start") => {
            ensure_running().await?;
            println!("figfig popup helper is running in the background.");
        }
        Some("stop") => require_ack(exchange(ClientMessage::Shutdown).await?)?,
        Some("doctor") => doctor().await?,
        Some("config" | "settings") => configure(&args[1..]).await?,
        Some("pause" | "resume") => {
            ensure_running().await?;
            let paused = args[0] == "pause";
            require_ack(exchange(ClientMessage::SetPaused { paused }).await?)?;
            println!(
                "{}",
                if paused {
                    "Suggestions paused."
                } else {
                    "Suggestions resumed."
                }
            );
        }
        Some("permissions") => permissions().await?,
        Some("bridge") => {
            let session_id = args.get(1).context("bridge requires a session ID")?;
            bridge::run(session_id.clone()).await?;
        }
        Some("complete") => complete(&args[1..])?,
        Some("--help" | "-h") | None => println!("{HELP}"),
        _ => bail!("Unknown command. Run figfig --help."),
    }
    Ok(())
}

async fn configure(args: &[String]) -> Result<()> {
    if let config::ConfigResult::Changed = config::run(args)? {
        // Persist preferences without starting the helper just to edit a file.
        if UnixStream::connect(runtime::socket_path()?).await.is_ok() {
            require_ack(exchange(ClientMessage::ReloadSettings).await?)?;
        }
        println!("Configuration saved.");
    }
    Ok(())
}

async fn permissions() -> Result<()> {
    ensure_running().await?;
    match exchange(ClientMessage::RequestAccessibility).await? {
        ServerMessage::Accessibility { granted: true } => {
            println!("Accessibility permission is already granted.");
        }
        ServerMessage::Accessibility { granted: false } => {
            println!("Enable figfig in the macOS privacy settings that just opened.");
            println!("Then run `figfig doctor` to verify. The setting is called Accessibility or Device Control and Data Access, depending on macOS.");
        }
        ServerMessage::Error { message } => bail!("{message}"),
        _ => bail!("Unexpected permission response"),
    }
    Ok(())
}

fn complete(args: &[String]) -> Result<()> {
    let buffer = args
        .first()
        .context("Usage: figfig complete 'git ch' [directory]")?;
    let cwd = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or(std::env::current_dir()?);
    let request = CompletionRequest {
        session_id: "cli".into(),
        revision: 1,
        buffer: buffer.clone(),
        cursor: buffer.chars().count(),
        cwd: cwd.to_string_lossy().into(),
        terminal_pid: 0,
        terminal_size: None,
    };
    let response = Engine::new().complete(&request, 12);
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

async fn doctor() -> Result<()> {
    println!(
        "figfig {}\nRuntime: {}\nSettings: {}",
        env!("CARGO_PKG_VERSION"),
        runtime::runtime_dir()?.display(),
        runtime::settings_path().display()
    );
    let _ = runtime::load_settings()?;
    match exchange(ClientMessage::Status).await? {
        ServerMessage::Status {
            version,
            paused,
            accessibility,
            terminal,
            error,
        } => {
            println!("Protocol: {version}\nPaused: {paused}\nAccessibility: {accessibility}\nFocused terminal: {}", terminal.as_deref().unwrap_or("none"));
            if let Some(error) = error {
                println!("Position tracking: {error}");
            }
            if terminal
                .as_deref()
                .and_then(figfig_core::terminal::host_for_name)
                .is_some_and(|host| host.family == "jetbrains")
            {
                println!("Cursor positioning: figfig Terminal plugin (JetBrains). See the installation guide to install the plugin.");
            } else if terminal
                .as_deref()
                .and_then(figfig_core::terminal::host_for_name)
                .is_some_and(|host| host.family == "electron")
            {
                println!("Cursor positioning: Accessibility; optional InputMethodKit fallback (`figfig setup terminal`).");
                if !accessibility {
                    bail!("Run figfig permissions to enable integrated-terminal focus detection.");
                }
            } else if terminal
                .as_deref()
                .is_some_and(figfig_core::terminal::uses_cursor_tracker)
            {
                println!("Cursor positioning: InputMethodKit (requires `figfig setup terminal`).");
            } else if !accessibility {
                bail!("Run figfig permissions to enable terminal cursor positioning.");
            }
        }
        ServerMessage::Error { message } => bail!("{message}"),
        _ => bail!("Unexpected application response"),
    }
    Ok(())
}
