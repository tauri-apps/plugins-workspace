#![cfg(target_os = "macos")]

use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::Arc;
use tokio;

use nix::errno::Errno;
use nix::fcntl::{fcntl, open, FcntlArg, OFlag};
use nix::sys::stat;
use nix::unistd::{mkfifo, unlink, write};

use crate::SingleInstanceCallback;
use tauri::{
    plugin::{self, TauriPlugin},
    utils::platform::current_exe,
    AppHandle, Config, Manager, RunEvent, Runtime,
};

struct FifoHandle;

fn fifo_path(config: Arc<Config>) -> PathBuf {
    let identifier = config
        .tauri
        .bundle
        .identifier
        .replace(['.', '-'].as_ref(), "_");
    let data_dir = tauri::api::path::app_local_data_dir(&config);
    match data_dir {
        Some(mut p) => {
            if p.is_file() {
                p.pop();
                p.push(identifier);
            } else {
                p.push(identifier);
            }
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
    let file = File::open(&path).unwrap();
    let reader = BufReader::new(file);
    let app_hand = app.app_handle();
    tokio::task::spawn_blocking(move || {
        for line in reader.lines() {
            let line = line.unwrap();
            // Here `line` contains the message from another instance.
            // You can now execute your callback.
            f(&app_hand, vec![line.clone()], String::new());
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
                            // Fifo exists, but no one is reading it
                            println!(
                                "Detected FIFO, but no read on other side, continuing with launch"
                            );

                            None
                        }
                        Err(_) => {
                            println!("other error");
                            None
                        }
                    };

                    // Attempt to write
                    if let Some(fd) = fdo {
                        match write(fd, b"Another instance is launched") {
                            Ok(_) => {
                                // Write succeeded, another instance is running and reading from the FIFO
                                std::process::exit(0);
                            }
                            Err(nix::Error::EAGAIN) | Err(nix::Error::EWOULDBLOCK) => {
                                // This should never actually happen, since we should catch it earlier.
                                bootup_reader(&path, app.clone(), f);

                                // Write would block, no other instance is reading from the FIFO
                                println!(
                                "Detected FIFO, but no read on other side, continuing with launch"
                            );
                                // Continue launching this instance
                            }
                            Err(err) => {
                                eprintln!("An error occurred while writing to the FIFO: {}", err);
                            }
                        }
                    } else {
                        bootup_reader(&path, app.clone(), f);
                    }
                }
                Err(err) => {
                    eprintln!("An error occurred: {}", err);
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
