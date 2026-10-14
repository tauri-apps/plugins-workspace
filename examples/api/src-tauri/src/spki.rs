// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! SPKI pinning for the http plugin, and the local HTTPS servers the e2e suite
//! (packages/api-e2e) checks it against.
//!
//! A throwaway CA issues two `localhost` certificates with different keys. Only
//! the first key is pinned, so `https://localhost:3005` (pinned key) is
//! accepted and `https://localhost:3006` (other key) is rejected even though
//! its chain is valid. Other hosts and plain HTTP keep working as before.

use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    sync::Arc,
};

use base64::{Engine, engine::general_purpose::STANDARD};
use rcgen::{BasicConstraints, CertificateParams, IsCa, KeyPair};
use sha2::{Digest, Sha256};
use tauri_plugin_http::rustls::{
    self, RootCertStore, ServerConfig, ServerConnection, StreamOwned,
    pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer},
};

const PINNED_ADDR: &str = "localhost:3005";
const UNPINNED_KEY_ADDR: &str = "localhost:3006";

/// Returns the pinning extension and starts the HTTPS servers it is tested against.
pub fn pinning() -> tauri_plugin_http::SpkiPinning {
    let ca_key = KeyPair::generate().expect("failed to generate the test CA key");
    let mut ca_params = CertificateParams::new(Vec::<String>::new()).unwrap();
    ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    let ca = ca_params
        .self_signed(&ca_key)
        .expect("failed to create the test CA");

    let leaf = |key: &KeyPair| -> CertificateDer<'static> {
        CertificateParams::new(vec!["localhost".to_string()])
            .unwrap()
            .signed_by(key, &ca, &ca_key)
            .expect("failed to issue a test certificate")
            .der()
            .clone()
    };
    let pinned_key = KeyPair::generate().expect("failed to generate a test key");
    let other_key = KeyPair::generate().expect("failed to generate a test key");

    serve(PINNED_ADDR, leaf(&pinned_key), &pinned_key);
    serve(UNPINNED_KEY_ADDR, leaf(&other_key), &other_key);

    // Mozilla's roots for every other host, plus the test CA
    let mut roots = RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    };
    roots
        .add(ca.der().clone())
        .expect("failed to trust the test CA");

    let pin = format!(
        "sha256/{}",
        STANDARD.encode(Sha256::digest(pinned_key.public_key_der()))
    );
    tauri_plugin_http::SpkiPinning::new()
        .pin("localhost", pin)
        .expect("invalid test pin")
        .root_certificates(roots)
        .allow_unpinned_hosts()
        .allow_http()
}

/// Serves HTTPS on `addr`: `/redirect` redirects to the server with the
/// unpinned key, any other path answers `200 pinned`.
fn serve(addr: &'static str, cert: CertificateDer<'static>, key: &KeyPair) {
    let config =
        ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_no_client_auth()
            .with_single_cert(
                vec![cert],
                PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key.serialize_der())),
            )
            .expect("invalid test certificate");
    let config = Arc::new(config);

    std::thread::spawn(move || {
        let listener = match TcpListener::bind(addr) {
            Ok(listener) => listener,
            Err(e) => {
                eprintln!("failed to bind {addr}: {e}");
                return;
            }
        };
        for stream in listener.incoming().flatten() {
            let config = config.clone();
            std::thread::spawn(move || {
                let Ok(connection) = ServerConnection::new(config) else {
                    return;
                };
                let mut stream = BufReader::new(StreamOwned::new(connection, stream));

                // a failed handshake (the client rejected our key) surfaces here
                let mut request_line = String::new();
                if stream.read_line(&mut request_line).is_err() {
                    return;
                }
                let mut line = String::new();
                while stream.read_line(&mut line).is_ok_and(|n| n > 2) {
                    line.clear();
                }

                let response = if request_line.starts_with("GET /redirect ") {
                    format!(
                        "HTTP/1.1 302 Found\r\nLocation: https://{UNPINNED_KEY_ADDR}/\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    )
                } else {
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 6\r\nConnection: close\r\n\r\npinned".to_string()
                };
                let stream = stream.get_mut();
                let _ = stream.write_all(response.as_bytes());
                stream.conn.send_close_notify();
                let _ = stream.flush();
            });
        }
    });
}
