// Copyright 2021 Jonas Kruckenberg
// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(feature = "tray-icon")]
use crate::Tray;
use serde_repr::Deserialize_repr;
#[cfg(feature = "tray-icon")]
use tauri::Manager;
#[cfg(feature = "tray-icon")]
use tauri::Monitor;
use tauri::{PhysicalPosition, PhysicalSize, Result, Runtime, WebviewWindow, Window};

/// Well known window positions.
#[derive(Debug, Deserialize_repr)]
#[repr(u16)]
pub enum Position {
    TopLeft = 0,
    TopRight,
    BottomLeft,
    BottomRight,
    TopCenter,
    BottomCenter,
    LeftCenter,
    RightCenter,
    Center,
    #[cfg(feature = "tray-icon")]
    TrayLeft,
    #[cfg(feature = "tray-icon")]
    TrayBottomLeft,
    #[cfg(feature = "tray-icon")]
    TrayRight,
    #[cfg(feature = "tray-icon")]
    TrayBottomRight,
    #[cfg(feature = "tray-icon")]
    TrayCenter,
    #[cfg(feature = "tray-icon")]
    TrayBottomCenter,
}

/// A [`Window`] extension that provides extra methods related to positioning.
pub trait WindowExt {
    /// Moves the [`Window`] to the given [`Position`]
    ///
    /// All (non-tray) positions are relative to the **current** screen.
    fn move_window(&self, position: Position) -> Result<()>;
    #[cfg(feature = "tray-icon")]
    /// Moves the [`Window`] to the given [`Position`] while constraining Tray Positions to the dimensions of the screen.
    ///
    /// All non-tray positions will not be constrained by this method.
    ///
    /// This method allows you to position your Tray Windows without having them
    /// cut off on the screen borders.
    fn move_window_constrained(&self, position: Position) -> Result<()>;
}

impl<R: Runtime> WindowExt for WebviewWindow<R> {
    fn move_window(&self, pos: Position) -> Result<()> {
        self.as_ref().window().move_window(pos)
    }

    #[cfg(feature = "tray-icon")]
    fn move_window_constrained(&self, position: Position) -> Result<()> {
        self.as_ref().window().move_window_constrained(position)
    }
}

impl<R: Runtime> WindowExt for Window<R> {
    #[cfg(feature = "tray-icon")]
    fn move_window_constrained(&self, position: Position) -> Result<()> {
        // Diverge to basic move_window, if the position is not a tray position
        if !matches!(
            position,
            Position::TrayLeft
                | Position::TrayBottomLeft
                | Position::TrayRight
                | Position::TrayBottomRight
                | Position::TrayCenter
                | Position::TrayBottomCenter
        ) {
            return self.move_window(position);
        }

        let window_position = calculate_position(self, position)?;
        let monitor = get_monitor_for_tray_icon(self)?;
        if let Some(monitor) = monitor {
            let monitor_size = monitor.size();
            let monitor_position = monitor.position();
            // Constrain with the size the window will have on the tray's
            // monitor: a cross-monitor move rescales it, so the current
            // physical size is only valid where the window sits now.
            let window_size = {
                let size = self.outer_size()?;
                let current_scale = self
                    .current_monitor()?
                    .map(|m| m.scale_factor())
                    .unwrap_or(1.0);
                let target_scale = monitor.scale_factor();
                if current_scale > 0.0 && current_scale != target_scale {
                    PhysicalSize::<u32> {
                        width: (size.width as f64 / current_scale * target_scale) as u32,
                        height: (size.height as f64 / current_scale * target_scale) as u32,
                    }
                } else {
                    size
                }
            };

            let right_border_monitor = monitor_position.x as f64 + monitor_size.width as f64;
            let left_border_monitor = monitor_position.x as f64;
            let right_border_window = window_position.x as f64 + window_size.width as f64;
            let left_border_window = window_position.x as f64;

            let constrained_x = if left_border_window < left_border_monitor {
                left_border_monitor
            } else if right_border_window > right_border_monitor {
                right_border_monitor - window_size.width as f64
            } else {
                window_position.x as f64
            };

            let bottom_border_monitor = monitor_position.y as f64 + monitor_size.height as f64;
            let top_border_monitor = monitor_position.y as f64;
            let bottom_border_window = window_position.y as f64 + window_size.height as f64;
            let top_border_window = window_position.y as f64;

            let constrained_y = if top_border_window < top_border_monitor {
                top_border_monitor
            } else if bottom_border_window > bottom_border_monitor {
                bottom_border_monitor - window_size.height as f64
            } else {
                window_position.y as f64
            };

            self.set_position(PhysicalPosition::new(constrained_x, constrained_y))?;
        } else {
            // Fallback on non constrained positioning
            self.set_position(window_position)?;
        }

        Ok(())
    }

