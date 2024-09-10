// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{
    plugin::{Builder, PluginApi, TauriPlugin},
    AppHandle, Manager, Runtime,
};

mod commands;
mod config;
mod error;

pub use error::{Error, Result};

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "app.tauri.deep_link";

fn init_deep_link<R: Runtime>(
    app: &AppHandle<R>,
    api: PluginApi<R, Option<config::Config>>,
) -> crate::Result<DeepLink<R>> {
    #[cfg(target_os = "android")]
    {
        let _api = api;

        use tauri::{
            ipc::{Channel, InvokeResponseBody},
            Emitter,
        };

        let handle = _api.register_android_plugin(PLUGIN_IDENTIFIER, "DeepLinkPlugin")?;

        #[derive(serde::Deserialize)]
        struct Event {
            url: String,
        }

        let app_handle = app.clone();
        handle.run_mobile_plugin::<()>(
            "setEventHandler",
            imp::EventHandler {
                handler: Channel::new(move |event| {
                    let url = match event {
                        InvokeResponseBody::Json(payload) => {
                            serde_json::from_str::<Event>(&payload)
                                .ok()
                                .map(|payload| payload.url)
                        }
                        _ => None,
                    };

                    let _ = app_handle.emit("deep-link://new-url", vec![url]);

                    Ok(())
                }),
            },
        )?;

        return Ok(DeepLink(handle));
    }

    #[cfg(target_os = "ios")]
    return Ok(DeepLink {
        app: app.clone(),
        current: Default::default(),
        config: api.config().clone(),
    });

    #[cfg(desktop)]
    {
        let args = std::env::args();
        let current = if let Some(config) = api.config() {
            imp::deep_link_from_args(config, args)
        } else {
            None
        };

        Ok(DeepLink {
            app: app.clone(),
            current: std::sync::Mutex::new(if let Some(url) = current {
                Some(vec![url])
            } else {
                None
            }),
            config: api.config().clone(),
        })
    }
}

#[cfg(target_os = "android")]
mod imp {
    use tauri::{plugin::PluginHandle, Runtime};

    use serde::{Deserialize, Serialize};
    use tauri::ipc::Channel;

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct EventHandler {
        pub handler: Channel,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetCurrentResponse {
        pub url: Option<url::Url>,
    }

    /// Access to the deep-link APIs.
    pub struct DeepLink<R: Runtime>(pub(crate) PluginHandle<R>);

    impl<R: Runtime> DeepLink<R> {
        /// Get the current URLs that triggered the deep link. Use this on app load to check whether your app was started via a deep link.
        pub fn get_current(&self) -> crate::Result<Option<Vec<url::Url>>> {
            self.0
                .run_mobile_plugin::<GetCurrentResponse>("getCurrent", ())
                .map(|v| v.url.map(|url| vec![url]))
                .map_err(Into::into)
        }

        /// Register the app as the default handler for the specified protocol.
        ///
        /// - `protocol`: The name of the protocol without `://`. For example, if you want your app to handle `tauri://` links, call this method with `tauri` as the protocol.
        ///
        /// ## Platform-specific:
        ///
        /// - **macOS / Android / iOS**: Unsupported, will return [`Error::UnsupportedPlatform`](`crate::Error::UnsupportedPlatform`).
        pub fn register<S: AsRef<str>>(&self, _protocol: S) -> crate::Result<()> {
            Err(crate::Error::UnsupportedPlatform)
        }

        /// Unregister the app as the default handler for the specified protocol.
        ///
        /// - `protocol`: The name of the protocol without `://`.
        ///
        /// ## Platform-specific:
        ///
        /// - **Linux**: Can only unregister the scheme if it was initially registered with [`register`](`Self::register`). May not work on older distros.
        /// - **macOS / Android / iOS**: Unsupported, will return [`Error::UnsupportedPlatform`](`crate::Error::UnsupportedPlatform`).
        pub fn unregister<S: AsRef<str>>(&self, _protocol: S) -> crate::Result<()> {
            Err(crate::Error::UnsupportedPlatform)
        }

        /// Check whether the app is the default handler for the specified protocol.
        ///
        /// - `protocol`: The name of the protocol without `://`.
        ///
        /// ## Platform-specific:
        ///
        /// - **macOS / Android / iOS**: Unsupported, will return [`Error::UnsupportedPlatform`](`crate::Error::UnsupportedPlatform`).
        pub fn is_registered<S: AsRef<str>>(&self, _protocol: S) -> crate::Result<bool> {
            Err(crate::Error::UnsupportedPlatform)
        }
    }
}

#[cfg(not(target_os = "android"))]
mod imp {
    use std::sync::Mutex;
    #[cfg(target_os = "linux")]
    use std::{
        fs::{create_dir_all, File},
        io::Write,
        process::Command,
    };
    #[cfg(target_os = "linux")]
    use tauri::Manager;
    use tauri::{AppHandle, Runtime};
    #[cfg(windows)]
    use windows_registry::CURRENT_USER;

