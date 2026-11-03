// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// The cleanup hook held the stores lock while it locked the resource table,
// closing a store takes them the other way round.

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};

use tauri::test::{mock_builder, mock_context, noop_assets};
use tauri_plugin_store::{StoreBuilder, StoreExt};

#[test]
fn closing_a_store_during_cleanup_does_not_deadlock() {
    let dir = std::env::temp_dir().join(format!("store-cleanup-race-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (done, finished) = mpsc::channel();

    let worker = {
        let dir = dir.clone();
        thread::spawn(move || {
            for _ in 0..50 {
                let app = mock_builder()
                    .plugin(tauri_plugin_store::Builder::new().build())
                    .build(mock_context(noop_assets()))
                    .unwrap();
                // many stores keep the cleanup hook busy long enough to race with the closer
                for i in 0..100 {
                    StoreBuilder::new(&app, dir.join(format!("{i}.json")))
                        .build()
                        .unwrap();
                }

                let stop = Arc::new(AtomicBool::new(false));
                let closer = {
                    let app = app.handle().clone();
                    let stop = stop.clone();
                    let path = dir.join("0.json");
                    thread::spawn(move || {
                        while !stop.load(Ordering::Relaxed) {
                            if let Some(store) = app.get_store(&path) {
                                store.close_resource();
                            }
                            let _ = StoreBuilder::new(&app, &path).build();
                        }
                    })
                };

                app.cleanup_before_exit();
                stop.store(true, Ordering::Relaxed);
                closer.join().unwrap();
            }
            done.send(()).unwrap();
        })
    };

    let result = finished.recv_timeout(Duration::from_secs(60));
    let _ = std::fs::remove_dir_all(&dir);
    result.expect("closing a store during cleanup deadlocked");
    worker.join().unwrap();
}