    fn move_window(&self, pos: Position) -> Result<()> {
        let position = calculate_position(self, pos)?;
        self.set_position(position)
    }
}

#[cfg(feature = "tray-icon")]
/// Retrieve the monitor, where the tray icon is located on.
fn get_monitor_for_tray_icon<R: Runtime>(window: &Window<R>) -> Result<Option<Monitor>> {
    let tray_position = window
        .state::<Tray>()
        .0
        .lock()
        .unwrap()
        .map(|(pos, _)| pos)
        .unwrap_or_default();

    window.monitor_from_point(tray_position.x, tray_position.y)
}

/// Calculate the top-left position of the window based on the given
/// [`Position`].
fn calculate_position<R: Runtime>(
    window: &Window<R>,
    pos: Position,
) -> Result<PhysicalPosition<i32>> {
    let screen = window.current_monitor()?.ok_or_else(|| {
        tauri::Error::Io(std::io::Error::other("No monitor found for the window"))
    })?;
    // Only use the screen_position for the Tray independent positioning,
    // because a tray event may not be called on the currently active monitor.
    let screen_position = screen.position();
    let screen_size = PhysicalSize::<i32> {
        width: screen.size().width as i32,
        height: screen.size().height as i32,
    };
    let window_size = PhysicalSize::<i32> {
        width: window.outer_size()?.width as i32,
        height: window.outer_size()?.height as i32,
    };
    #[cfg(feature = "tray-icon")]
    let (tray_position, tray_size) = window
        .state::<Tray>()
        .0
        .lock()
        .unwrap()
        .map(|(pos, size)| {
            (
                Some((pos.x as i32, pos.y as i32)),
                Some((size.width as i32, size.height as i32)),
            )
        })
        .unwrap_or_default();
    #[cfg(not(feature = "tray-icon"))]
    let (tray_position, tray_size) = (None, None);

    // A move to another monitor rescales the window, so its current physical
    // size is only valid on the monitor it occupies right now. For the
    // tray-relative positions, express the size at the tray's monitor scale so
    // the window lands centered with the size it will actually have there.
    #[cfg(feature = "tray-icon")]
    let window_size = match (&pos, tray_position) {
        (
            Position::TrayLeft
            | Position::TrayBottomLeft
            | Position::TrayRight
            | Position::TrayBottomRight
            | Position::TrayCenter
            | Position::TrayBottomCenter,
            Some((tray_x, tray_y)),
        ) => match window.monitor_from_point(tray_x as f64, tray_y as f64)? {
            Some(tray_monitor) => {
                let current_scale = screen.scale_factor();
                let target_scale = tray_monitor.scale_factor();
                if current_scale > 0.0 && current_scale != target_scale {
                    PhysicalSize::<i32> {
                        width: (window_size.width as f64 / current_scale * target_scale) as i32,
                        height: (window_size.height as f64 / current_scale * target_scale) as i32,
                    }
                } else {
                    window_size
                }
            }
            None => window_size,
        },
        _ => window_size,
    };

    compute_position(
        *screen_position,
        screen_size,
        window_size,
        tray_position,
        tray_size,
        pos,
    )
}

