#![cfg(target_os = "macos")]

use nix::fcntl::{open, OFlag};
use nix::sys::stat;
use nix::unistd::{mkfifo, unlink, write};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::Arc;
use tokio;

use crate::SingleInstanceCallback;
use tauri::{
    plugin::{self, TauriPlugin},
    AppHandle, Config, Manager, RunEvent, Runtime,
};

fn fifo_path(config: Arc<Config>) -> PathBuf {
    let identifier = config
        .tauri
        .bundle
        .identifier
        .replace(['.', '-'].as_ref(), "_");
    let data_dir = tauri::api::path::app_local_data_dir(&config);
    match data_dir {
        Some(mut p) => {
            p.push(identifier);
            p
        }
        None => PathBuf::from(format!("/tmp/{}_single_instance_fifo", identifier)),
    }
}

fn bootup_reader<A: Runtime>(
    path: &PathBuf,
    app: AppHandle<A>,
    mut f: Box<SingleInstanceCallback<A>>,
) {
    let app_hand = app.app_handle();
    let inner_path = path.clone();
    let cwd = std::env::current_dir()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_string();
    tokio::task::spawn(async move {
        loop {
            let file = File::open(&inner_path).unwrap();
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line.unwrap();
                let args_vec: Vec<String> = line.split('\0').map(String::from).collect();

                // Here `line` contains the message from another instance.
                // You can now execute your callback.
                f(&app_hand, args_vec, cwd.clone());
            }
        }
    });
}

pub fn init<R: Runtime>(mut f: Box<SingleInstanceCallback<R>>) -> TauriPlugin<R> {
    plugin::Builder::new("single-instance")
        .setup(move |app| {
            let path = fifo_path(app.config());
            match mkfifo(&path, stat::Mode::S_IRWXU) {
                Ok(_) => {
                    // create and open the FIFO, then listen for messages
                    bootup_reader::<R>(&path, app.clone(), f);
                }
                Err(nix::Error::EEXIST) => {
                    // FIFO already exists, send a message to it
                    let fdo = match open(
                        &path,
                        OFlag::O_WRONLY | OFlag::O_NONBLOCK,
                        stat::Mode::empty(),
                    ) {
                        Ok(f) => Some(f),
                        Err(nix::Error::ENXIO) => {
                            // FIFO exists, but no one is reading it, so we're actually the only
                            // ones alive.
                            None
                        }
                        Err(_) => {
                            // There's some other error, and we bias towards launching.
                            None
                        }
                    };

                    // Attempt to write to the FIFO to the FIFO
                    if let Some(fd) = fdo {
                        let args_joined = std::env::args().collect::<Vec<String>>().join("\0");
                        let args_bytes = args_joined.as_bytes();

                        match write(fd, args_bytes) {
                            Ok(_) => {
                                // Write succeeded, another instance is running and reading from the FIFO
                                std::process::exit(0);
                            }
                            Err(nix::Error::EAGAIN) => {
                                // This should never actually happen, since we should catch "no reader" on open.
                                bootup_reader(&path, app.clone(), f);
                            }
                            Err(err) => {
                                // There's some other error, and we bias towards launching.
                                bootup_reader(&path, app.clone(), f);
                            }
                        }
                    } else {
                        bootup_reader(&path, app.clone(), f);
                    }
                }
                Err(err) => {
                    // There's some other error, and we bias towards launching.
                    // Unfortunately the error was in creating the FIFO, so no reader.
                    eprintln!("An error occurred launching single-instance: {}", err);
                }
            }
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
    let path = fifo_path(manager.config());
    let _ = unlink(&path);
}
