use super::*;
use serde_json::{json, Value};

const PUBLIC_KEY: &str = "untrusted comment: minisign public key E7620F1842B4E81F\nRWQf6LRCGA9i53mlYecO4IzT51TGPpvWucNSCh1CBM0QTaLn73Y7GFO3";
const SIGNATURE: &str = "untrusted comment: signature from minisign secret key\nRUQf6LRCGA9i559r3g7V1qNyJDApGip8MfqcadIgT9CuhV3EMhHoN1mGTkUidF/z7SrlQgXdy8ofjb7bNJJylDOocrCo8KLzZwo=\ntrusted comment: timestamp:1633700835\tfile:test\tprehashed\nwLMDjy9FLAuxZ3q4NlEvkgtyhrr0gtTu6KC4KBJdITbbOeAi1zBIYo0v4iTgt8jJpIidRJnp94ABQkJAgAooBQ==";

fn encode(value: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(value)
}

fn updater() -> Updater {
    Updater {
        current_version: Version::new(1, 0, 0),
        version_comparator: None,
        timeout: Some(Duration::from_secs(30)),
        proxy: None,
        no_proxy: true,
        endpoints: vec![],
        arch: std::env::consts::ARCH,
        target: Some("test-target".into()),
        headers: HeaderMap::new(),
        extract_path: PathBuf::from("runtime-path"),
        context: UpdaterContext {
            config: Config {
                pubkey: encode(PUBLIC_KEY),
                ..Default::default()
            },
            configure_client: Some(Arc::new(|_| panic!("unexpected network request"))),
            #[cfg(target_os = "macos")]
            run_on_main_thread: Arc::new(|_| panic!("unexpected main thread dispatch")),
            #[cfg(windows)]
            app_name: "Updater test".into(),
            #[cfg(windows)]
            installer_args: vec![],
            #[cfg(windows)]
            current_exe_args: vec![],
            #[cfg(windows)]
            on_before_exit: None,
            #[cfg(windows)]
            restart_after_install: true,
        },
    }
}

fn manifest() -> Value {
    json!({
        "version": "2.0.0",
        "notes": "Cached release notes",
        "pub_date": "2025-01-01T00:00:00Z",
        "platforms": {
            "test-target": {
                "url": "https://example.invalid/package",
                "signature": encode(SIGNATURE)
            }
        }
    })
}

#[test]
fn restores_release_metadata_with_runtime_context() {
    let mut updater = updater();
    updater
        .headers
        .insert("x-test", HeaderValue::from_static("runtime"));
    let mut metadata = manifest();
    metadata["extract_path"] = json!("untrusted-path");
    metadata["config"] = json!({ "pubkey": "untrusted-key" });
    let update = updater.restore_update(metadata.clone()).unwrap().unwrap();

    assert_eq!(update.raw_json, metadata);
    assert_eq!(update.current_version, "1.0.0");
    assert_eq!(update.version, "2.0.0");
    assert_eq!(update.body.as_deref(), Some("Cached release notes"));
    assert!(update.date.is_some());
    assert_eq!(update.target, "test-target");
    assert_eq!(update.extract_path, updater.extract_path);
    assert_eq!(update.context.config.pubkey, updater.context.config.pubkey);
    assert_eq!(update.headers, updater.headers);
    assert!(update.no_proxy);
    assert!(update.timeout.is_none());
    update.verify(b"test").unwrap();
}

#[test]
fn restores_dynamic_manifests() {
    let update = updater()
        .restore_update(json!({
            "version": "2.0.0",
            "url": "https://example.invalid/dynamic-package",
            "signature": encode(SIGNATURE)
        }))
        .unwrap()
        .unwrap();
    assert_eq!(
        update.download_url.as_str(),
        "https://example.invalid/dynamic-package"
    );
    update.verify(b"test").unwrap();
}

#[test]
fn respects_default_and_custom_version_comparators() {
    let mut updater = updater();
    for version in ["0.9.0", "1.0.0"] {
        let mut metadata = manifest();
        metadata["version"] = json!(version);
        assert!(updater.restore_update(metadata).unwrap().is_none());
    }
    updater.version_comparator = Some(Arc::new(|_, _| false));
    assert!(updater.restore_update(manifest()).unwrap().is_none());
    updater.version_comparator = Some(Arc::new(|current, release| release.version != current));
    let mut metadata = manifest();
    metadata["version"] = json!("0.9.0");
    assert!(updater.restore_update(metadata).unwrap().is_some());
}

#[test]
fn rejects_malformed_and_incompatible_manifests() {
    let updater = updater();
    assert!(updater.restore_update(json!({})).is_err());
    let mut metadata = manifest();
    metadata["platforms"] = json!({});
    assert!(matches!(
        updater.restore_update(metadata),
        Err(Error::TargetNotFound(_))
    ));
    let mut metadata = manifest();
    metadata["version"] = json!("not-a-version");
    assert!(updater.restore_update(metadata).is_err());
}

#[test]
fn verifies_cached_bytes_and_rejects_tampering() {
    let mut update = updater().restore_update(manifest()).unwrap().unwrap();
    update.verify(b"test").unwrap();
    assert!(update.verify(b"tampered").is_err());
    update.signature = encode(&SIGNATURE.replace("file:test", "file:other"));
    assert!(update.verify(b"test").is_err());
    update.signature = "invalid signature".into();
    assert!(update.verify(b"test").is_err());
    update.signature = encode(SIGNATURE);
    update.context.config.pubkey = encode(&PUBLIC_KEY.replace("RWQf", "RWQe"));
    assert!(update.verify(b"test").is_err());
}

#[test]
fn preserves_signed_version_requirement() {
    let mut updater = updater();
    updater.context.config.require_signed_version = true;
    let mut metadata = manifest();
    metadata["config"] = json!({ "requireSignedVersion": false });
    let update = updater.restore_update(metadata).unwrap().unwrap();
    assert!(matches!(
        update.verify(b"test"),
        Err(Error::MissingSignedVersion)
    ));
}

#[test]
fn online_and_restored_updates_use_the_same_metadata() {
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let metadata = manifest();
    let response = metadata.to_string();
    let server = std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut reader = BufReader::new(&mut socket);
        loop {
            let mut line = String::new();
            assert!(reader.read_line(&mut line).unwrap() > 0);
            if line == "\r\n" {
                break;
            }
        }
        write!(socket, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", response.len(), response).unwrap();
    });
    let mut updater = updater();
    updater.context.configure_client = None;
    updater.endpoints = vec![format!("http://{address}").parse().unwrap()];
    let online = tauri::async_runtime::block_on(updater.check())
        .unwrap()
        .unwrap();
    server.join().unwrap();
    let restored = updater
        .restore_update(online.raw_json.clone())
        .unwrap()
        .unwrap();
    assert_eq!(restored.raw_json, metadata);
    assert_eq!(restored.version, online.version);
    assert_eq!(restored.current_version, online.current_version);
    assert_eq!(restored.target, online.target);
    assert_eq!(restored.download_url, online.download_url);
    assert_eq!(restored.signature, online.signature);
    assert_eq!(restored.extract_path, online.extract_path);
    assert_eq!(restored.timeout, online.timeout);
}
