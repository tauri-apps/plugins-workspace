// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Key derivation used to turn the user password into the key that encrypts a snapshot.
//!
//! Only available when the **kdf** Cargo feature is enabled, which is the case by default.

use rand_chacha::ChaCha20Rng;
use rand_core::{RngCore, SeedableRng};
use std::path::Path;

/// NOTE: Hash supplied to Stronghold must be 32bits long.
/// This is a current limitation of Stronghold.
const HASH_LENGTH: usize = 32;

/// Password hashing functions that can be used as the key derivation function of
/// [`Builder::new`](crate::Builder::new).
pub struct KeyDerivation {}

impl KeyDerivation {
    /// Hashes `password` with Argon2 using the salt stored in `salt_path`, returning the
    /// 32 bytes key used to encrypt a snapshot.
    ///
    /// The salt is read from `salt_path` when that file already exists, otherwise a new
    /// random salt is generated and written to it.
    ///
    /// # Panics
    ///
    /// Panics when the salt file cannot be read or written, when its contents are not
    /// 32 bytes long, or when hashing the password fails.
    pub fn argon2(password: &str, salt_path: &Path) -> Vec<u8> {
        let mut salt = [0u8; HASH_LENGTH];
        create_or_get_salt(&mut salt, salt_path);

        argon2::hash_raw(password.as_bytes(), &salt, &Default::default())
            .expect("Failed to generate hash for password")
    }
}

fn create_or_get_salt(salt: &mut [u8], salt_path: &Path) {
    if salt_path.is_file() {
        // Get existing salt
        let tmp = std::fs::read(salt_path).unwrap();
        salt.clone_from_slice(&tmp);
    } else {
        // Generate new salt
        let mut gen = ChaCha20Rng::from_os_rng();
        gen.fill_bytes(salt);
        std::fs::write(salt_path, salt).expect("Failed to write salt for Stronghold")
    }
}
