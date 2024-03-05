// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::Error;
use crate::Runtime;
use crate::Store;
use std::fs::create_dir_all;
use std::fs::read;
use std::fs::File;
use std::io::Write;
use tauri::Manager;

#[cfg(desktop)]
impl<R: Runtime> Store<R> {
    pub fn save(&self) -> Result<(), Error> {
        let app_dir = self
            .app
            .path()
            .app_data_dir()
            .expect("failed to resolve app dir");
        let store_path = app_dir.join(&self.path);

        create_dir_all(store_path.parent().expect("invalid store path"))?;

        let bytes = (self.serialize)(&self.cache).map_err(Error::Serialize)?;
        let mut f = File::create(&store_path)?;
        f.write_all(&bytes)?;

        Ok(())
    }

    /// Update the store from the on-disk state
    pub fn load(&mut self) -> Result<(), Error> {
        let app_dir = self
            .app
            .path()
            .app_data_dir()
            .expect("failed to resolve app dir");
        let store_path = app_dir.join(&self.path);

        let bytes = read(store_path)?;

        self.cache
            .extend((self.deserialize)(&bytes).map_err(Error::Deserialize)?);

        Ok(())
    }
}
