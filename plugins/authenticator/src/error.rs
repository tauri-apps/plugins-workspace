use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Base64Decode(#[from] base64::DecodeError),
    #[error(transparent)]
    JSON(#[from] serde_json::Error),
    #[error(transparent)]
    U2F(#[from] crate::u2f_crate::u2ferror::U2fError),
    #[error(transparent)]
    Auth(#[from] authenticator::errors::AuthenticatorError),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
