#![cfg(target_os = "macos")]

use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::Arc;

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
    let exe_path = current_exe();
    match exe_path {
        Ok(mut p) => {
            if p.is_file() {
                p.pop();
                p.push(identifier);
            } else {
                p.push(identifier);
            }
            p
        }
        Err(_) => PathBuf::from(format!("/tmp/{}_single_instance_fifo", identifier)),
    }
}

pub fn init<R: Runtime>(mut f: Box<SingleInstanceCallback<R>>) -> TauriPlugin<R> {
    plugin::Builder::new("single-instance")
        .setup(move |app| {
            let path = fifo_path(app.config());
            match mkfifo(&path, stat::Mode::S_IRWXU) {
                Ok(_) => {
                    // create and open the FIFO, then listen for messages
                    let file = File::open(&path).unwrap();
                    let reader = BufReader::new(file);
                    for line in reader.lines() {
                        let line = line.unwrap();
                        // Here `line` contains the message from another instance.
                        // You can now execute your callback.
                        f(&app, vec![line.clone()], String::new());
                    }
                }
                Err(nix::Error::EEXIST) => {
                    // FIFO already exists, send a message to it
                    let fd = open(
                        &path,
                        OFlag::O_WRONLY | OFlag::O_NONBLOCK,
                        stat::Mode::empty(),
                    )
                    .unwrap();

                    // Attempt to write
                    match write(fd, b"Another instance is launched") {
                        Ok(_) => {
                            // Write succeeded, another instance is running and reading from the FIFO
                            std::process::exit(0);
                        }
                        Err(nix::Error::EAGAIN) | Err(nix::Error::EWOULDBLOCK) => {
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
