// Quick Start 2: REST Client (programmatic config)
use grvt_rust_sdk::{Environment, GrvtClient, GrvtConfig};

#[tokio::main]
async fn main() -> Result<(), grvt_rust_sdk::GrvtError> {
    let config = GrvtConfig::builder()
        .environment(Environment::Testnet)
        .api_key("your-api-key")
        .sub_account_id("12345")
        .build()?;

    let _client = GrvtClient::from_config(&config).await?;
    // ... use _client
    Ok(())
}
