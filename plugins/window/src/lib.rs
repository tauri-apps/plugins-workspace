use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

mod commands;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("window")
        .invoke_handler(tauri::generate_handler![
            commands::create,
            // getters
            commands::scale_factor,
            commands::inner_position,
            commands::outer_position,
            commands::inner_size,
            commands::outer_size,
            commands::is_fullscreen,
            commands::is_minimized,
            commands::is_maximized,
            commands::is_decorated,
            commands::is_resizable,
            commands::is_visible,
            commands::title,
            commands::current_monitor,
            commands::primary_monitor,
            commands::available_monitors,
            commands::theme,
            // setters
            commands::center,
            commands::request_user_attention,
            commands::set_resizable,
            commands::set_title,
            commands::maximize,
            commands::unmaximize,
            commands::minimize,
            commands::unminimize,
            commands::show,
            commands::hide,
            commands::close,
            commands::set_decorations,
            commands::set_shadow,
            commands::set_always_on_top,
            commands::set_content_protected,
            commands::set_size,
            commands::set_min_size,
            commands::set_max_size,
            commands::set_position,
            commands::set_fullscreen,
            commands::set_focus,
            commands::set_skip_taskbar,
            commands::set_cursor_grab,
            commands::set_cursor_visible,
            commands::set_cursor_icon,
            commands::set_cursor_position,
            commands::set_ignore_cursor_events,
            commands::start_dragging,
            commands::print,
            commands::set_icon,
            commands::toggle_maximize,
            commands::internal_toggle_maximize,
            commands::internal_toggle_devtools,
        ])
        .build()
}
