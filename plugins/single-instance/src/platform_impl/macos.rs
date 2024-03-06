// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg(target_os = "macos")]

use std::{
    io::{BufWriter, Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
};

use crate::SingleInstanceCallback;
use tauri::{
    plugin::{self, TauriPlugin},
    AppHandle, Config, Manager, RunEvent, Runtime,
};

enum ErrorKind {
    Retry,
    Abort,
}

pub fn init<R: Runtime>(cb: Box<SingleInstanceCallback<R>>) -> TauriPlugin<R> {
    plugin::Builder::new("single-instance")
        .setup(|app, _api| {
            start(app, cb, false);
            Ok(())
        })
        .on_event(|app, event| {
            if let RunEvent::Exit = event {
                destroy(app);
            }
        })
        .build()
}

pub fn destroy<R: Runtime, M: Manager<R>>(manager: &M) {
    let path = socket_path(manager.config());
    socket_cleanup(path);
}

fn start<R: Runtime>(app: &AppHandle<R>, cb: Box<SingleInstanceCallback<R>>, retry: bool) {
    let socket = socket_path(app.config());

    match UnixListener::bind(&socket) {
        Ok(listener) => {
            // This process is the singleton.
            // -> Listen for launches of other instances.
            listen_for_other_instances(listener, app.clone(), cb);
        }
        Err(err) => {
            match err.raw_os_error() {
                // Err: Address already in use
                Some(48) => {
                    if !retry {
                        // Another process already is likely the singleton.
                        // -> Notify singleton of launch and exit.
                        match notify_singleton(socket) {
                            Ok(_) => {
                                std::process::exit(0);
                            }
                            Err(ErrorKind::Retry) => {
                                start(app, cb, true);
                            }
                            Err(ErrorKind::Abort) => {
                                eprintln!("single_instance failed - launching normally");
                            }
                        }
                    } else {
                        eprintln!(
                            "single_instance failed - launching normally: address already in use"
                        );
                    }
                }
                Some(e) => {
                    // Unhandled OS error -> Try to launch instead.
                    eprintln!("single_instance failed - launching normally: {}", e);
                }
                None => {
                    // Not an OS error -> Try to launch instead
                    eprintln!("single_instance failed - launching normally: {}", err);
                }
            }
        }
    };
}

fn listen_for_other_instances<A: Runtime>(
    listener: UnixListener,
    app: AppHandle<A>,
    mut cb: Box<SingleInstanceCallback<A>>,
) {
    let cwd = std::env::current_dir()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_string();

    tauri::async_runtime::spawn(async move {
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut s = String::new();
                    match stream.read_to_string(&mut s) {
                        Ok(_) => {
                            let args: Vec<String> = s.split('\0').map(String::from).collect();
                            cb(&app.clone().app_handle(), args, cwd.clone());
                        }
                        Err(e) => println!("Error reading stream: {e}"),
                    }
                }
                Err(err) => {
                    eprintln!("single_instance notify failed: {}", err);
                    continue;
                }
            }
        }
    });
}

fn notify_singleton(socket: PathBuf) -> Result<(), ErrorKind> {
    match UnixStream::connect(&socket) {
        Ok(stream) => {
            let mut bf = BufWriter::new(&stream);
            let args_joined = std::env::args().collect::<Vec<String>>().join("\0");

            if let Err(_e) = bf.write_all(args_joined.as_bytes()) {
                return Err(ErrorKind::Abort);
            }
            if let Err(_e) = bf.flush() {
                return Err(ErrorKind::Abort);
            }

            drop(bf);
            Ok(())
        }
        Err(e) => {
            match e.raw_os_error() {
                Some(61) => {
                    // An old singleton likely did not clean up socket properly.
                    // -> Delete old socket and retry.
                    socket_cleanup(socket);
                    Err(ErrorKind::Retry)
                }
                Some(_) => {
                    // Unexpected OS error -> Try to launch instead.
                    Err(ErrorKind::Abort)
                }
                None => {
                    // Not an OS error -> Try to launch instead
                    Err(ErrorKind::Abort)
                }
            }
        }
    }
}

fn socket_path(config: &Config) -> PathBuf {
    let identifier = config.identifier.replace(['.', '-'].as_ref(), "_");
    // Use /tmp as socket path must be shorter than 100 chars.
    PathBuf::from(format!("/tmp/{}_si.sock", identifier))
}

fn socket_cleanup(socket: PathBuf) {
    let _ = std::fs::remove_file(socket).unwrap();
}
