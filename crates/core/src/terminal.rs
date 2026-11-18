//! Shared host profiles. The native cursor helper embeds this same registry.
use serde::Deserialize;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CursorBackend {
    Accessibility,
    InputMethod,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalHost {
    pub name: String,
    pub bundle: String,
    pub executable: String,
    pub bundle_id: String,
    pub backend: CursorBackend,
    pub family: String,
}

pub fn hosts() -> &'static [TerminalHost] {
    static HOSTS: LazyLock<Vec<TerminalHost>> = LazyLock::new(|| {
        serde_json::from_str(include_str!("../../../assets/terminal-hosts.json"))
            .expect("valid bundled terminal host registry")
    });
    &HOSTS
}

pub fn host_for_executable(executable: &str) -> Option<&'static TerminalHost> {
    hosts().iter().find(|host| {
        executable.ends_with(&format!(
            "/{}/Contents/MacOS/{}",
            host.bundle, host.executable
        ))
    })
}

pub fn host_for_bundle(identifier: &str) -> Option<&'static TerminalHost> {
    hosts().iter().find(|host| host.bundle_id == identifier)
}

pub fn host_for_name(name: &str) -> Option<&'static TerminalHost> {
    hosts().iter().find(|host| host.name == name)
}

/// Identify the main executable, never an editor's CLI or PTY helper.
pub fn terminal_name(executable: &str) -> Option<&'static str> {
    host_for_executable(executable).map(|host| host.name.as_str())
}

pub fn uses_cursor_tracker(name: &str) -> bool {
    host_for_name(name).is_some_and(|host| host.backend == CursorBackend::InputMethod)
}

pub fn tracker_hint(name: &str) -> &'static str {
    if host_for_name(name).is_some_and(|host| host.family == "jetbrains") {
        "JetBrains cursor tracking is unavailable. Install the figfig Terminal plugin (see the installation guide), restart the IDE, and use a local zsh terminal."
    } else {
        "Integrated-terminal cursor tracking did not respond. Run `figfig doctor`. After a first installation, log out of macOS and back in, then retry `figfig setup terminal` and restart your editor. VSCode and Cursor also require `figfig permissions` for terminal focus detection."
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_identifies_every_main_process_in_any_install_directory() {
        for host in hosts() {
            for directory in [
                "/Applications",
                "/Users/test/Library/Application Support/JetBrains/Toolbox/apps/2026.3",
            ] {
                let path = format!(
                    "{directory}/{}/Contents/MacOS/{}",
                    host.bundle, host.executable
                );
                assert_eq!(terminal_name(&path), Some(host.name.as_str()));
                assert_eq!(host_for_bundle(&host.bundle_id).unwrap().name, host.name);
            }
            assert!(!host.family.is_empty() || host.backend == CursorBackend::Accessibility);
        }
    }

    #[test]
    fn does_not_mistake_cli_launchers_or_subprocesses_for_the_host() {
        for path in [
            "/Applications/Zed.app/Contents/MacOS/cli",
            "/Applications/Zed.app/Contents/MacOS/zed-helper",
            "/tmp/OtherZed.app/Contents/MacOS/zed",
            "/Applications/Visual Studio Code.app/Contents/Frameworks/Code Helper.app/Contents/MacOS/Code Helper",
            "/Applications/IntelliJ IDEA.app/Contents/lib/pty4j/darwin/pty4j-unix-spawn-helper",
            "/bin/zsh",
        ] {
            assert_eq!(terminal_name(path), None);
        }
        assert!(host_for_bundle("com.microsoft.VSCode.helper").is_none());
        assert!(!uses_cursor_tracker("Terminal.app"));
        assert!(uses_cursor_tracker("VSCode"));
        assert!(uses_cursor_tracker("IntelliJ IDEA"));
    }
}