    /// Access to the deep-link APIs.
    pub struct DeepLink<R: Runtime> {
        pub(crate) app: AppHandle<R>,
        pub(crate) current: Mutex<Option<Vec<url::Url>>>,
        pub(crate) config: Option<crate::config::Config>,
    }

    pub(crate) fn deep_link_from_args<S: AsRef<str>, I: Iterator<Item = S>>(
        config: &crate::config::Config,
        mut args: I,
    ) -> Option<url::Url> {
        if cfg!(windows) || cfg!(target_os = "linux") {
            args.next(); // bin name
            let arg = args.next();

            let maybe_deep_link = args.next().is_none(); // single argument
            if !maybe_deep_link {
                return None;
            }

            if let Some(url) = arg.and_then(|arg| arg.as_ref().parse::<url::Url>().ok()) {
                if config.desktop.contains_scheme(&url.scheme().to_string()) {
                    return Some(url);
                } else if cfg!(debug_assertions) {
                    log::warn!("argument {url} does not match any configured deep link scheme; skipping it");
                }
            }
        }

        None
    }

    impl<R: Runtime> DeepLink<R> {
        /// Checks if the provided list of arguments (which should match [`std::env::args`])
        /// contains a deep link argument (for Linux and Windows).
        ///
        /// On Linux and Windows the deep links trigger a new app instance with the deep link URL as its only argument.
        ///
        /// This function does what it can to verify if the argument is actually a deep link, though it could also be a regular CLI argument.
        /// To enhance its checks, we only match deep links against the schemes defined in the Tauri configuration
        /// i.e. dynamic schemes WON'T be processed.
        ///
        /// This function updates the [`Self::get_current`] value and emits a `deep-link://new-url` event.
        #[cfg(desktop)]
        #[doc(hidden)]
        pub fn handle_cli_arguments<S: AsRef<str>, I: Iterator<Item = S>>(&self, mut args: I) {
            use tauri::Emitter;

            let Some(config) = &self.config else {
                return;
            };

            if let Some(url) = deep_link_from_args(config, args) {
                let mut current = self.current.lock().unwrap();
                current.replace(vec![url.clone()]);
                let _ = self.app.emit("deep-link://new-url", vec![url]);
            }
        }

        /// Get the current URLs that triggered the deep link. Use this on app load to check whether your app was started via a deep link.
        ///
        /// ## Platform-specific:
        ///
        /// - **Windows / Linux**: This function reads the command line arguments and checks if there's only one value, which must be an URL with scheme matching one of the configured values.
        ///     Note that you must manually check the arguments when registering deep link schemes dynamically with [`Self::register`].
        ///     Additionally, the deep link might have been provided as a CLI argument so you should check if its format matches what you expect.
        pub fn get_current(&self) -> crate::Result<Option<Vec<url::Url>>> {
            return Ok(self.current.lock().unwrap().clone());
        }

