// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

#[cfg(desktop)]
mod desktop_commands;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    let mut init_script = String::new();
    // window.print works on Linux/Windows; need to use the API on macOS
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        init_script.push_str(include_str!("./scripts/print.js"));
    }
    init_script.push_str(include_str!("./scripts/drag.js"));
    #[cfg(any(debug_assertions, feature = "devtools"))]
    init_script.push_str(include_str!("./scripts/toggle-devtools.js"));

    init_script.push_str(include_str!("api-iife.js"));

    Builder::new("window")
        .js_init_script(init_script)
        .invoke_handler(|invoke| {
            #[cfg(desktop)]
            {
                let handler: Box<dyn Fn(tauri::Invoke<R>) -> bool> =
                    Box::new(tauri::generate_handler![
                        desktop_commands::create,
                        // getters
                        desktop_commands::scale_factor,
                        desktop_commands::inner_position,
                        desktop_commands::outer_position,
                        desktop_commands::inner_size,
                        desktop_commands::outer_size,
                        desktop_commands::is_fullscreen,
                        desktop_commands::is_minimized,
                        desktop_commands::is_maximized,
                        desktop_commands::is_focused,
                        desktop_commands::is_decorated,
                        desktop_commands::is_resizable,
                        desktop_commands::is_maximizable,
                        desktop_commands::is_minimizable,
                        desktop_commands::is_closable,
                        desktop_commands::is_visible,
                        desktop_commands::title,
                        desktop_commands::current_monitor,
                        desktop_commands::primary_monitor,
                        desktop_commands::available_monitors,
                        desktop_commands::theme,
                        // setters
                        desktop_commands::center,
                        desktop_commands::request_user_attention,
                        desktop_commands::set_resizable,
                        desktop_commands::set_maximizable,
                        desktop_commands::set_minimizable,
                        desktop_commands::set_closable,
                        desktop_commands::set_title,
                        desktop_commands::maximize,
                        desktop_commands::unmaximize,
                        desktop_commands::minimize,
                        desktop_commands::unminimize,
                        desktop_commands::show,
                        desktop_commands::hide,
                        desktop_commands::close,
                        desktop_commands::set_decorations,
                        desktop_commands::set_shadow,
                        desktop_commands::set_effects,
                        desktop_commands::set_always_on_top,
                        desktop_commands::set_content_protected,
                        desktop_commands::set_size,
                        desktop_commands::set_min_size,
                        desktop_commands::set_max_size,
                        desktop_commands::set_position,
                        desktop_commands::set_fullscreen,
                        desktop_commands::set_focus,
                        desktop_commands::set_skip_taskbar,
                        desktop_commands::set_cursor_grab,
                        desktop_commands::set_cursor_visible,
                        desktop_commands::set_cursor_icon,
                        desktop_commands::set_cursor_position,
                        desktop_commands::set_ignore_cursor_events,
                        desktop_commands::start_dragging,
                        desktop_commands::print,
                        desktop_commands::set_icon,
                        desktop_commands::toggle_maximize,
                        desktop_commands::internal_toggle_maximize,
                        #[cfg(any(debug_assertions, feature = "devtools"))]
                        desktop_commands::internal_toggle_devtools,
                    ]);
                #[allow(clippy::needless_return)]
                return handler(invoke);
            }
            #[cfg(mobile)]
            {
                invoke.resolver.reject("Window API not available on mobile");
                return true;
            }
        })
        .build()
}
