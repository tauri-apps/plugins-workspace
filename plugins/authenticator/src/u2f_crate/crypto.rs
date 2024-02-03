// Copyright 2021 Flavio Oliveira
// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Cryptographic operation wrapper for Webauthn. This module exists to
//! allow ease of auditing, safe operation wrappers for the webauthn library,
//! and cryptographic provider abstraction. This module currently uses OpenSSL
//! as the cryptographic primitive provider.

// Source can be found here: https://github.com/Firstyear/webauthn-rs/blob/master/src/crypto.rs

#![allow(non_camel_case_types)]

use openssl::{bn, ec, hash, nid, sign, x509};
use std::convert::TryFrom;

// use super::constants::*;
use crate::u2f_crate::u2ferror::U2fError;
use openssl::pkey::Public;

// use super::proto::*;

// Why OpenSSL over another rust crate?
// - Well, the openssl crate allows us to reconstruct a public key from the
//   x/y group coords, where most others want a pkcs formatted structure. As
//   a result, it's easiest to use openssl as it gives us exactly what we need
//   for these operations, and despite it's many challenges as a library, it
//   has resources and investment into it's maintenance, so we can a least
//   assert a higher level of confidence in it that <backyard crypto here>.

// Object({Integer(-3): Bytes([48, 185, 178, 204, 113, 186, 105, 138, 190, 33, 160, 46, 131, 253, 100, 177, 91, 243, 126, 128, 245, 119, 209, 59, 186, 41, 215, 196, 24, 222, 46, 102]), Integer(-2): Bytes([158, 212, 171, 234, 165, 197, 86, 55, 141, 122, 253, 6, 92, 242, 242, 114, 158, 221, 238, 163, 127, 214, 120, 157, 145, 226, 232, 250, 144, 150, 218, 138]), Integer(-1): U64(1), Integer(1): U64(2), Integer(3): I64(-7)})
//

/// An X509PublicKey. This is what is otherwise known as a public certificate
/// which comprises a public key and other signed metadata related to the issuer
/// of the key.
pub struct X509PublicKey {
    pubk: x509::X509,
}

impl std::fmt::Debug for X509PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "X509PublicKey")
    }
}

impl TryFrom<&[u8]> for X509PublicKey {
    type Error = U2fError;

    // Must be DER bytes. If you have PEM, base64decode first!
    fn try_from(d: &[u8]) -> Result<Self, Self::Error> {
        let pubk = x509::X509::from_der(d)?;
        Ok(X509PublicKey { pubk })
    }
}

impl X509PublicKey {
    pub(crate) fn common_name(&self) -> Option<String> {
        let cert = &self.pubk;

        let subject = cert.subject_name();
        let common = subject
            .entries_by_nid(openssl::nid::Nid::COMMONNAME)
            .next()
            .map(|b| b.data().as_slice());

        if let Some(common) = common {
            std::str::from_utf8(common).ok().map(|s| s.to_string())
        } else {
            None
        }
    }

    pub(crate) fn is_secp256r1(&self) -> Result<bool, U2fError> {
        // Can we get the public key?
        let pk = self.pubk.public_key()?;

        let ec_key = pk.ec_key()?;

        ec_key.check_key()?;

        let ec_grpref = ec_key.group();

        let ec_curve = ec_grpref.curve_name().ok_or(U2fError::OpenSSLNoCurveName)?;

        Ok(ec_curve == nid::Nid::X9_62_PRIME256V1)
    }

    pub(crate) fn verify_signature(
        &self,
        signature: &[u8],
        verification_data: &[u8],
    ) -> Result<bool, U2fError> {
        let pkey = self.pubk.public_key()?;

        // TODO: Should this determine the hash type from the x509 cert? Or other?
        let mut verifier = sign::Verifier::new(hash::MessageDigest::sha256(), &pkey)?;
        verifier.update(verification_data)?;
        Ok(verifier.verify(signature)?)
    }
}

pub struct NISTP256Key {
    /// The key's public X coordinate.
    pub x: [u8; 32],
    /// The key's public Y coordinate.
    pub y: [u8; 32],
}

impl NISTP256Key {
    pub fn from_bytes(public_key_bytes: &[u8]) -> Result<Self, U2fError> {
        if public_key_bytes.len() != 65 {
            return Err(U2fError::InvalidPublicKey);
        }

        if public_key_bytes[0] != 0x04 {
            return Err(U2fError::InvalidPublicKey);
        }

        let mut x: [u8; 32] = Default::default();
        x.copy_from_slice(&public_key_bytes[1..=32]);

        let mut y: [u8; 32] = Default::default();
        y.copy_from_slice(&public_key_bytes[33..=64]);

        Ok(NISTP256Key { x, y })
    }

    fn get_key(&self) -> Result<ec::EcKey<Public>, U2fError> {
        let ec_group = ec::EcGroup::from_curve_name(openssl::nid::Nid::X9_62_PRIME256V1)?;

        let xbn = bn::BigNum::from_slice(&self.x)?;
        let ybn = bn::BigNum::from_slice(&self.y)?;

        let ec_key = openssl::ec::EcKey::from_public_key_affine_coordinates(&ec_group, &xbn, &ybn)?;

        // Validate the key is sound. IIRC this actually checks the values
        // are correctly on the curve as specified
        ec_key.check_key()?;

        Ok(ec_key)
    }

    pub fn verify_signature(
        &self,
        signature: &[u8],
        verification_data: &[u8],
    ) -> Result<bool, U2fError> {
        let pkey = self.get_key()?;

        let signature = openssl::ecdsa::EcdsaSig::from_der(signature)?;
        let hash = openssl::sha::sha256(verification_data);

        Ok(signature.verify(hash.as_ref(), &pkey)?)
    }
}
