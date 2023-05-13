use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

mod commands;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("window")
        .invoke_handler(tauri::generate_handler![
            #[cfg(feature = "allow-create")]
            commands::create,
            // getters
            #[cfg(feature = "allow-scale-factor")]
            commands::scale_factor,
            #[cfg(feature = "allow-inner-position")]
            commands::inner_position,
            #[cfg(feature = "allow-outer-position")]
            commands::outer_position,
            #[cfg(feature = "allow-inner-size")]
            commands::inner_size,
            #[cfg(feature = "allow-outer-size")]
            commands::outer_size,
            #[cfg(feature = "allow-is-fullscreen")]
            commands::is_fullscreen,
            #[cfg(feature = "allow-is-minimized")]
            commands::is_minimized,
            #[cfg(feature = "allow-is-maximized")]
            commands::is_maximized,
            #[cfg(feature = "allow-is-decorated")]
            commands::is_decorated,
            #[cfg(feature = "allow-is-resizable")]
            commands::is_resizable,
            #[cfg(feature = "allow-is-visible")]
            commands::is_visible,
            #[cfg(feature = "allow-title")]
            commands::title,
            #[cfg(feature = "allow-current-monitor")]
            commands::current_monitor,
            #[cfg(feature = "allow-primary-monitor")]
            commands::primary_monitor,
            #[cfg(feature = "allow-available-monitors")]
            commands::available_monitors,
            #[cfg(feature = "allow-theme")]
            commands::theme,
            // setters
            #[cfg(feature = "allow-center")]
            commands::center,
            #[cfg(feature = "allow-request-user-attention")]
            commands::request_user_attention,
            #[cfg(feature = "allow-set-resizable")]
            commands::set_resizable,
            #[cfg(feature = "allow-set-title")]
            commands::set_title,
            #[cfg(feature = "allow-maximize")]
            commands::maximize,
            #[cfg(feature = "allow-unmaximize")]
            commands::unmaximize,
            #[cfg(feature = "allow-minimize")]
            commands::minimize,
            #[cfg(feature = "allow-unminimize")]
            commands::unminimize,
            #[cfg(feature = "allow-show")]
            commands::show,
            #[cfg(feature = "allow-hide")]
            commands::hide,
            #[cfg(feature = "allow-close")]
            commands::close,
            #[cfg(feature = "allow-set-decorations")]
            commands::set_decorations,
            #[cfg(feature = "allow-set-shadow")]
            commands::set_shadow,
            #[cfg(feature = "allow-set-always-on-top")]
            commands::set_always_on_top,
            #[cfg(feature = "allow-set-content-protected")]
            commands::set_content_protected,
            #[cfg(feature = "allow-set-size")]
            commands::set_size,
            #[cfg(feature = "allow-set-min-size")]
            commands::set_min_size,
            #[cfg(feature = "allow-set-max-size")]
            commands::set_max_size,
            #[cfg(feature = "allow-set-position")]
            commands::set_position,
            #[cfg(feature = "allow-set-fullscreen")]
            commands::set_fullscreen,
            #[cfg(feature = "allow-set-focus")]
            commands::set_focus,
            #[cfg(feature = "allow-set-skip-taskbar")]
            commands::set_skip_taskbar,
            #[cfg(feature = "allow-set-cursor-grab")]
            commands::set_cursor_grab,
            #[cfg(feature = "allow-set-cursor-visible")]
            commands::set_cursor_visible,
            #[cfg(feature = "allow-set-cursor-icon")]
            commands::set_cursor_icon,
            #[cfg(feature = "allow-set-cursor-position")]
            commands::set_cursor_position,
            #[cfg(feature = "allow-set-ignore-cursor-events")]
            commands::set_ignore_cursor_events,
            #[cfg(feature = "allow-start-dragging")]
            commands::start_dragging,
            #[cfg(feature = "allow-print")]
            commands::print,
            #[cfg(feature = "allow-set-icon")]
            commands::set_icon,
            #[cfg(feature = "allow-toggle-maximize")]
            commands::toggle_maximize,
            #[cfg(feature = "allow-toggle-maximize")]
            commands::internal_toggle_maximize,
            #[cfg(debug_assertions)]
            commands::internal_toggle_devtools,
        ])
        .build()
}