/// The pure positioning math: computes the window's top-left corner from the
/// screen, window, and tray geometry alone, with no window-system access.
fn compute_position(
    screen_position: PhysicalPosition<i32>,
    screen_size: PhysicalSize<i32>,
    window_size: PhysicalSize<i32>,
    tray_position: Option<(i32, i32)>,
    tray_size: Option<(i32, i32)>,
    pos: Position,
) -> Result<PhysicalPosition<i32>> {
    use Position::*;

    #[cfg(not(feature = "tray-icon"))]
    let _ = (tray_position, tray_size);
    let screen_position = &screen_position;

    let physical_pos = match pos {
        TopLeft => *screen_position,
        TopRight => PhysicalPosition {
            x: screen_position.x + (screen_size.width - window_size.width),
            y: screen_position.y,
        },
        BottomLeft => PhysicalPosition {
            x: screen_position.x,
            y: screen_size.height - (window_size.height - screen_position.y),
        },
        BottomRight => PhysicalPosition {
            x: screen_position.x + (screen_size.width - window_size.width),
            y: screen_size.height - (window_size.height - screen_position.y),
        },
        TopCenter => PhysicalPosition {
            x: screen_position.x + ((screen_size.width / 2) - (window_size.width / 2)),
            y: screen_position.y,
        },
        BottomCenter => PhysicalPosition {
            x: screen_position.x + ((screen_size.width / 2) - (window_size.width / 2)),
            y: screen_size.height - (window_size.height - screen_position.y),
        },
        LeftCenter => PhysicalPosition {
            x: screen_position.x,
            y: screen_position.y + (screen_size.height / 2) - (window_size.height / 2),
        },
        RightCenter => PhysicalPosition {
            x: screen_position.x + (screen_size.width - window_size.width),
            y: screen_position.y + (screen_size.height / 2) - (window_size.height / 2),
        },
        Center => PhysicalPosition {
            x: screen_position.x + ((screen_size.width / 2) - (window_size.width / 2)),
            y: screen_position.y + (screen_size.height / 2) - (window_size.height / 2),
        },
        #[cfg(feature = "tray-icon")]
        TrayLeft => {
            if let (Some((tray_x, tray_y)), Some((_, _tray_height))) = (tray_position, tray_size) {
                let y = tray_y - window_size.height;
                // Choose y value based on the target OS
                #[cfg(target_os = "windows")]
                let y = if y < 0 { tray_y + _tray_height } else { y };

                #[cfg(target_os = "macos")]
                let y = if y < 0 { tray_y } else { y };

                PhysicalPosition { x: tray_x, y }
            } else {
                return Err(tauri::Error::Io(std::io::Error::other(
                    "Tray position not set",
                )));
            }
        }
        #[cfg(feature = "tray-icon")]
        TrayBottomLeft => {
            if let Some((tray_x, tray_y)) = tray_position {
                PhysicalPosition {
                    x: tray_x,
                    y: tray_y,
                }
            } else {
                return Err(tauri::Error::Io(std::io::Error::other(
                    "Tray position not set",
                )));
            }
        }
        #[cfg(feature = "tray-icon")]
        TrayRight => {
            if let (Some((tray_x, tray_y)), Some((tray_width, _tray_height))) =
                (tray_position, tray_size)
            {
                let y = tray_y - window_size.height;
                // Choose y value based on the target OS
                #[cfg(target_os = "windows")]
                let y = if y < 0 { tray_y + _tray_height } else { y };

                #[cfg(target_os = "macos")]
                let y = if y < 0 { tray_y } else { y };

                PhysicalPosition {
                    x: tray_x + tray_width,
                    y,
                }
            } else {
                return Err(tauri::Error::Io(std::io::Error::other(
                    "Tray position not set",
                )));
            }
        }
        #[cfg(feature = "tray-icon")]
        TrayBottomRight => {
            if let (Some((tray_x, tray_y)), Some((tray_width, _))) = (tray_position, tray_size) {
                PhysicalPosition {
                    x: tray_x + tray_width,
                    y: tray_y,
                }
            } else {
                return Err(tauri::Error::Io(std::io::Error::other(
                    "Tray position not set",
                )));
            }
        }
        #[cfg(feature = "tray-icon")]
        TrayCenter => {
            if let (Some((tray_x, tray_y)), Some((tray_width, _tray_height))) =
                (tray_position, tray_size)
            {
                let x = tray_x + tray_width / 2 - window_size.width / 2;
                let y = tray_y - window_size.height;
                // Choose y value based on the target OS
                #[cfg(target_os = "windows")]
                let y = if y < 0 { tray_y + _tray_height } else { y };

                #[cfg(target_os = "macos")]
                let y = if y < 0 { tray_y } else { y };

                PhysicalPosition { x, y }
            } else {
                return Err(tauri::Error::Io(std::io::Error::other(
                    "Tray position not set",
                )));
            }
        }
        #[cfg(feature = "tray-icon")]
        TrayBottomCenter => {
            if let (Some((tray_x, tray_y)), Some((tray_width, _))) = (tray_position, tray_size) {
                PhysicalPosition {
                    x: tray_x + (tray_width / 2) - (window_size.width / 2),
                    y: tray_y,
                }
            } else {
                return Err(tauri::Error::Io(std::io::Error::other(
                    "Tray position not set",
                )));
            }
        }
    };

    Ok(physical_pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Synthetic mixed-DPI fixture mirroring a real arrangement: a 4K external
    // at scale 1 as primary, a Retina laptop panel at scale 2 below-left of
    // it. Physical (pixel) coordinates throughout, like the plugin uses.
    const EXTERNAL_POS: PhysicalPosition<i32> = PhysicalPosition { x: 0, y: 0 };
    const EXTERNAL_SIZE: PhysicalSize<i32> = PhysicalSize {
        width: 3840,
        height: 2160,
    };
    const RETINA_POS: PhysicalPosition<i32> = PhysicalPosition { x: -4112, y: 1042 };
    const RETINA_SIZE: PhysicalSize<i32> = PhysicalSize {
        width: 4112,
        height: 2658,
    };
    const WINDOW: PhysicalSize<i32> = PhysicalSize {
        width: 400,
        height: 300,
    };

    fn on_retina(pos: Position) -> PhysicalPosition<i32> {
        compute_position(RETINA_POS, RETINA_SIZE, WINDOW, None, None, pos).unwrap()
    }

    #[test]
    fn corners_and_centers_on_a_negative_origin_monitor() {
        assert_eq!(on_retina(Position::TopLeft), RETINA_POS);
        assert_eq!(
            on_retina(Position::TopRight),
            PhysicalPosition { x: -400, y: 1042 }
        );
        assert_eq!(
            on_retina(Position::Center),
            PhysicalPosition { x: -2256, y: 2221 }
        );
        // The bottom-edge form `H - (h - S.y)` is algebraically the monitor's
        // bottom edge minus the window height: 2658 - (300 - 1042) = 3400,
        // and 1042 + 2658 - 300 = 3400. Keep it; it is correct.
        assert_eq!(
            on_retina(Position::BottomLeft),
            PhysicalPosition { x: -4112, y: 3400 }
        );
        assert_eq!(
            on_retina(Position::BottomRight),
            PhysicalPosition { x: -400, y: 3400 }
        );
    }

    #[test]
    fn corners_and_centers_on_the_primary() {
        let on_primary =
            |pos| compute_position(EXTERNAL_POS, EXTERNAL_SIZE, WINDOW, None, None, pos).unwrap();
        assert_eq!(on_primary(Position::TopLeft), EXTERNAL_POS);
        assert_eq!(
            on_primary(Position::BottomRight),
            PhysicalPosition { x: 3440, y: 1860 }
        );
        assert_eq!(
            on_primary(Position::Center),
            PhysicalPosition { x: 1720, y: 930 }
        );
        assert_eq!(
            on_primary(Position::LeftCenter),
            PhysicalPosition { x: 0, y: 930 }
        );
    }

    #[cfg(feature = "tray-icon")]
    fn with_tray(
        tray_position: (i32, i32),
        tray_size: (i32, i32),
        pos: Position,
    ) -> PhysicalPosition<i32> {
        compute_position(
            EXTERNAL_POS,
            EXTERNAL_SIZE,
            WINDOW,
            Some(tray_position),
            Some(tray_size),
            pos,
        )
        .unwrap()
    }

    #[cfg(feature = "tray-icon")]
    #[test]
    fn tray_positions_follow_the_tray_rect() {
        // Tray icon in a menu bar: y = 0, window cannot fit above it.
        let at_top = with_tray((1000, 0), (44, 44), Position::TrayCenter);
        assert_eq!(at_top.x, 1000 + 22 - 200);
        #[cfg(target_os = "macos")]
        assert_eq!(at_top.y, 0);
        #[cfg(target_os = "windows")]
        assert_eq!(at_top.y, 44);

        // Tray rect below the window's height: no clamping on any platform.
        let mid = with_tray((1000, 800), (44, 44), Position::TrayLeft);
        assert_eq!(mid, PhysicalPosition { x: 1000, y: 500 });

        let bottom = with_tray((1000, 800), (44, 44), Position::TrayBottomCenter);
        assert_eq!(bottom, PhysicalPosition { x: 822, y: 800 });
    }

    #[cfg(feature = "tray-icon")]
    #[test]
    fn tray_positions_error_without_a_tray_rect() {
        let result = compute_position(
            EXTERNAL_POS,
            EXTERNAL_SIZE,
            WINDOW,
            None,
            None,
            Position::TrayCenter,
        );
        assert!(result.is_err());
    }
}
