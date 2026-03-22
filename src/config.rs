use crate::error::GrvtError;
use std::env;

/// GRVT environment variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Prod,
    Testnet,
    Staging,
    Dev,
}

impl Environment {
    pub fn from_str_value(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "prod" | "production" | "mainnet" => Some(Self::Prod),
            "testnet" | "tn" => Some(Self::Testnet),
            "staging" | "stg" => Some(Self::Staging),
            "dev" | "development" => Some(Self::Dev),
            _ => None,
        }
    }

    pub fn full_base(&self) -> &'static str {
        match self {
            Self::Prod => "https://trades.grvt.io",
            Self::Testnet => "https://trades.testnet.grvt.io",
            Self::Staging => "https://trades.staging.gravitymarkets.io",
            Self::Dev => "https://trades.dev.gravitymarkets.io",
        }
    }

    pub fn lite_base(&self) -> &'static str {
        match self {
            Self::Prod => "https://trades.grvt.io",
            Self::Testnet => "https://trades.testnet.grvt.io",
            Self::Staging => "https://trades.staging.gravitymarkets.io",
            Self::Dev => "https://trades.dev.gravitymarkets.io",
        }
    }

    pub fn auth_base(&self) -> &'static str {
        match self {
            Self::Prod => "https://edge.grvt.io",
            Self::Testnet => "https://edge.testnet.grvt.io",
            Self::Staging => "https://edge.staging.gravitymarkets.io",
            Self::Dev => "https://edge.dev.gravitymarkets.io",
        }
    }

    pub fn market_data_base(&self) -> &'static str {
        match self {
            Self::Prod => "https://market-data.grvt.io",
            Self::Testnet => "https://market-data.testnet.grvt.io",
            Self::Staging => "https://market-data.staging.gravitymarkets.io",
            Self::Dev => "https://market-data.dev.gravitymarkets.io",
        }
    }

    pub fn full_ws(&self) -> &'static str {
        match self {
            Self::Prod => "wss://trades.grvt.io/ws/full",
            Self::Testnet => "wss://trades.testnet.grvt.io/ws/full",
            Self::Staging => "wss://trades.staging.gravitymarkets.io/ws/full",
            Self::Dev => "wss://trades.dev.gravitymarkets.io/ws/full",
        }
    }

    pub fn lite_ws(&self) -> &'static str {
        match self {
            Self::Prod => "wss://trades.grvt.io/ws/lite",
            Self::Testnet => "wss://trades.testnet.grvt.io/ws/lite",
            Self::Staging => "wss://trades.staging.gravitymarkets.io/ws/lite",
            Self::Dev => "wss://trades.dev.gravitymarkets.io/ws/lite",
        }
    }

    pub fn market_data_ws(&self) -> &'static str {
        match self {
            Self::Prod => "wss://market-data.grvt.io/ws/full",
            Self::Testnet => "wss://market-data.testnet.grvt.io/ws/full",
            Self::Staging => "wss://market-data.staging.gravitymarkets.io/ws/full",
            Self::Dev => "wss://market-data.dev.gravitymarkets.io/ws/full",
        }
    }

    pub fn default_chain_id(&self) -> u64 {
        match self {
            Self::Prod => 325,
            Self::Testnet => 326,
            Self::Staging => 326,
            Self::Dev => 327,
        }
    }
}

/// SDK configuration. Use [`GrvtConfig::from_env`] for environment-variable based
/// setup, or [`GrvtConfigBuilder`] for programmatic configuration.
#[derive(Debug, Clone)]
pub struct GrvtConfig {
    pub environment: Environment,
    pub api_key: String,
    pub sub_account_id: String,
    pub private_key_hex: Option<String>,
    pub chain_id: Option<u64>,
}

impl GrvtConfig {
    pub fn builder() -> GrvtConfigBuilder {
        GrvtConfigBuilder::default()
    }

    /// Resolve all values from environment variables (compatible with grvt-pysdk env layout).
    ///
    /// Required env vars:
    ///   - `GRVT_ENV` (defaults to "testnet")
    ///   - `GRVT_API_KEY_<ENV>` or `GRVT_API_KEY` (environment-specific API key)
    ///   - `GRVT_TRADING_ACCOUNT_ID` or `GRVT_SUB_ACCOUNT_ID`
    ///
    /// Optional env vars:
    ///   - `GRVT_PRIVATE_KEY` / `GRVT_API_SECRET_TESTNET` (for signing)
    ///   - `GRVT_CHAIN_ID`
    pub fn from_env() -> crate::error::Result<Self> {
        let environment = resolve_environment();
        let api_key = resolve_api_key(&environment)?;
        let sub_account_id = resolve_sub_account_id()?;
        let private_key_hex = env::var("GRVT_PRIVATE_KEY")
            .or_else(|_| env::var("GRVT_API_SECRET_TESTNET"))
            .ok();
        let chain_id = env::var("GRVT_CHAIN_ID")
            .ok()
            .and_then(|v| v.parse().ok());

        Ok(Self {
            environment,
            api_key,
            sub_account_id,
            private_key_hex,
            chain_id,
        })
    }

