/// Identify the executable that owns a local terminal session.
pub fn terminal_name(executable: &str) -> Option<&'static str> {
    if executable.ends_with("/Terminal.app/Contents/MacOS/Terminal") {
        Some("Terminal.app")
    } else if executable.ends_with("/iTerm.app/Contents/MacOS/iTerm2")
        || executable.ends_with("/iTerm2.app/Contents/MacOS/iTerm2")
    {
        Some("iTerm2")
    } else if ["Zed.app", "Zed Preview.app", "Zed Nightly.app"]
        .iter()
        .any(|bundle| executable.ends_with(&format!("/{bundle}/Contents/MacOS/zed")))
    {
        Some("Zed")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_terminal_hosts_and_zed_channels() {
        for (path, name) in [
            (
                "/System/Applications/Utilities/Terminal.app/Contents/MacOS/Terminal",
                "Terminal.app",
            ),
            ("/Applications/iTerm.app/Contents/MacOS/iTerm2", "iTerm2"),
            ("/Applications/Zed.app/Contents/MacOS/zed", "Zed"),
            ("/Applications/Zed Preview.app/Contents/MacOS/zed", "Zed"),
            ("/Applications/Zed Nightly.app/Contents/MacOS/zed", "Zed"),
        ] {
            assert_eq!(terminal_name(path), Some(name));
        }
    }

    #[test]
    fn does_not_mistake_cli_launchers_or_subprocesses_for_the_host() {
        for path in [
            "/Applications/Zed.app/Contents/MacOS/cli",
            "/Applications/Zed.app/Contents/MacOS/zed-helper",
            "/tmp/OtherZed.app/Contents/MacOS/zed",
            "/bin/zsh",
        ] {
            assert_eq!(terminal_name(path), None);
        }
    }
}
