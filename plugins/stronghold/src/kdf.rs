use argon2::Argon2;
use rand_chacha::ChaCha20Rng;
use rand_core::{RngCore, SeedableRng};
use std::path::Path;

/// NOTE: Hash supplied to Stronghold must be 32bits long.
/// This is a current limitation of Stronghold.
const HASH_LENGTH: usize = 32;

pub struct KeyDerivation {}

impl KeyDerivation {
    /// Will create a key from [`password`] and a generated salt.
    /// Salt will be generated to file [`salt_path`] or taken from it
    /// if file already exists
    pub fn argon2(password: &str, salt_path: &Path) -> Vec<u8> {
        let mut salt = [0u8; HASH_LENGTH];
        create_or_get_salt(&mut salt, salt_path);

        let mut encoded = [0u8; HASH_LENGTH];
        Argon2::default()
            .hash_password_into(password.as_bytes(), &salt, &mut encoded)
            .expect("Failed to generate hash for password");
        encoded.to_vec()
    }
}

fn create_or_get_salt(salt: &mut [u8], salt_path: &Path) {
    if salt_path.is_file() {
        // Get existing salt
        let tmp = std::fs::read(salt_path).unwrap();
        salt.clone_from_slice(&tmp);
    } else {
        // Generate new salt
        let mut gen = ChaCha20Rng::from_entropy();
        gen.fill_bytes(salt);
        std::fs::write(salt_path, salt).expect("Failed to write salt for Stronghold")
    }
}
