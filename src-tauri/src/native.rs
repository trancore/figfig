//! macOS integration reads caret geometry, never terminal contents or keystrokes.
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
pub struct Caret {
    pub x: f64,
    pub y: f64,
    pub height: f64,
    pub pid: i32,
    pub focus: usize,
}

#[cfg(target_os = "macos")]
mod mac {
    use super::Caret;
    use core_foundation::{
        base::TCFType, boolean::CFBoolean, dictionary::CFDictionary, string::CFString,
    };
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSScreen, NSWindow};
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
        if name.contains("/Terminal.app/") {
            Some("Terminal.app".into())
        } else if name.contains("/iTerm.app/") || name.contains("/iTerm2.app/") {
            Some("iTerm2".into())
        } else {
            None
        }
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
            return Err("Focus Terminal.app or iTerm2".into());
        }
        let element = app.attr("AXFocusedUIElement")?;
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
            if range.location <= 0 {
                return Err("Terminal does not expose caret bounds".into());
            }
            range.location -= 1;
            range.length = 1;
            let previous = Owned::new(unsafe { AXValueCreate(4, &range as *const _ as Ref) })?;
            if !get_rect(previous.0, &mut rect) {
                return Err("Terminal does not expose caret bounds".into());
            }
            rect.origin.x += rect.size.width;
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
pub fn caret(_: i32) -> Result<Caret, String> {
    Err("Only macOS is supported".into())
}
#[cfg(not(target_os = "macos"))]
pub fn place(_: &tauri::WebviewWindow, _: Caret, _: f64, _: f64) -> Result<(), String> {
    Err("Only macOS is supported".into())
}
