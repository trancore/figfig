mod tracker;

pub use tracker::{request_tracker_caret, setup_cursor, tracker_ready};

use super::{Caret, PositionSource, TerminalSize};
use core_foundation::{
    base::TCFType, boolean::CFBoolean, dictionary::CFDictionary, string::CFString,
};
use objc2::MainThreadMarker;
use objc2_app_kit::{NSScreen, NSWindow, NSWindowCollectionBehavior, NSWorkspace};
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
    fn CFGetTypeID(value: Ref) -> usize;
    fn CFArrayGetTypeID() -> usize;
}
unsafe extern "C" {
    fn proc_pidpath(pid: i32, buffer: *mut c_void, buffer_size: u32) -> i32;
}

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGEventSourceCounterForEventType(state: i32, event_type: u32) -> u32;
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

    fn frame(&self, invalid_bounds: &str) -> Result<Rect, String> {
        let position = self.attr("AXPosition")?;
        let size = self.attr("AXSize")?;
        let mut rect = Rect::default();
        if !unsafe { AXValueGetValue(position.0, 1, &mut rect.origin as *mut _ as *mut c_void) }
            || !unsafe { AXValueGetValue(size.0, 2, &mut rect.size as *mut _ as *mut c_void) }
        {
            return Err(invalid_bounds.into());
        }
        Ok(rect)
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

pub fn terminal_host(pid: i32) -> Option<&'static figfig_core::terminal::TerminalHost> {
    let mut buffer = [0u8; 4096];
    let length =
        unsafe { proc_pidpath(pid, buffer.as_mut_ptr() as *mut c_void, buffer.len() as u32) };
    if length <= 0 {
        return None;
    }
    let path = String::from_utf8_lossy(&buffer[..length as usize]);
    figfig_core::terminal::host_for_executable(path.trim_end_matches('\0'))
}

pub fn frontmost_pid() -> i32 {
    NSWorkspace::sharedWorkspace()
        .frontmostApplication()
        .map_or(0, |app| app.processIdentifier())
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

pub fn zed_window_anchor(pid: i32, terminal_size: Option<TerminalSize>) -> Result<Caret, String> {
    if !trusted() {
        return Err("Run `figfig permissions` so figfig can position the popup in Zed.".into());
    }
    if pid <= 0 || frontmost_pid() != pid {
        return Err("The shell's terminal is not focused".into());
    }

    let (app, _) = focused_application(pid)?;
    let window = app.attr("AXFocusedWindow")?;
    let rect = window.frame("Zed did not expose valid window bounds")?;
    if !rect.origin.x.is_finite()
        || !rect.origin.y.is_finite()
        || !rect.size.width.is_finite()
        || !rect.size.height.is_finite()
        || rect.size.width <= 0.0
        || rect.size.height <= 0.0
    {
        return Err("Zed did not expose valid window bounds".into());
    }

    const COMBINED_SESSION: i32 = 0;
    const MOUSE_EVENTS: [u32; 4] = [1, 3, 25, 22]; // Left, right, other, scroll.
    let mouse_events = MOUSE_EVENTS
        .map(|event| unsafe { CGEventSourceCounterForEventType(COMBINED_SESSION, event) });
    let focus = app.attr("AXFocusedUIElement").ok();
    let grid_offset = terminal_size
        .and_then(|size| super::zed_terminal_offset(rect.size.width, rect.size.height, size));
    let (offset_x, offset_y, height) = grid_offset.unwrap_or((
        rect.size.width / 2.0,
        rect.size.height - 42.0_f64.min(rect.size.height / 2.0),
        0.0,
    ));
    Ok(Caret {
        x: rect.origin.x + offset_x,
        y: rect.origin.y + offset_y,
        height,
        pid,
        focus: unsafe { CFHash(window.0) }
            ^ focus.map_or(0, |element| unsafe { CFHash(element.0) }.rotate_left(1)),
        source: PositionSource::ZedWindow {
            mouse_events,
            terminal_size: terminal_size.filter(|_| grid_offset.is_some()),
        },
    })
}

fn focused_application(expected_pid: i32) -> Result<(Owned, i32), String> {
    let system = Owned::new(unsafe { AXUIElementCreateSystemWide() })?;
    unsafe { AXUIElementSetMessagingTimeout(system.0, 0.05) };
    let app = system.attr("AXFocusedApplication")?;
    let mut pid = 0;
    if unsafe { AXUIElementGetPid(app.0, &mut pid) } != 0 {
        return Err("No focused application".into());
    }
    if expected_pid > 0 && pid != expected_pid {
        return Err("The shell's terminal is not focused".into());
    }
    Ok((app, pid))
}

pub fn caret(expected_pid: i32) -> Result<Caret, String> {
    if !trusted() {
        return Err("Accessibility permission is required".into());
    }
    let (app, pid) = focused_application(expected_pid)?;
    let host =
        terminal_host(pid).ok_or("Focus a supported terminal or editor integrated terminal")?;
    let element = app.attr("AXFocusedUIElement")?;
    let is_electron = host.family == "electron";
    if is_electron {
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
        if is_electron {
            // xterm positions its hidden IME textarea at the cursor. Its
            // element bounds remain useful when AXBoundsForRange is absent.
            rect = element.frame("Terminal does not expose caret bounds")?;
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
        source: PositionSource::Accessibility,
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
    let anchored_to_window = matches!(
        caret.source,
        PositionSource::ZedWindow {
            terminal_size: None,
            ..
        }
    );
    let left = if anchored_to_window {
        caret.x - width / 2.0
    } else {
        caret.x
    };
    let x = left.clamp(
        available.origin.x,
        (available.origin.x + available.size.width - width).max(available.origin.x),
    );
    let gap = match caret.source {
        PositionSource::ZedWindow {
            terminal_size: Some(_),
            ..
        } => 2.0,
        _ => 6.0,
    };
    let below = cocoa_y - caret.height - gap;
    let top = if anchored_to_window || below - height < available.origin.y {
        cocoa_y + height + gap
    } else {
        below
    };
    let top = top.clamp(
        available.origin.y + height,
        (available.origin.y + available.size.height).max(available.origin.y + height),
    );
    let ptr = window.ns_window().map_err(|e| e.to_string())?;
    let ns_window = unsafe { &*(ptr as *const NSWindow) };
    ns_window.setIgnoresMouseEvents(true);
    ns_window.setCollectionBehavior(
        ns_window.collectionBehavior() | NSWindowCollectionBehavior::FullScreenAuxiliary,
    );
    ns_window.setFrameTopLeftPoint(NSPoint::new(x, top));
    // orderFrontRegardless does not make this non-focusable window key.
    ns_window.orderFrontRegardless();
    Ok(())
}
