// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, OnceLock},
};

use crate::{ExtensionError, HttpTransportExtension, RequestContext};
use base64::{Engine, engine::general_purpose::STANDARD};
use rustls::{
    DigitallySignedStruct, Error as RustlsError, RootCertStore, SignatureScheme,
    client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    pki_types::{CertificateDer, ServerName, UnixTime},
};
use serde::Serialize;
use sha2::{Digest, Sha256};

const ERROR_CODE_PIN_MISMATCH: &str = "SPKI_PIN_MISMATCH";
const ERROR_CODE_HOST_NOT_PINNED: &str = "SPKI_HOST_NOT_PINNED";
const ERROR_CODE_HTTP_NOT_ALLOWED: &str = "SPKI_HTTP_NOT_ALLOWED";
const ERROR_CODE_DANGEROUS_SETTINGS: &str = "SPKI_DANGEROUS_SETTINGS";
const ERROR_CODE_INITIALIZATION: &str = "SPKI_INITIALIZATION_FAILED";

/// Native SPKI pinning transport extension.
///
/// Normal WebPKI chain and hostname validation runs before the leaf
/// certificate's SubjectPublicKeyInfo SHA-256 digest is compared with the
/// pins configured for the request host.
///
/// Certificate chains are validated against Mozilla's root store
/// (`webpki-roots`) unless [`root_certificates`](Self::root_certificates)
/// replaces it.
///
/// The TLS connection to an `https://` proxy, whether configured per request
/// or picked up from the environment by the `system-proxy` feature, is
/// verified with these settings too: pin the proxy host as well or use
/// [`allow_unpinned_hosts`](Self::allow_unpinned_hosts). Requests tunneled
/// through an `http://` proxy are still verified against the target's pins.
#[derive(Debug, Clone)]
pub struct SpkiPinning {
    pins: Arc<HashMap<String, HashSet<[u8; 32]>>>,
    roots: Option<Arc<RootCertStore>>,
    require_pins_for_all_hosts: bool,
    allow_http: bool,
    /// The TLS config built from the settings above on first use, shared by
    /// every request. Builder methods that change those settings reset it.
    tls: Arc<OnceLock<Result<rustls::ClientConfig, String>>>,
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
            roots: None,
            require_pins_for_all_hosts: true,
            allow_http: false,
            tls: Default::default(),
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
        self.tls = Default::default();
        Ok(self)
    }

    /// Replaces the trusted root certificates used for chain validation.
    ///
    /// Use this to pin servers whose certificates are issued by a private or
    /// self-signed CA, or to trust the operating system's roots (for example,
    /// loaded with `rustls-native-certs`). Pins are checked after the chain is
    /// validated against these roots.
    pub fn root_certificates(mut self, roots: RootCertStore) -> Self {
        self.roots = Some(Arc::new(roots));
        self.tls = Default::default();
        self
    }

    /// Allows hosts without configured pins to use normal WebPKI validation.
    pub fn allow_unpinned_hosts(mut self) -> Self {
        self.require_pins_for_all_hosts = false;
        self.tls = Default::default();
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

impl SpkiPinning {
    fn tls_config(&self) -> Result<rustls::ClientConfig, String> {
        self.tls
            .get_or_init(|| {
                build_tls_config(
                    self.pins.clone(),
                    self.roots.clone(),
                    self.require_pins_for_all_hosts,
                )
            })
            .clone()
    }

    fn validate_request(&self, request: &RequestContext) -> Result<(), ExtensionError> {
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
            let host = pin_lookup_host(request.url().host_str().unwrap_or_default());
            if !self.pins.contains_key(&host) {
                return Err(extension_error(PinningFailure::new(
                    ERROR_CODE_HOST_NOT_PINNED,
                    "no SPKI pins are configured for the request host",
                    Some(&host),
                )));
            }
        }

        Ok(())
    }
}

impl<R: tauri::Runtime> HttpTransportExtension<R> for SpkiPinning {
    fn validate(&self, request: &RequestContext) -> Result<(), ExtensionError> {
        self.validate_request(request)
    }

