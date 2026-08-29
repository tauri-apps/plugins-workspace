// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{ExtensionError, HttpTransportExtension, RequestContext};
use base64::{engine::general_purpose::STANDARD, Engine};
use rustls::{
    client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    pki_types::{CertificateDer, ServerName, UnixTime},
    DigitallySignedStruct, Error as RustlsError, RootCertStore, SignatureScheme,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

const ERROR_CODE_PIN_MISMATCH: &str = "SPKI_PIN_MISMATCH";
const ERROR_CODE_HOST_NOT_PINNED: &str = "SPKI_HOST_NOT_PINNED";
const ERROR_CODE_HTTP_NOT_ALLOWED: &str = "SPKI_HTTP_NOT_ALLOWED";
const ERROR_CODE_DANGEROUS_SETTINGS: &str = "SPKI_DANGEROUS_SETTINGS";
const ERROR_CODE_INITIALIZATION: &str = "SPKI_INITIALIZATION_FAILED";
const RUSTLS_PIN_MISMATCH_MARKER: &str = "TAURI_HTTP_SPKI_PIN_MISMATCH:";
const RUSTLS_HOST_NOT_PINNED_MARKER: &str = "TAURI_HTTP_SPKI_HOST_NOT_PINNED:";

/// Native SPKI pinning transport extension.
///
/// Normal WebPKI chain and hostname validation runs before the leaf
/// certificate's SubjectPublicKeyInfo SHA-256 digest is compared with the
/// pins configured for the request host.
#[derive(Debug, Clone)]
pub struct SpkiPinning {
    pins: Arc<HashMap<String, HashSet<[u8; 32]>>>,
    require_pins_for_all_hosts: bool,
    allow_http: bool,
}

impl SpkiPinning {
    /// Creates a fail-closed pinning extension.
    ///
    /// By default every HTTPS request, including redirect targets, must have at
    /// least one configured pin. Use [`allow_unpinned_hosts`](Self::allow_unpinned_hosts)
    /// to keep normal WebPKI validation for hosts without pins.
    pub fn new() -> Self {
        Self {
            pins: Arc::new(HashMap::new()),
            require_pins_for_all_hosts: true,
            allow_http: false,
        }
    }

    /// Adds one `sha256/<base64>` SPKI pin for an exact DNS hostname.
    ///
    /// Register at least two pins per host before rotating a server key.
    pub fn pin(
        mut self,
        host: impl AsRef<str>,
        pin: impl AsRef<str>,
    ) -> Result<Self, SpkiPinError> {
        let host = normalize_host(host.as_ref())?;
        let pin = parse_pin(pin.as_ref())?;
        Arc::make_mut(&mut self.pins)
            .entry(host)
            .or_default()
            .insert(pin);
        Ok(self)
    }

    /// Allows hosts without configured pins to use normal WebPKI validation.
    pub fn allow_unpinned_hosts(mut self) -> Self {
        self.require_pins_for_all_hosts = false;
        self
    }

    /// Allows plain HTTP requests and HTTPS-to-HTTP redirects.
    ///
    /// The default is HTTPS-only so a pinned request cannot silently leave the
    /// TLS transport through a redirect.
    pub fn allow_http(mut self) -> Self {
        self.allow_http = true;
        self
    }
}

impl Default for SpkiPinning {
    fn default() -> Self {
        Self::new()
    }
}

impl<R: tauri::Runtime> HttpTransportExtension<R> for SpkiPinning {
    fn validate(&self, request: &RequestContext) -> Result<(), ExtensionError> {
        if request.danger_accept_invalid_certs() || request.danger_accept_invalid_hostnames() {
            return Err(extension_error(PinningFailure::new(
                ERROR_CODE_DANGEROUS_SETTINGS,
                "dangerous TLS settings cannot be combined with SPKI pinning",
                request.url().host_str(),
            )));
        }

        if request.url().scheme() != "https" {
            return if self.allow_http {
                Ok(())
            } else {
                Err(extension_error(PinningFailure::new(
                    ERROR_CODE_HTTP_NOT_ALLOWED,
                    "SPKI pinning is HTTPS-only unless plain HTTP is explicitly enabled",
                    request.url().host_str(),
                )))
            };
        }

        if self.require_pins_for_all_hosts {
            let host = request.url().host_str().unwrap_or_default();
            if !self.pins.contains_key(host) {
                return Err(extension_error(PinningFailure::new(
                    ERROR_CODE_HOST_NOT_PINNED,
                    "no SPKI pins are configured for the request host",
                    Some(host),
                )));
            }
        }

        Ok(())
    }

    fn configure(
        &self,
        builder: reqwest::ClientBuilder,
        _request: &RequestContext,
    ) -> Result<reqwest::ClientBuilder, ExtensionError> {
        let tls = build_tls_config(self.pins.clone(), self.require_pins_for_all_hosts).map_err(
            |message| {
                extension_error(PinningFailure::new(
                    ERROR_CODE_INITIALIZATION,
                    message,
                    None,
                ))
            },
        )?;
        Ok(builder
            .https_only(!self.allow_http)
            .use_preconfigured_tls(tls))
    }

    fn map_transport_error(
        &self,
        error: &reqwest::Error,
        _request: &RequestContext,
    ) -> Option<ExtensionError> {
        if let Some(host) = error_chain_marker_value(error, RUSTLS_PIN_MISMATCH_MARKER) {
            return Some(extension_error(PinningFailure::new(
                ERROR_CODE_PIN_MISMATCH,
                "the server certificate public key does not match a configured SPKI pin",
                Some(&host),
            )));
        }
        if let Some(host) = error_chain_marker_value(error, RUSTLS_HOST_NOT_PINNED_MARKER) {
            return Some(extension_error(PinningFailure::new(
                ERROR_CODE_HOST_NOT_PINNED,
                "no SPKI pins are configured for the request host",
                Some(&host),
            )));
        }
        None
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SpkiPinError {
    #[error("SPKI pin host must be a non-empty exact DNS hostname")]
    InvalidHost,
    #[error("SPKI pin must use the sha256/<base64> format")]
    InvalidFormat,
    #[error("SPKI pin must decode to exactly 32 bytes")]
    InvalidLength,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PinningFailure<'a> {
    code: &'static str,
    message: String,
    host: Option<&'a str>,
}

impl<'a> PinningFailure<'a> {
    fn new(code: &'static str, message: impl Into<String>, host: Option<&'a str>) -> Self {
        Self {
            code,
            message: message.into(),
            host,
        }
    }
}

fn extension_error(error: PinningFailure<'_>) -> ExtensionError {
    ExtensionError::new(error).expect("SPKI pinning errors must serialize")
}

fn normalize_host(host: &str) -> Result<String, SpkiPinError> {
    let host = host.trim().trim_end_matches('.');
    if host.is_empty() || host.contains('*') {
        return Err(SpkiPinError::InvalidHost);
    }
    match url::Host::parse(host).map_err(|_| SpkiPinError::InvalidHost)? {
        url::Host::Domain(host) if !host.is_empty() => Ok(host.to_ascii_lowercase()),
        url::Host::Ipv4(_) | url::Host::Ipv6(_) | url::Host::Domain(_) => {
            Err(SpkiPinError::InvalidHost)
        }
    }
}

fn parse_pin(pin: &str) -> Result<[u8; 32], SpkiPinError> {
    let encoded = pin
        .strip_prefix("sha256/")
        .ok_or(SpkiPinError::InvalidFormat)?;
    let decoded = STANDARD
        .decode(encoded)
        .map_err(|_| SpkiPinError::InvalidFormat)?;
    decoded.try_into().map_err(|_| SpkiPinError::InvalidLength)
}

fn error_chain_marker_value(error: &reqwest::Error, marker: &str) -> Option<String> {
    let mut source: Option<&(dyn std::error::Error + 'static)> = Some(error);
    while let Some(current) = source {
        let message = current.to_string();
        if let Some((_, value)) = message.split_once(marker) {
            return value.split_whitespace().next().map(ToOwned::to_owned);
        }
        source = current.source();
    }
    None
}

#[derive(Debug)]
struct PinnedServerCertVerifier {
    web_pki: Arc<rustls::client::WebPkiServerVerifier>,
    pins: Arc<HashMap<String, HashSet<[u8; 32]>>>,
    require_pins_for_all_hosts: bool,
}

impl ServerCertVerifier for PinnedServerCertVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp_response: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, RustlsError> {
        self.web_pki.verify_server_cert(
            end_entity,
            intermediates,
            server_name,
            ocsp_response,
            now,
        )?;

        let host = server_name
            .to_str()
            .trim_end_matches('.')
            .to_ascii_lowercase();
        let Some(pins) = self.pins.get(&host) else {
            return if self.require_pins_for_all_hosts {
                Err(RustlsError::General(format!(
                    "{RUSTLS_HOST_NOT_PINNED_MARKER}{host}"
                )))
            } else {
                Ok(ServerCertVerified::assertion())
            };
        };

        let (_, certificate) = x509_parser::parse_x509_certificate(end_entity.as_ref())
            .map_err(|_| RustlsError::General("failed to parse certificate SPKI".into()))?;
        let digest: [u8; 32] = Sha256::digest(certificate.public_key().raw).into();
        if pins.contains(&digest) {
            Ok(ServerCertVerified::assertion())
        } else {
            Err(RustlsError::General(format!(
                "{RUSTLS_PIN_MISMATCH_MARKER}{host}"
            )))
        }
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, RustlsError> {
        self.web_pki.verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, RustlsError> {
        self.web_pki.verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.web_pki.supported_verify_schemes()
    }
}

fn build_tls_config(
    pins: Arc<HashMap<String, HashSet<[u8; 32]>>>,
    require_pins_for_all_hosts: bool,
) -> Result<rustls::ClientConfig, String> {
    let roots = RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    };
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let web_pki = rustls::client::WebPkiServerVerifier::builder_with_provider(
        Arc::new(roots),
        provider.clone(),
    )
    .build()
    .map_err(|error| error.to_string())?;
    let verifier = Arc::new(PinnedServerCertVerifier {
        web_pki,
        pins,
        require_pins_for_all_hosts,
    });

    rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|error| error.to_string())
        .map(|builder| {
            builder
                .dangerous()
                .with_custom_certificate_verifier(verifier)
                .with_no_client_auth()
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rcgen::{BasicConstraints, CertificateParams, IsCa, KeyPair, KeyUsagePurpose};

    const EXAMPLE_PIN: &str = "sha256/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

    fn test_verifier(
        pins: HashMap<String, HashSet<[u8; 32]>>,
    ) -> (PinnedServerCertVerifier, CertificateDer<'static>, [u8; 32]) {
        let ca_key = KeyPair::generate().unwrap();
        let mut ca_params = CertificateParams::new(Vec::<String>::new()).unwrap();
        ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        ca_params.key_usages = vec![
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::DigitalSignature,
        ];
        let ca = ca_params.self_signed(&ca_key).unwrap();

        let leaf_key = KeyPair::generate().unwrap();
        let leaf = CertificateParams::new(vec![
            "api.example.test".into(),
            "redirect.example.test".into(),
        ])
        .unwrap()
        .signed_by(&leaf_key, &ca, &ca_key)
        .unwrap();
        let leaf = leaf.der().clone();
        let (_, parsed) = x509_parser::parse_x509_certificate(leaf.as_ref()).unwrap();
        let pin = Sha256::digest(parsed.public_key().raw).into();

        let mut roots = RootCertStore::empty();
        roots.add(ca.der().clone()).unwrap();
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let web_pki =
            rustls::client::WebPkiServerVerifier::builder_with_provider(Arc::new(roots), provider)
                .build()
                .unwrap();

        (
            PinnedServerCertVerifier {
                web_pki,
                pins: Arc::new(pins),
                require_pins_for_all_hosts: true,
            },
            leaf,
            pin,
        )
    }

    fn verify(
        verifier: &PinnedServerCertVerifier,
        leaf: &CertificateDer<'_>,
        host: &'static str,
    ) -> Result<ServerCertVerified, RustlsError> {
        verifier.verify_server_cert(
            leaf,
            &[],
            &ServerName::try_from(host).unwrap(),
            &[],
            UnixTime::now(),
        )
    }

    #[test]
    fn accepts_multiple_pins_for_a_normalized_host() {
        let extension = SpkiPinning::new()
            .pin("API.Example.COM.", EXAMPLE_PIN)
            .unwrap()
            .pin(
                "api.example.com",
                "sha256/AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE=",
            )
            .unwrap();

        assert_eq!(extension.pins["api.example.com"].len(), 2);
    }

    #[test]
    fn rejects_wildcards_ips_invalid_hosts_and_malformed_pins() {
        for host in ["*.example.com", "127.0.0.1", "not a host"] {
            assert!(matches!(
                SpkiPinning::new().pin(host, EXAMPLE_PIN),
                Err(SpkiPinError::InvalidHost)
            ));
        }
        assert!(matches!(
            SpkiPinning::new().pin("example.com", "AAAAAAAA"),
            Err(SpkiPinError::InvalidFormat)
        ));
    }

    #[test]
    fn webpki_and_hostname_validation_run_before_pin_matching() {
        let (mut verifier, leaf, leaf_pin) = test_verifier(HashMap::new());
        Arc::make_mut(&mut verifier.pins)
            .entry("api.example.test".into())
            .or_default()
            .insert(leaf_pin);

        assert!(verify(&verifier, &leaf, "api.example.test").is_ok());

        let hostname_error = verify(&verifier, &leaf, "wrong.example.test").unwrap_err();
        assert!(matches!(
            hostname_error,
            RustlsError::InvalidCertificate(rustls::CertificateError::NotValidForName)
                | RustlsError::InvalidCertificate(
                    rustls::CertificateError::NotValidForNameContext { .. }
                )
        ));
    }

    #[test]
    fn rejects_wrong_pins_and_unpinned_redirect_hosts() {
        let mut pins = HashMap::new();
        pins.insert("api.example.test".into(), HashSet::from([[0; 32]]));
        let (verifier, leaf, _) = test_verifier(pins);

        let mismatch = verify(&verifier, &leaf, "api.example.test").unwrap_err();
        assert_eq!(
            mismatch.to_string(),
            format!("unexpected error: {RUSTLS_PIN_MISMATCH_MARKER}api.example.test")
        );

        let redirect = verify(&verifier, &leaf, "redirect.example.test").unwrap_err();
        assert_eq!(
            redirect.to_string(),
            format!("unexpected error: {RUSTLS_HOST_NOT_PINNED_MARKER}redirect.example.test")
        );
    }

    #[test]
    fn builds_a_rustls_client_config() {
        let extension = SpkiPinning::new().pin("example.com", EXAMPLE_PIN).unwrap();
        let _config = build_tls_config(extension.pins, true).unwrap();
    }
}
