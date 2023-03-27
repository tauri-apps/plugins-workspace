// Copyright 2021 Jonas Kruckenberg
// SPDX-License-Identifier: MIT

#[cfg(feature = "system-tray")]
use crate::Tray;
use serde_repr::Deserialize_repr;
#[cfg(feature = "system-tray")]
use tauri::Manager;
use tauri::{PhysicalPosition, PhysicalSize, Result, Runtime, Window};

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
    #[cfg(feature = "system-tray")]
    TrayLeft,
    #[cfg(feature = "system-tray")]
    TrayFixedLeft,
    #[cfg(feature = "system-tray")]
    TrayBottomLeft,
    #[cfg(feature = "system-tray")]
    TrayRight,
    #[cfg(feature = "system-tray")]
    TrayFixedRight,
    #[cfg(feature = "system-tray")]
    TrayBottomRight,
    #[cfg(feature = "system-tray")]
    TrayCenter,
    #[cfg(feature = "system-tray")]
    TrayFixedCenter,
    #[cfg(feature = "system-tray")]
    TrayBottomCenter,
}

/// A [`Window`] extension that provides extra methods related to positioning.
pub trait WindowExt {
    /// Moves the [`Window`] to the given [`Position`]
    ///
    /// All positions are relative to the **current** screen.
    fn move_window(&self, position: Position) -> Result<()>;
}

impl<R: Runtime> WindowExt for Window<R> {
    fn move_window(&self, pos: Position) -> Result<()> {
        use Position::*;

        let screen = self.current_monitor()?.unwrap();
        let screen_position = screen.position();
        let screen_size = PhysicalSize::<i32> {
            width: screen.size().width as i32,
            height: screen.size().height as i32,
        };
        let window_size = PhysicalSize::<i32> {
            width: self.outer_size()?.width as i32,
            height: self.outer_size()?.height as i32,
        };
        #[cfg(feature = "system-tray")]
        let (tray_position, tray_size) = self
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
            #[cfg(feature = "system-tray")]
            TrayLeft => {
                if let Some((tray_x, tray_y)) = tray_position {
                    PhysicalPosition {
                        x: tray_x,
                        y: tray_y - window_size.height,
                    }
                } else {
                    panic!("Tray position not set");
                }
            }
            #[cfg(feature = "system-tray")]
            TrayFixedLeft => {
                if let(Some((tray_x, tray_y)), Some((_, tray_height))) = (tray_position, tray_size) {
                    let y = tray_y - window_size.height;
                    // Choose y value based on the target OS
                    #[cfg(target_os = "windows")]
                    let y = if y < 0 { tray_y + tray_height } else { y };

                    #[cfg(target_os = "macos")]
                    let y = if y < 0 { tray_y } else { y };

                    PhysicalPosition { x: tray_x, y }
                } else {
                    panic!("Tray position not set");
                }
            }
            #[cfg(feature = "system-tray")]
            TrayBottomLeft => {
                if let Some((tray_x, tray_y)) = tray_position {
                    PhysicalPosition {
                        x: tray_x,
                        y: tray_y,
                    }
                } else {
                    panic!("Tray position not set");
                }
            }
            #[cfg(feature = "system-tray")]
            TrayRight => {
                if let (Some((tray_x, tray_y)), Some((tray_width, _))) = (tray_position, tray_size)
                {
                    PhysicalPosition {
                        x: tray_x + tray_width,
                        y: tray_y - window_size.height,
                    }
                } else {
                    panic!("Tray position not set");
                }
            }
            #[cfg(feature = "system-tray")]
            TrayFixedRight => {
                if let(Some((tray_x, tray_y)), Some((tray_width, tray_height))) = (tray_position, tray_size) {
                    let y = tray_y - window_size.height;
                    // Choose y value based on the target OS
                    #[cfg(target_os = "windows")]
                    let y = if y < 0 { tray_y + tray_height } else { y };

                    #[cfg(target_os = "macos")]
                    let y = if y < 0 { tray_y } else { y };

                    PhysicalPosition { x: tray_x + tray_width, y }
                } else {
                    panic!("Tray position not set");
                }
            }
            #[cfg(feature = "system-tray")]
            TrayBottomRight => {
                if let (Some((tray_x, tray_y)), Some((tray_width, _))) = (tray_position, tray_size)
                {
                    PhysicalPosition {
                        x: tray_x + tray_width,
                        y: tray_y,
                    }
                } else {
                    panic!("Tray position not set");
                }
            }
            #[cfg(feature = "system-tray")]
            TrayCenter => {
                if let (Some((tray_x, tray_y)), Some((tray_width, _))) = (tray_position, tray_size)
                {
                    PhysicalPosition {
                        x: tray_x + (tray_width / 2) - (window_size.width / 2),
                        y: tray_y - window_size.height,
                    }
                } else {
                    panic!("Tray position not set");
                }
            }
            #[cfg(feature = "system-tray")]
            TrayFixedCenter => {
                if let (Some((tray_x, tray_y)), Some((tray_width, tray_height))) = (tray_position, tray_size)
                {
                    let x = tray_x + tray_width / 2 - window_size.width / 2;
                    let y = tray_y - window_size.height;
                    // Choose y value based on the target OS
                    #[cfg(target_os = "windows")]
                    let y = if y < 0 { tray_y + tray_height } else { y };

                    #[cfg(target_os = "macos")]
                    let y = if y < 0 { tray_y } else { y };

                    PhysicalPosition { x, y }
                } else {
                    panic!("Tray position not set");
                }
            }
            #[cfg(feature = "system-tray")]
            TrayBottomCenter => {
                if let (Some((tray_x, tray_y)), Some((tray_width, _))) = (tray_position, tray_size)
                {
                    PhysicalPosition {
                        x: tray_x + (tray_width / 2) - (window_size.width / 2),
                        y: tray_y,
                    }
                } else {
                    panic!("Tray position not set");
                }
            }
        };

        self.set_position(tauri::Position::Physical(physical_pos))
    }
}