    pub fn effective_chain_id(&self) -> u64 {
        self.chain_id.unwrap_or_else(|| self.environment.default_chain_id())
    }
}

#[derive(Debug, Default)]
pub struct GrvtConfigBuilder {
    environment: Option<Environment>,
    api_key: Option<String>,
    sub_account_id: Option<String>,
    private_key_hex: Option<String>,
    chain_id: Option<u64>,
}

impl GrvtConfigBuilder {
    pub fn environment(mut self, env: Environment) -> Self {
        self.environment = Some(env);
        self
    }

    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn sub_account_id(mut self, id: impl Into<String>) -> Self {
        self.sub_account_id = Some(id.into());
        self
    }

    pub fn private_key_hex(mut self, key: impl Into<String>) -> Self {
        self.private_key_hex = Some(key.into());
        self
    }

    pub fn chain_id(mut self, id: u64) -> Self {
        self.chain_id = Some(id);
        self
    }

    pub fn build(self) -> crate::error::Result<GrvtConfig> {
        let environment = self.environment.ok_or_else(|| {
            GrvtError::Config("environment is required".into())
        })?;
        let api_key = self.api_key.ok_or_else(|| {
            GrvtError::Config("api_key is required".into())
        })?;
        let sub_account_id = self.sub_account_id.ok_or_else(|| {
            GrvtError::Config("sub_account_id is required".into())
        })?;

        Ok(GrvtConfig {
            environment,
            api_key,
            sub_account_id,
            private_key_hex: self.private_key_hex,
            chain_id: self.chain_id,
        })
    }
}

fn resolve_environment() -> Environment {
    let raw = env::var("GRVT_ENV").unwrap_or_else(|_| "testnet".into());
    Environment::from_str_value(&raw).unwrap_or(Environment::Testnet)
}

fn resolve_api_key(environment: &Environment) -> crate::error::Result<String> {
    let (primary, fallback) = match environment {
        Environment::Prod => ("GRVT_API_KEY_PROD", Some("GRVT_API_KEY_MAINNET")),
        Environment::Testnet => ("GRVT_API_KEY_TESTNET", None),
        Environment::Staging => ("GRVT_API_KEY_STAGING", None),
        Environment::Dev => ("GRVT_API_KEY_DEV", None),
    };

    if let Ok(v) = env::var(primary) {
        if !v.is_empty() {
            return Ok(v);
        }
    }
    if let Some(fb) = fallback {
        if let Ok(v) = env::var(fb) {
            if !v.is_empty() {
                return Ok(v);
            }
        }
    }
    // Generic fallback
    if let Ok(v) = env::var("GRVT_API_KEY") {
        if !v.is_empty() {
            return Ok(v);
        }
    }
    Err(GrvtError::Config(format!(
        "environment variable {primary} is not set (GRVT_ENV={environment:?})"
    )))
}

fn resolve_sub_account_id() -> crate::error::Result<String> {
    for var in ["GRVT_TRADING_ACCOUNT_ID", "GRVT_SUB_ACCOUNT_ID"] {
        if let Ok(v) = env::var(var) {
            if !v.is_empty() {
                return Ok(v);
            }
        }
    }
    Err(GrvtError::Config(
        "GRVT_TRADING_ACCOUNT_ID or GRVT_SUB_ACCOUNT_ID must be set".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_from_str() {
        assert_eq!(Environment::from_str_value("prod"), Some(Environment::Prod));
        assert_eq!(Environment::from_str_value("mainnet"), Some(Environment::Prod));
        assert_eq!(Environment::from_str_value("testnet"), Some(Environment::Testnet));
        assert_eq!(Environment::from_str_value("TN"), Some(Environment::Testnet));
        assert_eq!(Environment::from_str_value("staging"), Some(Environment::Staging));
        assert_eq!(Environment::from_str_value("dev"), Some(Environment::Dev));
        assert_eq!(Environment::from_str_value("unknown"), None);
    }

    #[test]
    fn test_builder_missing_fields() {
        let err = GrvtConfigBuilder::default().build();
        assert!(err.is_err());
    }

    #[test]
    fn test_builder_ok() {
        let cfg = GrvtConfig::builder()
            .environment(Environment::Testnet)
            .api_key("key123")
            .sub_account_id("42")
            .build()
            .unwrap();
        assert_eq!(cfg.environment, Environment::Testnet);
        assert_eq!(cfg.api_key, "key123");
        assert_eq!(cfg.sub_account_id, "42");
        assert_eq!(cfg.effective_chain_id(), 326);
    }

    #[test]
    fn test_endpoints() {
        let env = Environment::Testnet;
        assert!(env.full_base().contains("testnet"));
        assert!(env.auth_base().contains("testnet"));
        assert!(env.market_data_base().contains("testnet"));
        assert!(env.full_ws().starts_with("wss://"));
        assert!(env.market_data_ws().starts_with("wss://"));
    }
}
