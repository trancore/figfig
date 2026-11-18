//! macOS integration reads window and caret geometry, never terminal contents or keystrokes.

use figfig_core::protocol::TerminalSize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionSource {
    Accessibility,
    Tracker,
    // Mouse events let us dismiss if the terminal loses focus or scrolls.
    ZedWindow {
        mouse_events: [u32; 4],
        terminal_size: Option<TerminalSize>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Caret {
    pub x: f64,
    pub y: f64,
    pub height: f64,
    pub pid: i32,
    pub focus: usize,
    pub source: PositionSource,
}

#[cfg(any(target_os = "macos", test))]
fn zed_terminal_offset(
    window_width: f64,
    window_height: f64,
    size: TerminalSize,
) -> Option<(f64, f64, f64)> {
    const BOTTOM_CHROME: f64 = 32.0;
    let width = f64::from(size.width);
    let height = f64::from(size.height);
    // Only infer the origin for a wide, shallow grid consistent with a bottom
    // dock. Zed doesn't expose the grid's position or the side docks' widths.
    if width <= window_width / 2.0
        || width > window_width
        || height == 0.0
        || height + BOTTOM_CHROME > window_height * 2.0 / 3.0
    {
        return None;
    }
    let cell = size.cursor?;
    if !(1..=cell.rows).contains(&cell.row) || !(1..=cell.columns).contains(&cell.column) {
        return None;
    }
    let line_height = height / f64::from(cell.rows);
    let cell_width = width / f64::from(cell.columns);
    let rows_below = f64::from(cell.rows - cell.row);
    // Zed truncates the cell height passed to the PTY. It can also leave a
    // partial row below the grid until the terminal fills. The position of
    // the last row does not depend on either estimate.
    let rounding_loss = (rows_below * 0.5).min(line_height / 2.0);
    let bottom_padding = (rows_below / 4.0).min(1.0) * line_height / 2.0;
    let line_bottom =
        window_height - BOTTOM_CHROME - rows_below * line_height - rounding_loss - bottom_padding;
    Some((
        (window_width - width) / 2.0 + f64::from(cell.column - 1) * cell_width,
        line_bottom - line_height,
        line_height,
    ))
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
        source: PositionSource::Tracker,
    })
}

#[cfg(target_os = "macos")]
mod mac;

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
pub fn terminal_host(_: i32) -> Option<&'static figfig_core::terminal::TerminalHost> {
    None
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
pub fn zed_window_anchor(_: i32, _: Option<TerminalSize>) -> Result<Caret, String> {
    Err("Only macOS is supported".into())
}
#[cfg(not(target_os = "macos"))]
pub fn place(_: &tauri::WebviewWindow, _: Caret, _: f64, _: f64) -> Result<(), String> {
    Err("Only macOS is supported".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_zed_bottom_dock_anchors_at_the_reported_cell() {
        let grid = TerminalSize {
            width: 1458,
            height: 342,
            cursor: Some(figfig_core::protocol::TerminalCell {
                row: 3,
                column: 7,
                rows: 18,
                columns: 162,
            }),
        };
        let offset = zed_terminal_offset(2560.0, 1440.0, grid);
        assert_eq!(offset, Some((605.0, 1087.0, 19.0)));

        let mut scrolled = grid;
        scrolled.cursor.as_mut().unwrap().row = 18;
        assert_eq!(
            zed_terminal_offset(2560.0, 1440.0, scrolled),
            Some((605.0, 1389.0, 19.0))
        );
        scrolled.cursor.as_mut().unwrap().row = 19;
        assert_eq!(zed_terminal_offset(2560.0, 1440.0, scrolled), None);
    }

    #[test]
    fn zed_grid_without_bottom_dock_dimensions_uses_window_placement() {
        for grid in [
            TerminalSize {
                width: 500,
                height: 900,
                cursor: None,
            },
            TerminalSize {
                width: 2000,
                height: 1200,
                cursor: None,
            },
            TerminalSize {
                width: 2000,
                height: 0,
                cursor: None,
            },
            TerminalSize {
                width: 3000,
                height: 300,
                cursor: None,
            },
        ] {
            assert_eq!(zed_terminal_offset(2560.0, 1440.0, grid), None, "{grid:?}");
        }
    }

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
                source: PositionSource::Tracker,
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
        assert_eq!(caret.source, PositionSource::Tracker);
    }
}
