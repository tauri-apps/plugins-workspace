use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    GlobalHotkey(String),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

impl From<global_hotkey::Error> for Error {
    fn from(value: global_hotkey::Error) -> Self {
        Self::GlobalHotkey(value.to_string())
    }
}
