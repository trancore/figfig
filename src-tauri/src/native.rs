//! macOS integration reads caret geometry, never terminal contents or keystrokes.
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
pub struct Caret {
    pub x: f64,
    pub y: f64,
    pub height: f64,
    pub pid: i32,
    pub focus: usize,
    pub tracker: bool,
}

fn convert_tracker_caret(
    position: &figfig_core::CursorPosition,
    screen_height: f64,
) -> Result<Caret, String> {
    if position.terminal_pid <= 0
        || position.focus == 0
        || !position.x.is_finite()
        || !position.y.is_finite()
        || !(1.0..=200.0).contains(&position.height)
        || !screen_height.is_finite()
    {
        return Err("The terminal did not expose a valid insertion point".into());
    }
    let y = match position.coordinate_system {
        figfig_core::protocol::CoordinateSystem::AppKit => {
            screen_height - position.y - position.height
        }
        figfig_core::protocol::CoordinateSystem::Accessibility => position.y,
    };
    if !y.is_finite() {
        return Err("The terminal returned invalid cursor coordinates".into());
    }
    Ok(Caret {
        x: position.x,
        y,
        height: position.height,
        pid: position.terminal_pid,
        focus: position
            .focus
            .try_into()
            .map_err(|_| "Invalid cursor focus identifier")?,
        tracker: true,
    })
}

