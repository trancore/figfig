use super::{terminal_host, trusted, CFArrayGetCount, CFArrayGetValueAtIndex, CFEqual, Owned, Ref};
use core_foundation::{base::TCFType, dictionary::CFDictionary, string::CFString, url::CFURL};
use objc2::MainThreadMarker;
use std::{path::PathBuf, ptr};

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFBooleanGetValue(value: Ref) -> bool;
    fn CFNotificationCenterGetDistributedCenter() -> Ref;
    fn CFNotificationCenterPostNotification(
        center: Ref,
        name: Ref,
        object: Ref,
        user_info: Ref,
        deliver_immediately: bool,
    );
}

#[link(name = "Carbon", kind = "framework")]
unsafe extern "C" {
    fn TISRegisterInputSource(url: Ref) -> i32;
    fn TISCreateInputSourceList(properties: Ref, include_all: bool) -> Ref;
    fn TISGetInputSourceProperty(source: Ref, key: Ref) -> Ref;
    fn TISEnableInputSource(source: Ref) -> i32;
    fn TISSelectInputSource(source: Ref) -> i32;
    fn TISDeselectInputSource(source: Ref) -> i32;
    fn TISDisableInputSource(source: Ref) -> i32;
    static kTISPropertyBundleID: Ref;
    static kTISPropertyInputSourceCategory: Ref;
    static kTISCategoryPaletteInputSource: Ref;
    static kTISPropertyInputSourceIsEnabled: Ref;
    static kTISPropertyInputSourceIsSelected: Ref;
}

enum TrackerEndpoint {
    JetBrains(PathBuf),
    InputMethod,
}

fn tracker_endpoint(pid: i32) -> Result<TrackerEndpoint, String> {
    let host = terminal_host(pid).ok_or("Focus a supported integrated terminal")?;
    if host.family == "jetbrains" {
        use std::os::unix::fs::{FileTypeExt, MetadataExt};
        let runtime = figfig_core::runtime::runtime_dir().map_err(|e| e.to_string())?;
        let endpoint = runtime.join(format!("jb-{pid}.sock"));
        let metadata = std::fs::symlink_metadata(&endpoint)
            .map_err(|_| figfig_core::terminal::tracker_hint(&host.name))?;
        if !metadata.file_type().is_socket()
            || metadata.mode() & 0o077 != 0
            || metadata.uid() != std::fs::metadata(runtime).map_err(|e| e.to_string())?.uid()
        {
            return Err("Insecure JetBrains cursor adapter socket".into());
        }
        return Ok(TrackerEndpoint::JetBrains(endpoint));
    }
    if host.family == "electron" && !trusted() {
        return Err(
            "Run `figfig permissions` to enable integrated-terminal focus detection.".into(),
        );
    }
    let sources = cursor_sources()?;
    let mut registered = false;
    let mut enabled = false;
    let mut selected = false;
    for index in 0..unsafe { CFArrayGetCount(sources.0) } {
        let source = unsafe { CFArrayGetValueAtIndex(sources.0, index) };
        if !is_palette(source) {
            continue;
        }
        registered = true;
        let source_enabled = source_flag(source, unsafe { kTISPropertyInputSourceIsEnabled });
        enabled |= source_enabled;
        selected |=
            source_enabled && source_flag(source, unsafe { kTISPropertyInputSourceIsSelected });
    }
    crate::cursor::TrackerState::from_flags(registered, enabled, selected).require_active()?;
    Ok(TrackerEndpoint::InputMethod)
}

pub fn tracker_ready(pid: i32) -> Result<(), String> {
    tracker_endpoint(pid).map(|_| ())
}

