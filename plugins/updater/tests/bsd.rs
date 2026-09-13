// Copyright 2019-2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]

use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
    time::{Duration, Instant},
};
use tauri::test::{mock_builder, mock_context, noop_assets};
use tauri_plugin_updater::UpdaterExt;

#[test]
fn custom_target_checks_metadata_without_installing() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let address = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(Instant::now() < deadline, "updater request timed out");
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("accept updater request: {error}"),
            }
        };
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut request = Vec::new();
        while !request.ends_with(b"\r\n\r\n") {
            let mut byte = [0];
            stream.read_exact(&mut byte).unwrap();
            request.push(byte[0]);
            assert!(request.len() < 8192);
        }
        assert!(String::from_utf8(request)
            .unwrap()
            .starts_with("GET /updates/freebsd-x86_64 HTTP/1.1\r\n"));
        let body = serde_json::json!({
            "version": "999.0.0",
            "notes": "Install this release with the package manager.",
            "platforms": {
                "freebsd-x86_64": {
                    "url": "https://example.com/app.pkg",
                    "signature": "metadata-only-test"
                }
            }
        })
        .to_string();
        write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
    });

    let mut context = mock_context(noop_assets());
    context.config_mut().plugins = serde_json::from_value(serde_json::json!({
        "updater": {
            "pubkey": "",
            "dangerousInsecureTransportProtocol": true,
            "endpoints": [format!("http://{address}/updates/{{{{target}}}}")]
        }
    }))
    .unwrap();
    let app = mock_builder()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .build(context)
        .unwrap();
    let destination = tempfile::tempdir().unwrap();
    let updater = app
        .updater_builder()
        .target("freebsd-x86_64")
        .executable_path(destination.path().join("app"))
        .timeout(Duration::from_secs(5))
        .no_proxy()
        .build()
        .unwrap();
    let update = tauri::async_runtime::block_on(updater.check())
        .unwrap()
        .expect("custom target must expose available release metadata");
    server.join().unwrap();
    assert_eq!(update.target, "freebsd-x86_64");
    assert_eq!(update.version, "999.0.0");
    assert_eq!(update.download_url.as_str(), "https://example.com/app.pkg");
    assert_eq!(
        update.body.as_deref(),
        Some("Install this release with the package manager.")
    );

    #[cfg(not(target_os = "linux"))]
    {
        assert!(matches!(
            update.install(b"not a Linux package"),
            Err(tauri_plugin_updater::Error::UnsupportedOs)
        ));
        assert!(matches!(
            tauri::async_runtime::block_on(app.updater().unwrap().check()),
            Err(tauri_plugin_updater::Error::UnsupportedOs)
        ));
        assert_eq!(tauri_plugin_updater::target(), None);
    }
    assert_eq!(std::fs::read_dir(destination.path()).unwrap().count(), 0);
}
