#![cfg(target_os = "linux")]

use crate::SingleInstanceCallback;
use tauri::{
    AppHandle,
    plugin::{self, TauriPlugin},
    Manager, RunEvent, Runtime,
};
use zbus::{
    blocking::{Connection, ConnectionBuilder},
    dbus_interface,
};

struct ConnectionHandle(Connection);

struct SingleInstanceDBus<R: Runtime> {
    callback: Box<SingleInstanceCallback<R>>,
    app_handle: AppHandle<R>,
}

#[dbus_interface(name = "org.SingleInstance.DBus")]
impl<R:Runtime> SingleInstanceDBus<R> {
    fn execute_callback(&mut self, argv: Vec<String>, cwd: String) {
        (self.callback)(&self.app_handle, argv, cwd);
    }
}

pub fn init<R: Runtime>(f: Box<SingleInstanceCallback<R>>) -> TauriPlugin<R> {
    plugin::Builder::new("single-instance")
        .setup(|app| {
            let app_name = app.package_info().name.clone();
            let single_instance_dbus = SingleInstanceDBus {
                callback: f,
                app_handle: app.clone(),
            };
            let dbus_name = format!("org.{}.SingleInstance", app_name);
            let dbus_path = format!("/org/{}/SingleInstance", app_name);

            match ConnectionBuilder::session()
                .unwrap()
                .name(dbus_name.as_str())
                .unwrap()
                .serve_at(dbus_path.as_str(), single_instance_dbus)
                .unwrap()
                .build()
            {
                Ok(connection) => {
                    app.manage(ConnectionHandle(connection));
                }
                Err(zbus::Error::NameTaken) => {
                    if let Ok(connection) = Connection::session() {
                        let _ = connection.call_method(
                            Some(dbus_name.as_str()),
                            dbus_path.as_str(),
                            Some("org.SingleInstance.DBus"),
                            "ExecuteCallback",
                            &(
                                std::env::args().collect::<Vec<String>>(),
                                std::env::current_dir()
                                    .unwrap_or_default()
                                    .to_str()
                                    .unwrap_or_default(),
                            ),
                        );
                    }
                    std::process::exit(0)
                }
                _ => {}
            }

            Ok(())
        })
        .on_event(|app, event| {
            if let RunEvent::Exit = event {
                if let Some(connection) = app.try_state::<ConnectionHandle>() {
                    let app_name = app.package_info().name.clone();
                    let dbus_name = format!("org.{}.SingleInstance", app_name);
                    let _ = connection.0.release_name(dbus_name);
                }
            }
        })
        .build()
}