        /// Registers all schemes defined in the configuration file.
        ///
        /// This is useful to ensure the schemes are registered even if the user unregistered it
        /// or did not install the app properly (e.g. an AppImage that was not properly registered with an AppImage launcher).
        pub fn register_all(&self) -> crate::Result<()> {
            let Some(config) = &self.config else {
                return Ok(());
            };

            for scheme in config.desktop.schemes() {
                self.register(scheme)?;
            }

            Ok(())
        }

        /// Register the app as the default handler for the specified protocol.
        ///
        /// - `protocol`: The name of the protocol without `://`. For example, if you want your app to handle `tauri://` links, call this method with `tauri` as the protocol.
        ///
        /// ## Platform-specific:
        ///
        /// - **macOS / Android / iOS**: Unsupported, will return [`Error::UnsupportedPlatform`](`crate::Error::UnsupportedPlatform`).
        pub fn register<S: AsRef<str>>(&self, _protocol: S) -> crate::Result<()> {
            #[cfg(windows)]
            {
                let key_base = format!("Software\\Classes\\{}", _protocol.as_ref());

                let exe = dunce::simplified(&tauri::utils::platform::current_exe()?)
                    .display()
                    .to_string();

                let key_reg = CURRENT_USER.create(&key_base)?;
                key_reg.set_string(
                    "",
                    &format!("URL:{} protocol", self.app.config().identifier),
                )?;
                key_reg.set_string("URL Protocol", "")?;

                let icon_reg = CURRENT_USER.create(format!("{key_base}\\DefaultIcon"))?;
                icon_reg.set_string("", &format!("{},0", &exe))?;

                let cmd_reg = CURRENT_USER.create(format!("{key_base}\\shell\\open\\command"))?;

                cmd_reg.set_string("", &format!("{} \"%1\"", &exe))?;

                Ok(())
            }

            #[cfg(target_os = "linux")]
            {
                let bin = tauri::utils::platform::current_exe()?;
                let file_name = format!(
                    "{}-handler.desktop",
                    bin.file_name().unwrap().to_string_lossy()
                );
                let appimage = self.app.env().appimage;
                let exec = appimage
                    .clone()
                    .unwrap_or_else(|| bin.into_os_string())
                    .to_string_lossy()
                    .to_string();

                let target = self.app.path().data_dir()?.join("applications");

                create_dir_all(&target)?;

                let target_file = target.join(&file_name);

                let mime_type = format!("x-scheme-handler/{}", _protocol.as_ref());

                if let Ok(mut desktop_file) = ini::Ini::load_from_file(&target_file) {
                    if let Some(section) = desktop_file.section_mut(Some("Desktop Entry")) {
                        let old_mimes = section.remove("MimeType");
                        section.append(
                            "MimeType",
                            format!("{mime_type};{}", old_mimes.unwrap_or_default()),
                        );
                        desktop_file.write_to_file(&target_file)?;
                    }
                } else {
                    let mut file = File::create(target_file)?;
                    file.write_all(
                        format!(
                            include_str!("template.desktop"),
                            name = self
                                .app
                                .config()
                                .product_name
                                .clone()
                                .unwrap_or_else(|| file_name.clone()),
                            exec = exec,
                            mime_type = mime_type
                        )
                        .as_bytes(),
                    )?;
                }

                Command::new("update-desktop-database")
                    .arg(target)
                    .status()?;

                Command::new("xdg-mime")
                    .args(["default", &file_name, _protocol.as_ref()])
                    .status()?;

                Ok(())
            }

            #[cfg(not(any(windows, target_os = "linux")))]
            Err(crate::Error::UnsupportedPlatform)
        }

