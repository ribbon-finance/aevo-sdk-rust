use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Env {
    Mainnet,
    #[default]
    Testnet,
}

impl Env {
    pub fn rest_url(self) -> &'static str {
        match self {
            Self::Mainnet => "https://api.aevo.xyz",
            Self::Testnet => "https://api-testnet.aevo.xyz",
        }
    }

    pub fn ws_url(self) -> &'static str {
        match self {
            Self::Mainnet => "wss://ws.aevo.xyz",
            Self::Testnet => "wss://ws-testnet.aevo.xyz",
        }
    }

    pub fn domain_name(self) -> &'static str {
        match self {
            Self::Mainnet => "Aevo Mainnet",
            Self::Testnet => "Aevo Testnet",
        }
    }

    pub fn chain_id(self) -> u64 {
        match self {
            Self::Mainnet => 1,
            Self::Testnet => 11_155_111,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AuthMode {
    /// Send `AEVO-KEY`, `AEVO-TIMESTAMP`, and `AEVO-SIGNATURE`.
    #[default]
    Hmac,
    /// Send `AEVO-KEY` and `AEVO-SECRET` directly.
    SecretHeader,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[REDACTED]")
    }
}