pub fn request_tracker_caret(request_id: &str, terminal_pid: i32) -> Result<(), String> {
    if let TrackerEndpoint::JetBrains(endpoint) = tracker_endpoint(terminal_pid)? {
        use tokio::io::AsyncWriteExt;
        let mut message = serde_json::to_vec(&serde_json::json!({
            "requestId": request_id, "terminalPid": terminal_pid,
        }))
        .map_err(|e| e.to_string())?;
        message.push(b'\n');
        // Never block the AppKit thread on an IDE's socket/backlog. Failure
        // leaves the query unanswered and produces the usual 500ms hint.
        tauri::async_runtime::spawn(async move {
            let _ = tokio::time::timeout(std::time::Duration::from_millis(50), async {
                let mut stream = tokio::net::UnixStream::connect(endpoint).await?;
                stream.write_all(&message).await
            })
            .await;
        });
        return Ok(());
    }
    let path = figfig_core::runtime::socket_path().map_err(|e| e.to_string())?;
    let info = CFDictionary::from_CFType_pairs(&[
        (
            CFString::new("socketPath"),
            CFString::new(&path.to_string_lossy()),
        ),
        (CFString::new("requestId"), CFString::new(request_id)),
        (
            CFString::new("terminalPid"),
            CFString::new(&terminal_pid.to_string()),
        ),
    ]);
    notify(
        "dev.figfig.cursor.request",
        info.as_concrete_TypeRef() as Ref,
    );
    Ok(())
}

fn notify(name: &str, info: Ref) {
    let name = CFString::new(name);
    unsafe {
        CFNotificationCenterPostNotification(
            CFNotificationCenterGetDistributedCenter(),
            name.as_concrete_TypeRef() as Ref,
            ptr::null(),
            info,
            true,
        );
    }
}

fn cursor_sources() -> Result<Owned, String> {
    let key = unsafe { CFString::wrap_under_get_rule(kTISPropertyBundleID as _) };
    let filter = CFDictionary::from_CFType_pairs(&[(key, CFString::new("dev.figfig.cursor"))]);
    Owned::new(unsafe { TISCreateInputSourceList(filter.as_concrete_TypeRef() as Ref, true) })
}

fn is_palette(source: Ref) -> bool {
    let category = unsafe { TISGetInputSourceProperty(source, kTISPropertyInputSourceCategory) };
    !category.is_null() && unsafe { CFEqual(category, kTISCategoryPaletteInputSource) }
}

fn source_flag(source: Ref, property: Ref) -> bool {
    let state = unsafe { TISGetInputSourceProperty(source, property) };
    !state.is_null() && unsafe { CFBooleanGetValue(state) }
}

pub fn setup_cursor(bundle_path: &str, enabled: bool) -> Result<(), String> {
    let _mtm = MainThreadMarker::new().ok_or("Input method setup requires the main thread")?;
    if bundle_path.len() > 4096
        || !bundle_path.ends_with("/Library/Input Methods/figfig Cursor.app")
    {
        return Err("Invalid cursor tracker bundle location".into());
    }
    let url = CFURL::from_path(bundle_path, true).ok_or("Invalid cursor tracker URL")?;
    if enabled {
        let result = unsafe { TISRegisterInputSource(url.as_concrete_TypeRef() as Ref) };
        if result != 0 {
            return Err(format!(
                "Could not register cursor tracking (OSStatus {result})"
            ));
        }
    }
    let sources = cursor_sources()?;
    let count = unsafe { CFArrayGetCount(sources.0) };
    if count == 0 && enabled {
        return Err("macOS has not registered cursor tracking yet. Log out and back in, then run `figfig setup terminal` again.".into());
    }
    for index in 0..count {
        let source = unsafe { CFArrayGetValueAtIndex(sources.0, index) };
        if !is_palette(source) {
            return Err("Cursor tracking must be a palette, not a keyboard input source".into());
        }
        if enabled {
            let result = unsafe { TISEnableInputSource(source) };
            if result != 0 {
                return Err(format!(
                    "Could not enable cursor tracking (OSStatus {result})"
                ));
            }
            if !source_flag(source, unsafe { kTISPropertyInputSourceIsEnabled }) {
                return crate::cursor::TrackerState::Disabled.require_active();
            }
            let result = unsafe { TISSelectInputSource(source) };
            if result != 0 || !source_flag(source, unsafe { kTISPropertyInputSourceIsSelected }) {
                return Err(format!("macOS could not activate cursor tracking (OSStatus {result}). Log out and back in, then retry `figfig setup terminal`."));
            }
        } else {
            unsafe {
                TISDeselectInputSource(source);
            }
            let result = unsafe { TISDisableInputSource(source) };
            if result != 0 {
                return Err(format!(
                    "Could not disable cursor tracking (OSStatus {result})"
                ));
            }
        }
    }
    if !enabled {
        notify("dev.figfig.cursor.stop", ptr::null());
    }
    Ok(())
}
