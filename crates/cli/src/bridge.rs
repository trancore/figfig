use crate::helper::ensure_running;
use anyhow::{bail, Result};
use figfig_core::{
    protocol::TerminalSize,
    runtime,
    transport::{write_frame, FrameReader},
    ClientMessage, CompletionRequest, CompletionResponse, ServerMessage, MAX_FRAME,
};
use std::{
    fs::File,
    io::{BufRead, Write},
    process::Command,
};
use tokio::{io::BufReader, net::UnixStream};

pub async fn run(session_id: String) -> Result<()> {
    let path = runtime::socket_path()?;
    ensure_running().await?;
    let stream = UnixStream::connect(&path).await?;
    let terminal_pid = find_terminal_pid();
    let zed_tty = if std::env::var_os("TERM_PROGRAM").is_some_and(|host| host == "zed") {
        File::open("/dev/tty").ok()
    } else {
        None
    };
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<[String; 6]>(32);
    // Input is fixed-field NUL framing. No shell code is evaluated in either direction.
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut reader = stdin.lock();
        while let Ok(Some(fields)) = read_shell_fields(&mut reader) {
            if sender.blocking_send(fields).is_err() {
                break;
            }
        }
    });
    let (read, mut write) = stream.into_split();
    let mut reader = FrameReader::new(BufReader::new(read));
    let mut latest_revision = 0;
    let mut dismissed = false;
    loop {
        tokio::select! {
            fields = receiver.recv() => {
                let Some(fields) = fields else { break; };
                let [operation, revision, cursor, cwd, buffer, detail] = fields;
                let revision = revision.parse::<u64>().unwrap_or(0);
                let message = match operation.as_str() {
                    "complete" => {
                        latest_revision = revision;
                        dismissed = false;
                        ClientMessage::Complete {
                            request: CompletionRequest {
                                session_id: session_id.clone(),
                                revision,
                                cursor: cursor.parse().unwrap_or(0),
                                cwd,
                                buffer,
                                terminal_pid,
                                terminal_size: zed_tty.as_ref().and_then(|tty| terminal_size(tty, &detail)),
                            },
                        }
                    }
                    "select" => ClientMessage::Select {
                        session_id: session_id.clone(),
                        revision,
                        selected: detail.parse().unwrap_or(0),
                    },
                    "hide" => {
                        latest_revision = revision;
                        dismissed = true;
                        ClientMessage::Hide { session_id: session_id.clone(), revision }
                    }
                    _ => continue,
                };
                write_frame(&mut write, &message).await?;
            }
            frame = reader.next_frame() => {
                let Some(frame) = frame? else { break; };
                match serde_json::from_slice::<ServerMessage>(&frame)? {
                    ServerMessage::Completion { response }
                        if response.session_id == session_id
                            && response.revision == latest_revision
                            && !dismissed => write_shell_response(&response)?,
                    ServerMessage::Preferences { settings, paused } => {
                        println!("P {}", u8::from(settings.history_enabled && !paused));
                        std::io::stdout().flush()?;
                    }
                    ServerMessage::Error { message } => eprintln!("figfig: {message}"),
                    _ => (),
                }
            }
        }
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn terminal_size(tty: &File, cursor: &str) -> Option<TerminalSize> {
    use figfig_core::protocol::TerminalCell;
    use std::os::fd::AsRawFd;

    let mut size: libc::winsize = unsafe { std::mem::zeroed() };
    if unsafe { libc::ioctl(tty.as_raw_fd(), libc::TIOCGWINSZ, &mut size) } != 0
        || size.ws_xpixel == 0
        || size.ws_ypixel == 0
    {
        return None;
    }
    Some(TerminalSize {
        width: size.ws_xpixel,
        height: size.ws_ypixel,
        cursor: cursor.split_once(';').and_then(|(row, column)| {
            Some(TerminalCell {
                row: row.parse().ok()?,
                column: column.parse().ok()?,
                rows: size.ws_row,
                columns: size.ws_col,
            })
        }),
    })
}

#[cfg(not(target_os = "macos"))]
fn terminal_size(_: &File, _: &str) -> Option<TerminalSize> {
    None
}

fn write_shell_response(response: &CompletionResponse) -> Result<()> {
    let mut words = vec![
        "R".into(),
        response.revision.to_string(),
        response.replace_start.to_string(),
        response.replace_end.to_string(),
        if response.visible { "1" } else { "0" }.into(),
        response.candidates.len().to_string(),
    ];
    for candidate in &response.candidates {
        // quote twice: the first represents shell insertion, the second is transport.
        words.push(figfig_core::parser::shell_quote(&candidate.insert_text));
        words.push(if candidate.append_space { "1" } else { "0" }.into());
    }
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    writeln!(out, "{}", words.join(" "))?;
    out.flush()?;
    Ok(())
}

fn find_terminal_pid() -> i32 {
    if let Ok(value) = std::env::var("FIGFIG_TERMINAL_PID") {
        return value.parse().unwrap_or(0);
    }
    let mut pid = unsafe { getppid() };
    // IDEs add PTY hosts, launchers, and shell-integration wrappers.
    for _ in 0..64 {
        let Ok(output) = Command::new("/bin/ps")
            .args(["-p", &pid.to_string(), "-o", "ppid=", "-o", "comm="])
            .output()
        else {
            break;
        };
        let text = String::from_utf8_lossy(&output.stdout);
        let text = text.trim();
        let Some((parent, command)) = text.split_once(char::is_whitespace) else {
            break;
        };
        if figfig_core::terminal::terminal_name(command.trim()).is_some() {
            return pid;
        }
        pid = parent.parse().unwrap_or(0);
        if pid <= 1 {
            break;
        }
    }
    0
}

unsafe extern "C" {
    fn getppid() -> i32;
}

fn read_shell_fields(reader: &mut impl BufRead) -> Result<Option<[String; 6]>> {
    let mut fields = std::array::from_fn(|_| String::new());
    let mut size = 0;
    for field in &mut fields {
        let mut bytes = Vec::new();
        loop {
            let available = reader.fill_buf()?;
            if available.is_empty() {
                return Ok(None);
            }
            let end = available.iter().position(|&byte| byte == 0);
            let count = end.map(|index| index + 1).unwrap_or(available.len());
            size += count;
            if size > MAX_FRAME {
                bail!("Shell frame too large");
            }
            bytes.extend_from_slice(&available[..count]);
            reader.consume(count);
            if end.is_some() {
                bytes.pop();
                break;
            }
        }
        *field = String::from_utf8(bytes)?;
    }
    Ok(Some(fields))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, Cursor};

    #[test]
    fn shell_fields_preserve_unicode_newlines_and_empty_values() {
        let bytes = b"complete\x007\x003\x00/tmp\x00git \xe6\x97\xa5\n\x00\x00hide\x008\x000\x00\x00\x00\x00";
        let mut reader = BufReader::with_capacity(2, Cursor::new(bytes));
        let fields = read_shell_fields(&mut reader).unwrap().unwrap();
        assert_eq!(fields, ["complete", "7", "3", "/tmp", "git 日\n", ""]);
        assert_eq!(read_shell_fields(&mut reader).unwrap().unwrap()[0], "hide");
        assert!(read_shell_fields(&mut reader).unwrap().is_none());
    }

    #[test]
    fn incomplete_or_invalid_shell_frames_are_not_forwarded() {
        assert!(read_shell_fields(&mut Cursor::new(b"complete\x001\x00"))
            .unwrap()
            .is_none());
        assert!(read_shell_fields(&mut Cursor::new(b"\xff\x00")).is_err());
        assert!(read_shell_fields(&mut Cursor::new(vec![b'x'; MAX_FRAME + 1])).is_err());
    }
}
