// Quick Start 1: REST Client (from environment)
use grvt_rust_sdk::{GrvtClient, GrvtConfig};

#[tokio::main]
async fn main() -> Result<(), grvt_rust_sdk::GrvtError> {
    dotenvy::dotenv().ok();
    let config = GrvtConfig::from_env()?;
    let client = GrvtClient::from_config(&config).await?;

    let positions = client
        .positions_full(&grvt_rust_sdk::types::SubAccountRequest {
            sub_account_id: config.sub_account_id.clone(),
        })
        .await?;

    println!("Positions: {:?}", positions.result);
    Ok(())
}
