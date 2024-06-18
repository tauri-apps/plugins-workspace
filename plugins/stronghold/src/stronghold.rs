// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{convert::TryFrom, ops::Deref, path::Path};

use iota_stronghold::{KeyProvider, SnapshotPath};
use serde::{Serialize, Serializer};
use zeroize::Zeroizing;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("stronghold not initialized")]
    StrongholdNotInitialized,
    #[error(transparent)]
    Stronghold(#[from] iota_stronghold::ClientError),
    #[error(transparent)]
    Memory(#[from] iota_stronghold::MemoryError),
    #[error(transparent)]
    Procedure(#[from] iota_stronghold::procedures::ProcedureError),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

pub struct Stronghold {
    inner: iota_stronghold::Stronghold,
    path: SnapshotPath,
    keyprovider: KeyProvider,
}

impl Stronghold {
    pub fn new<P: AsRef<Path>>(path: P, password: Vec<u8>) -> Result<Self> {
        let path = SnapshotPath::from_path(path);
        let stronghold = iota_stronghold::Stronghold::default();
        let keyprovider = KeyProvider::try_from(Zeroizing::new(password))?;
        if path.exists() {
            stronghold.load_snapshot(&keyprovider, &path)?;
        }
        Ok(Self {
            inner: stronghold,
            path,
            keyprovider,
        })
    }

    pub fn save(&self) -> Result<()> {
        self.inner
            .commit_with_keyprovider(&self.path, &self.keyprovider)?;
        Ok(())
    }

    pub fn inner(&self) -> &iota_stronghold::Stronghold {
        &self.inner
    }
}

impl Deref for Stronghold {
    type Target = iota_stronghold::Stronghold;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
