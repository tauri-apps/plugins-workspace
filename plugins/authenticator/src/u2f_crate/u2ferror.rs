// Copyright 2021 Flavio Oliveira
// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use thiserror::Error;

#[derive(Debug, Error)]
pub enum U2fError {
    #[error("ASM1 Decoder error")]
    Asm1DecoderError,
    #[error("Not able to verify signature")]
    BadSignature,
    #[error("Not able to generate random bytes")]
    RandomSecureBytesError,
    #[error("Invalid Reserved Byte")]
    InvalidReservedByte,
    #[error("Challenge Expired")]
    ChallengeExpired,
    #[error("Wrong Key Handler")]
    WrongKeyHandler,
    #[error("Invalid Client Data")]
    InvalidClientData,
    #[error("Invalid Signature Data")]
    InvalidSignatureData,
    #[error("Invalid User Presence Byte")]
    InvalidUserPresenceByte,
    #[error("Failed to parse certificate")]
    BadCertificate,
    #[error("Not Trusted Anchor")]
    NotTrustedAnchor,
    #[error("Counter too low")]
    CounterTooLow,
    #[error("Invalid public key")]
    OpenSSLNoCurveName,
    #[error("OpenSSL no curve name")]
    InvalidPublicKey,
    #[error(transparent)]
    OpenSSLError(#[from] openssl::error::ErrorStack),
}
