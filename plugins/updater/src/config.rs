use serde::{Deserialize, Deserializer};
use url::Url;

/// Updater configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub endpoints: Vec<UpdaterEndpoint>,
    /// Additional arguments given to the NSIS or WiX installer.
    #[serde(default, alias = "installer-args")]
    pub installer_args: Vec<String>,
}

/// A URL to an updater server.
///
/// The URL must use the `https` scheme on production.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UpdaterEndpoint(pub Url);

impl std::fmt::Display for UpdaterEndpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'de> Deserialize<'de> for UpdaterEndpoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let url = Url::deserialize(deserializer)?;
        #[cfg(all(not(debug_assertions), not(feature = "schema")))]
        {
            if url.scheme() != "https" {
                return Err(serde::de::Error::custom(
                    "The configured updater endpoint must use the `https` protocol.",
                ));
            }
        }
        Ok(Self(url))
    }
}