#[cfg(target_os = "macos")]
mod mac {
    use super::Caret;
    use core_foundation::{
        base::TCFType, boolean::CFBoolean, dictionary::CFDictionary, string::CFString, url::CFURL,
    };
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSScreen, NSWindow, NSWorkspace};
    use objc2_foundation::NSPoint;
    use std::{ffi::c_void, ptr};

    type Ref = *const c_void;
    #[repr(C)]
    #[derive(Default, Copy, Clone)]
    struct Range {
        location: isize,
        length: isize,
    }
    #[repr(C)]
    #[derive(Default, Copy, Clone)]
    struct Point {
        x: f64,
        y: f64,
    }
    #[repr(C)]
    #[derive(Default, Copy, Clone)]
    struct Size {
        width: f64,
        height: f64,
    }
    #[repr(C)]
    #[derive(Default, Copy, Clone)]
    struct Rect {
        origin: Point,
        size: Size,
    }

    #[link(name = "ApplicationServices", kind = "framework")]
    unsafe extern "C" {
        fn AXIsProcessTrusted() -> bool;
        fn AXIsProcessTrustedWithOptions(options: Ref) -> bool;
        static kAXTrustedCheckOptionPrompt: Ref;
        fn AXUIElementCreateSystemWide() -> Ref;
        fn AXUIElementCopyAttributeValue(element: Ref, attribute: Ref, value: *mut Ref) -> i32;
        fn AXUIElementCopyParameterizedAttributeValue(
            element: Ref,
            attribute: Ref,
            parameter: Ref,
            value: *mut Ref,
        ) -> i32;
        fn AXUIElementGetPid(element: Ref, pid: *mut i32) -> i32;
        fn AXUIElementSetMessagingTimeout(element: Ref, timeout: f32) -> i32;
        fn AXValueGetValue(value: Ref, kind: u32, output: *mut c_void) -> bool;
        fn AXValueCreate(kind: u32, value: *const c_void) -> Ref;
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFRelease(value: Ref);
        fn CFHash(value: Ref) -> usize;
        fn CFEqual(a: Ref, b: Ref) -> bool;
        fn CFArrayGetCount(array: Ref) -> isize;
        fn CFArrayGetValueAtIndex(array: Ref, index: isize) -> Ref;
        fn CFBooleanGetValue(value: Ref) -> bool;
        fn CFGetTypeID(value: Ref) -> usize;
        fn CFArrayGetTypeID() -> usize;
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
    unsafe extern "C" {
        fn proc_pidpath(pid: i32, buffer: *mut c_void, buffer_size: u32) -> i32;
    }

    struct Owned(Ref);
    impl Drop for Owned {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    CFRelease(self.0);
                }
            }
        }
    }
    impl Owned {
        fn new(value: Ref) -> Result<Self, String> {
            if value.is_null() {
                Err("Accessibility returned no element".into())
            } else {
                Ok(Self(value))
            }
        }
        fn attr(&self, name: &str) -> Result<Self, String> {
            let attribute = CFString::new(name);
            let mut output = ptr::null();
            let code = unsafe {
                AXUIElementCopyAttributeValue(
                    self.0,
                    attribute.as_concrete_TypeRef() as Ref,
                    &mut output,
                )
            };
            if code != 0 {
                return Err(format!("Terminal does not expose {name} (AX error {code})"));
            }
            Self::new(output)
        }
    }

    pub fn trusted() -> bool {
        unsafe { AXIsProcessTrusted() }
    }
    pub fn request_permission() -> bool {
        unsafe {
            let key = CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt as _);
            let options = CFDictionary::from_CFType_pairs(&[(key, CFBoolean::true_value())]);
            AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef() as Ref)
        }
    }

    pub fn terminal_name(pid: i32) -> Option<String> {
        let mut buffer = [0u8; 4096];
        let length =
            unsafe { proc_pidpath(pid, buffer.as_mut_ptr() as *mut c_void, buffer.len() as u32) };
        if length <= 0 {
            return None;
        }
        let name = String::from_utf8_lossy(&buffer[..length as usize]);
        figfig_core::terminal::terminal_name(name.trim_end_matches('\0')).map(str::to_owned)
    }

    pub fn uses_cursor_tracker(pid: i32) -> bool {
        terminal_name(pid)
            .as_deref()
            .is_some_and(figfig_core::terminal::uses_cursor_tracker)
    }

    pub fn is_electron(pid: i32) -> bool {
        terminal_name(pid)
            .as_deref()
            .and_then(figfig_core::terminal::host_for_name)
            .is_some_and(|host| host.family == "electron")
    }

    pub fn frontmost_pid() -> i32 {
        NSWorkspace::sharedWorkspace()
            .frontmostApplication()
            .map_or(0, |app| app.processIdentifier())
    }

    pub fn tracker_ready(pid: i32) -> Result<(), String> {
        let terminal = terminal_name(pid).ok_or("Focus a supported integrated terminal")?;
        let host = figfig_core::terminal::host_for_name(&terminal).unwrap();
        if host.family == "jetbrains" {
            use std::os::unix::fs::{FileTypeExt, MetadataExt};
            let runtime = figfig_core::runtime::runtime_dir().map_err(|e| e.to_string())?;
            let endpoint = runtime.join(format!("jb-{pid}.sock"));
            let metadata = std::fs::symlink_metadata(endpoint)
                .map_err(|_| figfig_core::terminal::tracker_hint(&terminal))?;
            if !metadata.file_type().is_socket()
                || metadata.mode() & 0o077 != 0
                || metadata.uid() != std::fs::metadata(runtime).map_err(|e| e.to_string())?.uid()
            {
                return Err("Insecure JetBrains cursor adapter socket".into());
            }
            return Ok(());
        }
        if host.family == "electron" && !trusted() {
            return Err(
                "Run `figfig permissions` to enable integrated-terminal focus detection.".into(),
            );
        }
        let key = unsafe { CFString::wrap_under_get_rule(kTISPropertyBundleID as _) };
        let filter = CFDictionary::from_CFType_pairs(&[(key, CFString::new("dev.figfig.cursor"))]);
        let sources = Owned::new(unsafe {
            TISCreateInputSourceList(filter.as_concrete_TypeRef() as Ref, true)
        })?;
        let mut registered = false;
        let mut enabled = false;
        let mut selected = false;
        for index in 0..unsafe { CFArrayGetCount(sources.0) } {
            let source = unsafe { CFArrayGetValueAtIndex(sources.0, index) };
            let category =
                unsafe { TISGetInputSourceProperty(source, kTISPropertyInputSourceCategory) };
            if category.is_null() || !unsafe { CFEqual(category, kTISCategoryPaletteInputSource) } {
                continue;
            }
            registered = true;
            let state =
                unsafe { TISGetInputSourceProperty(source, kTISPropertyInputSourceIsEnabled) };
            let source_enabled = !state.is_null() && unsafe { CFBooleanGetValue(state) };
            enabled |= source_enabled;
            let state =
                unsafe { TISGetInputSourceProperty(source, kTISPropertyInputSourceIsSelected) };
            selected |= source_enabled && !state.is_null() && unsafe { CFBooleanGetValue(state) };
        }
        crate::cursor::TrackerState::from_flags(registered, enabled, selected).require_active()
    }

    pub fn request_tracker_caret(request_id: &str, terminal_pid: i32) -> Result<(), String> {
        tracker_ready(terminal_pid)?;
        let path = figfig_core::runtime::socket_path().map_err(|e| e.to_string())?;
        if terminal_name(terminal_pid)
            .as_deref()
            .and_then(figfig_core::terminal::host_for_name)
            .is_some_and(|host| host.family == "jetbrains")
        {
            use tokio::io::AsyncWriteExt;
            let endpoint = path
                .parent()
                .unwrap()
                .join(format!("jb-{terminal_pid}.sock"));
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
        let name = CFString::new("dev.figfig.cursor.request");
        let keys = [
            CFString::new("socketPath"),
            CFString::new("requestId"),
            CFString::new("terminalPid"),
        ];
        let values = [
            CFString::new(&path.to_string_lossy()),
            CFString::new(request_id),
            CFString::new(&terminal_pid.to_string()),
        ];
        let info = CFDictionary::from_CFType_pairs(&[
            (keys[0].clone(), values[0].clone()),
            (keys[1].clone(), values[1].clone()),
            (keys[2].clone(), values[2].clone()),
        ]);
        unsafe {
            CFNotificationCenterPostNotification(
                CFNotificationCenterGetDistributedCenter(),
                name.as_concrete_TypeRef() as Ref,
                ptr::null(),
                info.as_concrete_TypeRef() as Ref,
                true,
            );
        }
        Ok(())
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
        let key = unsafe { CFString::wrap_under_get_rule(kTISPropertyBundleID as _) };
        let filter = CFDictionary::from_CFType_pairs(&[(key, CFString::new("dev.figfig.cursor"))]);
        let sources = Owned::new(unsafe {
            TISCreateInputSourceList(filter.as_concrete_TypeRef() as Ref, true)
        })?;
        let count = unsafe { CFArrayGetCount(sources.0) };
        if count == 0 && enabled {
            return Err("macOS has not registered cursor tracking yet. Log out and back in, then run `figfig setup terminal` again.".into());
        }
        for index in 0..count {
            let source = unsafe { CFArrayGetValueAtIndex(sources.0, index) };
            let category =
                unsafe { TISGetInputSourceProperty(source, kTISPropertyInputSourceCategory) };
            if category.is_null() || !unsafe { CFEqual(category, kTISCategoryPaletteInputSource) } {
                return Err(
                    "Cursor tracking must be a palette, not a keyboard input source".into(),
                );
            }
            if enabled {
                let result = unsafe { TISEnableInputSource(source) };
                if result != 0 {
                    return Err(format!(
                        "Could not enable cursor tracking (OSStatus {result})"
                    ));
                }
                let state =
                    unsafe { TISGetInputSourceProperty(source, kTISPropertyInputSourceIsEnabled) };
                if state.is_null() || !unsafe { CFBooleanGetValue(state) } {
                    return crate::cursor::TrackerState::Disabled.require_active();
                }
                let result = unsafe { TISSelectInputSource(source) };
                let state =
                    unsafe { TISGetInputSourceProperty(source, kTISPropertyInputSourceIsSelected) };
                if result != 0 || state.is_null() || !unsafe { CFBooleanGetValue(state) } {
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
            let name = CFString::new("dev.figfig.cursor.stop");
            unsafe {
                CFNotificationCenterPostNotification(
                    CFNotificationCenterGetDistributedCenter(),
                    name.as_concrete_TypeRef() as Ref,
                    ptr::null(),
                    ptr::null(),
                    true,
                );
            }
        }
        Ok(())
    }

    pub fn tracker_caret(position: &figfig_core::CursorPosition) -> Result<Caret, String> {
        let mtm = MainThreadMarker::new().ok_or("Cursor positioning requires the main thread")?;
        let height = NSScreen::screens(mtm)
            .firstObject()
            .ok_or("No display")?
            .frame()
            .size
            .height;
        super::convert_tracker_caret(position, height)
    }

    pub fn caret(expected_pid: i32) -> Result<Caret, String> {
        if !trusted() {
            return Err("Accessibility permission is required".into());
        }
        let system = Owned::new(unsafe { AXUIElementCreateSystemWide() })?;
        unsafe {
            AXUIElementSetMessagingTimeout(system.0, 0.05);
        }
        let app = system.attr("AXFocusedApplication")?;
        let mut pid = 0;
        if unsafe { AXUIElementGetPid(app.0, &mut pid) } != 0 {
            return Err("No focused application".into());
        }
        if expected_pid > 0 && pid != expected_pid {
            return Err("The shell's terminal is not focused".into());
        }
        if terminal_name(pid).is_none() {
            return Err("Focus a supported terminal or editor integrated terminal".into());
        }
        let element = app.attr("AXFocusedUIElement")?;
        if is_electron(pid) {
            let classes = element.attr("AXDOMClassList")?;
            let terminal_class = CFString::new("xterm-helper-textarea");
            if unsafe { CFGetTypeID(classes.0) != CFArrayGetTypeID() }
                || !(0..unsafe { CFArrayGetCount(classes.0) }).any(|index| unsafe {
                    CFEqual(
                        CFArrayGetValueAtIndex(classes.0, index),
                        terminal_class.as_concrete_TypeRef() as Ref,
                    )
                })
            {
                return Err(
                    "Focus the integrated terminal input, not the editor or search field".into(),
                );
            }
        }
        let range_value = element.attr("AXSelectedTextRange")?;
        let mut range = Range::default();
        if !unsafe { AXValueGetValue(range_value.0, 4, &mut range as *mut _ as *mut c_void) }
            || range.length != 0
        {
            return Err("No insertion point, or terminal text is selected".into());
        }
        let attribute = CFString::new("AXBoundsForRange");
        let mut rect = Rect::default();
        let get_rect = |range_value: Ref, rect: &mut Rect| -> bool {
            let mut bounds = ptr::null();
            let error = unsafe {
                AXUIElementCopyParameterizedAttributeValue(
                    element.0,
                    attribute.as_concrete_TypeRef() as Ref,
                    range_value,
                    &mut bounds,
                )
            };
            if error != 0 || bounds.is_null() {
                return false;
            }
            let bounds = Owned(bounds);
            unsafe { AXValueGetValue(bounds.0, 3, rect as *mut _ as *mut c_void) }
        };
        if !get_rect(range_value.0, &mut rect) || rect.size.height <= 0.0 {
            if is_electron(pid) {
                // xterm positions its hidden IME textarea at the cursor. Its
                // element bounds remain useful when AXBoundsForRange is absent.
                let position = element.attr("AXPosition")?;
                let size = element.attr("AXSize")?;
                if !unsafe {
                    AXValueGetValue(position.0, 1, &mut rect.origin as *mut _ as *mut c_void)
                } || !unsafe {
                    AXValueGetValue(size.0, 2, &mut rect.size as *mut _ as *mut c_void)
                } {
                    return Err("Terminal does not expose caret bounds".into());
                }
            } else if range.location <= 0 {
                return Err("Terminal does not expose caret bounds".into());
            } else {
                range.location -= 1;
                range.length = 1;
                let previous = Owned::new(unsafe { AXValueCreate(4, &range as *const _ as Ref) })?;
                if !get_rect(previous.0, &mut rect) {
                    return Err("Terminal does not expose caret bounds".into());
                }
                rect.origin.x += rect.size.width;
            }
        }
        if !rect.origin.x.is_finite()
            || !rect.origin.y.is_finite()
            || !(1.0..=200.0).contains(&rect.size.height)
        {
            return Err("Terminal returned invalid caret bounds".into());
        }
        Ok(Caret {
            x: rect.origin.x,
            y: rect.origin.y,
            height: rect.size.height,
            pid,
            focus: unsafe { CFHash(element.0) },
            tracker: false,
        })
    }

    /// AX uses screen points with a top-left origin. AppKit uses a bottom-left origin.
    pub fn place(
        window: &tauri::WebviewWindow,
        caret: Caret,
        width: f64,
        height: f64,
    ) -> Result<(), String> {
        let mtm = MainThreadMarker::new().ok_or("Window positioning requires the main thread")?;
        let screens = NSScreen::screens(mtm);
        let primary = screens.firstObject().ok_or("No display")?.frame();
        let cocoa_y = primary.size.height - caret.y;
        let mut available = primary;
        for screen in screens.iter() {
            let frame = screen.frame();
            if caret.x >= frame.origin.x
                && caret.x < frame.origin.x + frame.size.width
                && cocoa_y >= frame.origin.y
                && cocoa_y <= frame.origin.y + frame.size.height
            {
                available = screen.visibleFrame();
                break;
            }
        }
        let x = caret.x.clamp(
            available.origin.x,
            (available.origin.x + available.size.width - width).max(available.origin.x),
        );
        let below = cocoa_y - caret.height - 6.0;
        let top = if below - height < available.origin.y {
            cocoa_y + height + 6.0
        } else {
            below
        };
        let top = top.clamp(
            available.origin.y + height,
            (available.origin.y + available.size.height).max(available.origin.y + height),
        );
        let ptr = window.ns_window().map_err(|e| e.to_string())?;
        let ns_window = unsafe { &*(ptr as *const NSWindow) };
        ns_window.setFrameTopLeftPoint(NSPoint::new(x, top));
        // orderFrontRegardless does not make this non-focusable window key.
        ns_window.orderFrontRegardless();
        Ok(())
    }
}

#[cfg(target_os = "macos")]
pub use mac::*;

#[cfg(not(target_os = "macos"))]
pub fn trusted() -> bool {
    false
}
#[cfg(not(target_os = "macos"))]
pub fn request_permission() -> bool {
    false
}
#[cfg(not(target_os = "macos"))]
pub fn terminal_name(_: i32) -> Option<String> {
    None
}

#[cfg(not(target_os = "macos"))]
pub fn uses_cursor_tracker(_: i32) -> bool {
    false
}

#[cfg(not(target_os = "macos"))]
pub fn is_electron(_: i32) -> bool {
    false
}

#[cfg(not(target_os = "macos"))]
pub fn tracker_ready(_: i32) -> Result<(), String> {
    Err("Only macOS is supported".into())
}
#[cfg(not(target_os = "macos"))]
pub fn frontmost_pid() -> i32 {
    0
}
#[cfg(not(target_os = "macos"))]
pub fn setup_cursor(_: &str, _: bool) -> Result<(), String> {
    Err("Only macOS is supported".into())
}
#[cfg(not(target_os = "macos"))]
pub fn request_tracker_caret(_: &str, _: i32) -> Result<(), String> {
    Err("Only macOS is supported".into())
}
#[cfg(not(target_os = "macos"))]
pub fn tracker_caret(_: &figfig_core::CursorPosition) -> Result<Caret, String> {
    Err("Only macOS is supported".into())
}
#[cfg(not(target_os = "macos"))]
pub fn caret(_: i32) -> Result<Caret, String> {
    Err("Only macOS is supported".into())
}
#[cfg(not(target_os = "macos"))]
pub fn place(_: &tauri::WebviewWindow, _: Caret, _: f64, _: f64) -> Result<(), String> {
    Err("Only macOS is supported".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn position() -> figfig_core::CursorPosition {
        figfig_core::CursorPosition {
            request_id: "request".into(),
            terminal_pid: 42,
            x: 120.0,
            y: 800.0,
            height: 20.0,
            focus: 1,
            coordinate_system: Default::default(),
        }
    }

    #[test]
    fn input_method_rect_uses_the_top_of_the_line_in_accessibility_coordinates() {
        let caret = convert_tracker_caret(&position(), 1080.0).unwrap();
        assert_eq!(
            caret,
            Caret {
                x: 120.0,
                y: 260.0,
                height: 20.0,
                pid: 42,
                focus: 1,
                tracker: true,
            }
        );
        let mut secondary = position();
        secondary.x = -1500.0;
        secondary.y = -400.0;
        let caret = convert_tracker_caret(&secondary, 1080.0).unwrap();
        assert_eq!((caret.x, caret.y), (-1500.0, 1460.0));
    }

    #[test]
    fn absent_or_invalid_insertion_points_are_not_used() {
        for height in [0.0, -1.0, 201.0, f64::NAN, f64::INFINITY] {
            let mut value = position();
            value.height = height;
            assert!(convert_tracker_caret(&value, 1080.0).is_err());
        }
        let mut value = position();
        value.x = f64::NAN;
        assert!(convert_tracker_caret(&value, 1080.0).is_err());
        value = position();
        value.y = f64::INFINITY;
        assert!(convert_tracker_caret(&value, 1080.0).is_err());
    }

    #[test]
    fn adapter_coordinates_are_already_top_left_screen_points() {
        let mut value = position();
        value.coordinate_system = figfig_core::CoordinateSystem::Accessibility;
        value.x = -1500.0;
        value.y = -200.0;
        let caret = convert_tracker_caret(&value, 1080.0).unwrap();
        assert_eq!((caret.x, caret.y, caret.height), (-1500.0, -200.0, 20.0));
        assert!(caret.tracker);
    }
}
