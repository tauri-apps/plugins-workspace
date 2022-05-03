#![cfg(target_os = "linux")]

use crate::SingleInstanceCallback;
use std::{cell::RefCell, rc::Rc};
use tauri::{
    plugin::{self, TauriPlugin},
    Manager, RunEvent, Runtime,
};
use zbus::{
    blocking::{Connection, ConnectionBuilder},
    dbus_interface,
};

struct ConnectionHandle(Connection);
const CLOSE_NEW_INSTANCE_ID: u32 = 1542;

struct SingleInstanceDBus {
    callback: Box<SingleInstanceCallback>,
}

#[dbus_interface(name = "org.SingleInstance.DBus")]
impl SingleInstanceDBus {
    fn execute_callback(&mut self, argv: Vec<String>, cwd: String) -> u32 {
        let ret = Rc::new(RefCell::new(1));
        let ret_c = Rc::clone(&ret);

        (self.callback)(
            argv,
            cwd,
            Box::new(move || {
                let mut ret = ret_c.borrow_mut();
                *ret = CLOSE_NEW_INSTANCE_ID;
            }),
        );
        ret.take()
    }
}

pub fn init<R: Runtime>(f: Box<SingleInstanceCallback>) -> TauriPlugin<R> {
    plugin::Builder::new("single-instance")
        .setup(|app| {
            let app_name = app.package_info().name.clone();
            let single_instance_dbus = SingleInstanceDBus { callback: f };
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
                    let connection = Connection::session().unwrap();
                    if let Ok(m) = connection.call_method(
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
                    ) {
                        let reply: u32 = m.body().unwrap_or_default();
                        if reply == CLOSE_NEW_INSTANCE_ID {
                            std::process::exit(0);
                        }
                    }
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