        /// Unregister the app as the default handler for the specified protocol.
        ///
        /// - `protocol`: The name of the protocol without `://`.
        ///
        /// ## Platform-specific:
        ///
        /// - **Linux**: Can only unregister the scheme if it was initially registered with [`register`](`Self::register`). May not work on older distros.
        /// - **macOS / Android / iOS**: Unsupported, will return [`Error::UnsupportedPlatform`](`crate::Error::UnsupportedPlatform`).
        pub fn unregister<S: AsRef<str>>(&self, _protocol: S) -> crate::Result<()> {
            #[cfg(windows)]
            {
                CURRENT_USER.remove_tree(format!("Software\\Classes\\{}", _protocol.as_ref()))?;

                Ok(())
            }

            #[cfg(target_os = "linux")]
            {
                let mimeapps_path = self.app.path().config_dir()?.join("mimeapps.list");
                let mut mimeapps = ini::Ini::load_from_file(&mimeapps_path)?;

                let file_name = format!(
                    "{}-handler.desktop",
                    tauri::utils::platform::current_exe()?
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                );

                if let Some(section) = mimeapps.section_mut(Some("Default Applications")) {
                    let scheme = format!("x-scheme-handler/{}", _protocol.as_ref());

                    if section.get(&scheme).unwrap_or_default() == file_name {
                        section.remove(scheme);
                    }
                }

                mimeapps.write_to_file(mimeapps_path)?;

                Ok(())
            }

            #[cfg(not(any(windows, target_os = "linux")))]
            Err(crate::Error::UnsupportedPlatform)
        }

        /// Check whether the app is the default handler for the specified protocol.
        ///
        /// - `protocol`: The name of the protocol without `://`.
        ///
        /// ## Platform-specific:
        ///
        /// - **macOS / Android / iOS**: Unsupported, will return [`Error::UnsupportedPlatform`](`crate::Error::UnsupportedPlatform`).
        pub fn is_registered<S: AsRef<str>>(&self, _protocol: S) -> crate::Result<bool> {
            #[cfg(windows)]
            {
                let cmd_reg = CURRENT_USER.open(format!(
                    "Software\\Classes\\{}\\shell\\open\\command",
                    _protocol.as_ref()
                ))?;

                let registered_cmd: String = cmd_reg.get_string("")?;

                let exe = dunce::simplified(&tauri::utils::platform::current_exe()?)
                    .display()
                    .to_string();

                Ok(registered_cmd == format!("{} \"%1\"", &exe))
            }
            #[cfg(target_os = "linux")]
            {
                let file_name = format!(
                    "{}-handler.desktop",
                    tauri::utils::platform::current_exe()?
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                );

                let output = Command::new("xdg-mime")
                    .args([
                        "query",
                        "default",
                        &format!("x-scheme-handler/{}", _protocol.as_ref()),
                    ])
                    .output()?;

                Ok(String::from_utf8_lossy(&output.stdout).contains(&file_name))
            }

            #[cfg(not(any(windows, target_os = "linux")))]
            Err(crate::Error::UnsupportedPlatform)
        }
    }
}

pub use imp::DeepLink;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`], [`tauri::WebviewWindow`], [`tauri::Webview`] and [`tauri::Window`] to access the deep-link APIs.
pub trait DeepLinkExt<R: Runtime> {
    fn deep_link(&self) -> &DeepLink<R>;
}

impl<R: Runtime, T: Manager<R>> crate::DeepLinkExt<R> for T {
    fn deep_link(&self) -> &DeepLink<R> {
        self.state::<DeepLink<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R, Option<config::Config>> {
    Builder::new("deep-link")
        .invoke_handler(tauri::generate_handler![
            commands::get_current,
            commands::register,
            commands::unregister,
            commands::is_registered
        ])
        .setup(|app, api| {
            app.manage(init_deep_link(app, api)?);
            Ok(())
        })
        .on_event(|_app, _event| {
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            if let tauri::RunEvent::Opened { urls } = _event {
                use tauri::Emitter;

                let _ = _app.emit("deep-link://new-url", urls);
                _app.state::<DeepLink<R>>()
                    .current
                    .lock()
                    .unwrap()
                    .replace(urls.clone());
            }
        })
        .build()
}
