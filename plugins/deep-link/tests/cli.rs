// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg(any(windows, target_os = "linux", target_os = "freebsd"))]

use std::{sync::mpsc, time::Duration};
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri_plugin_deep_link::DeepLinkExt;
use url::Url;

fn build_app(config: Option<serde_json::Value>) -> tauri::App<MockRuntime> {
    let mut context = mock_context::<MockRuntime, _>(noop_assets());
    if let Some(config) = config {
        context
            .config_mut()
            .plugins
            .0
            .insert("deep-link".into(), config);
    }
    mock_builder()
        .plugin(tauri_plugin_deep_link::init())
        .build(context)
        .unwrap()
}

#[test]
fn configured_cli_urls_update_current_and_emit_one_event() {
    for desktop in [
        serde_json::json!({ "schemes": ["tauri-test"] }),
        serde_json::json!([{ "schemes": ["other"] }, { "schemes": ["tauri-test"] }]),
    ] {
        let app = build_app(Some(serde_json::json!({ "desktop": desktop })));
        let deep_link = app.deep_link();
        let (tx, rx) = mpsc::channel();
        deep_link.on_open_url(move |event| tx.send(event.urls()).unwrap());

        for raw in ["tauri-test://first/path?value=1", "tauri-test://second"] {
            deep_link.handle_cli_arguments(["test-app", raw].into_iter());
            let expected = vec![Url::parse(raw).unwrap()];
            assert_eq!(deep_link.get_current().unwrap(), Some(expected.clone()));
            assert_eq!(rx.recv_timeout(Duration::from_secs(1)).unwrap(), expected);
            assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
        }
    }
}

#[test]
fn rejected_cli_arguments_preserve_current_without_emitting() {
    let app = build_app(Some(serde_json::json!({
        "desktop": { "schemes": ["tauri-test"] }
    })));
    let deep_link = app.deep_link();
    let initial = "tauri-test://initial";
    deep_link.handle_cli_arguments(["test-app", initial].into_iter());
    let expected = Some(vec![Url::parse(initial).unwrap()]);
    assert_eq!(deep_link.get_current().unwrap(), expected);

    let (tx, rx) = mpsc::channel();
    deep_link.on_open_url(move |event| tx.send(event.urls()).unwrap());
    for args in [
        vec![],
        vec!["test-app"],
        vec!["test-app", "not a URL"],
        vec!["test-app", "unconfigured://value"],
        vec!["test-app", "tauri-test://value", "--extra"],
    ] {
        deep_link.handle_cli_arguments(args.into_iter());
        assert_eq!(deep_link.get_current().unwrap(), expected);
        assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    }
}

#[test]
fn missing_configuration_or_schemes_do_not_accept_cli_urls() {
    for config in [None, Some(serde_json::json!({}))] {
        let app = build_app(config);
        let deep_link = app.deep_link();
        let (tx, rx) = mpsc::channel();
        deep_link.on_open_url(move |event| tx.send(event.urls()).unwrap());
        deep_link.handle_cli_arguments(["test-app", "tauri-test://value"].into_iter());
        assert_eq!(deep_link.get_current().unwrap(), None);
        assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    }
}