    fn configure(
        &self,
        builder: reqwest::ClientBuilder,
        _request: &RequestContext,
    ) -> Result<reqwest::ClientBuilder, ExtensionError> {
        let tls = self.tls_config().map_err(|message| {
            extension_error(PinningFailure::new(
                ERROR_CODE_INITIALIZATION,
                message,
                None,
            ))
        })?;
        Ok(builder
            .https_only(!self.allow_http)
            .use_preconfigured_tls(tls))
    }

    fn map_transport_error(
        &self,
        error: &reqwest::Error,
        _request: &RequestContext,
    ) -> Option<ExtensionError> {
        let rejection = pin_rejection(error)?;
        Some(extension_error(match rejection {
            PinRejection::Mismatch(host) => PinningFailure::new(
                ERROR_CODE_PIN_MISMATCH,
                "the server certificate public key does not match a configured SPKI pin",
                Some(host.as_str()),
            ),
            PinRejection::HostNotPinned(host) => PinningFailure::new(
                ERROR_CODE_HOST_NOT_PINNED,
                "no SPKI pins are configured for the request host",
                Some(host.as_str()),
            ),
        }))
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

/// The key a request or server name is looked up with: the same form
/// [`normalize_host`] stores pins under.
fn pin_lookup_host(host: &str) -> String {
    host.trim_end_matches('.').to_ascii_lowercase()
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

/// Why the pinned verifier rejected a certificate, carried through the TLS
/// stack as `rustls::Error::Other`.
#[derive(Debug, thiserror::Error)]
enum PinRejection {
    #[error("the server certificate public key does not match a configured SPKI pin for {0}")]
    Mismatch(String),
    #[error("no SPKI pins are configured for {0}")]
    HostNotPinned(String),
}

impl From<PinRejection> for RustlsError {
    fn from(rejection: PinRejection) -> Self {
        RustlsError::Other(rustls::OtherError(Arc::new(rejection)))
    }
}

/// Finds the [`PinRejection`] behind a transport error.
///
/// The TLS stack wraps the rustls error in (nested) `io::Error`s, whose
/// `source()` skips the error they wrap, so those are unwrapped with
/// `get_ref()` instead.
fn pin_rejection<'a>(error: &'a (dyn std::error::Error + 'static)) -> Option<&'a PinRejection> {
    let mut source = Some(error);
    while let Some(current) = source {
        let rejection = match current.downcast_ref::<RustlsError>() {
            Some(RustlsError::Other(other)) => other.0.downcast_ref::<PinRejection>(),
            _ => None,
        };
        if rejection.is_some() {
            return rejection;
        }
        source = match current.downcast_ref::<std::io::Error>() {
            Some(error) => error
                .get_ref()
                .map(|error| error as &(dyn std::error::Error + 'static)),
            None => current.source(),
        };
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

        let host = pin_lookup_host(&server_name.to_str());
        let Some(pins) = self.pins.get(&host) else {
            return if self.require_pins_for_all_hosts {
                Err(PinRejection::HostNotPinned(host).into())
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
            Err(PinRejection::Mismatch(host).into())
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
    roots: Option<Arc<RootCertStore>>,
    require_pins_for_all_hosts: bool,
) -> Result<rustls::ClientConfig, String> {
    let roots = roots.unwrap_or_else(|| {
        Arc::new(RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
        })
    });
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let web_pki =
        rustls::client::WebPkiServerVerifier::builder_with_provider(roots, provider.clone())
            .build()
            .map_err(|error| error.to_string())?;
    let verifier = Arc::new(PinnedServerCertVerifier {
        web_pki,
        pins,
        require_pins_for_all_hosts,
    });

    let mut config = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|error| error.to_string())?
        .dangerous()
        .with_custom_certificate_verifier(verifier)
        .with_no_client_auth();
    // reqwest uses a preconfigured TLS config as-is, so it must advertise the
    // ALPN protocols reqwest would otherwise set itself, or HTTP/2 is never negotiated.
    config.alpn_protocols = alpn_protocols();
    Ok(config)
}

fn alpn_protocols() -> Vec<Vec<u8>> {
    vec![
        #[cfg(feature = "http2")]
        b"h2".to_vec(),
        b"http/1.1".to_vec(),
    ]
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
    fn validate_matches_fully_qualified_hosts() {
        let extension = SpkiPinning::new()
            .pin("api.example.com", EXAMPLE_PIN)
            .unwrap();
        let request = |url: &str| {
            RequestContext::new(
                http::Method::GET,
                url::Url::parse(url).unwrap(),
                http::HeaderMap::new(),
                false,
                None,
                None,
                false,
                false,
                false,
            )
        };

        assert!(
            extension
                .validate_request(&request("https://api.example.com./v1"))
                .is_ok()
        );
        assert!(
            extension
                .validate_request(&request("https://API.example.com/v1"))
                .is_ok()
        );
        let error = extension
            .validate_request(&request("https://other.example.com./v1"))
            .unwrap_err();
        assert_eq!(error.value()["code"], ERROR_CODE_HOST_NOT_PINNED);
        assert_eq!(error.value()["host"], "other.example.com");
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

        // as reqwest reports it: the rustls error inside two nested io::Errors
        let wrapped = |error: RustlsError| -> Box<dyn std::error::Error + Send + Sync> {
            Box::new(std::io::Error::other(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                error,
            )))
        };

        let mismatch = wrapped(verify(&verifier, &leaf, "api.example.test").unwrap_err());
        assert!(matches!(
            pin_rejection(mismatch.as_ref()),
            Some(PinRejection::Mismatch(host)) if host == "api.example.test"
        ));

        let redirect = wrapped(verify(&verifier, &leaf, "redirect.example.test").unwrap_err());
        assert!(matches!(
            pin_rejection(redirect.as_ref()),
            Some(PinRejection::HostNotPinned(host)) if host == "redirect.example.test"
        ));

        let unrelated = wrapped(RustlsError::General("handshake failed".into()));
        assert!(pin_rejection(unrelated.as_ref()).is_none());
    }

    #[test]
    fn builds_a_rustls_client_config_with_custom_roots() {
        let ca_key = KeyPair::generate().unwrap();
        let mut ca_params = CertificateParams::new(Vec::<String>::new()).unwrap();
        ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        let ca = ca_params.self_signed(&ca_key).unwrap();
        let mut roots = RootCertStore::empty();
        roots.add(ca.der().clone()).unwrap();

        let extension = SpkiPinning::new()
            .pin("example.com", EXAMPLE_PIN)
            .unwrap()
            .root_certificates(roots);
        assert_eq!(extension.roots.as_ref().unwrap().len(), 1);
        build_tls_config(extension.pins, extension.roots, true).unwrap();
    }

    #[test]
    fn reuses_the_tls_config_until_settings_change() {
        let extension = SpkiPinning::new().pin("example.com", EXAMPLE_PIN).unwrap();
        extension.tls_config().unwrap();
        let shared = extension.clone();
        assert!(Arc::ptr_eq(&extension.tls, &shared.tls));
        assert!(shared.tls.get().is_some());

        // a clone changed afterwards gets its own config, without touching the original's
        let changed = shared
            .pin("example.org", EXAMPLE_PIN)
            .unwrap()
            .allow_unpinned_hosts();
        assert!(changed.tls.get().is_none());
        assert!(extension.tls.get().is_some());
        changed.tls_config().unwrap();
    }

    #[test]
    fn builds_a_rustls_client_config() {
        let extension = SpkiPinning::new().pin("example.com", EXAMPLE_PIN).unwrap();
        let config = build_tls_config(extension.pins, None, true).unwrap();
        assert_eq!(config.alpn_protocols.last().unwrap(), b"http/1.1");
        #[cfg(feature = "http2")]
        assert_eq!(config.alpn_protocols.first().unwrap(), b"h2");
    }
}
