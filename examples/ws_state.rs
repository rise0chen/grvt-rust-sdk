// Quick Start 6: WebSocket – State (authenticated)
use grvt_rust_sdk::{ws, GrvtClient, GrvtConfig};

#[tokio::main]
async fn main() -> Result<(), grvt_rust_sdk::GrvtError> {
    dotenvy::dotenv().ok();
    let config = GrvtConfig::from_env()?;
    let client = GrvtClient::from_config(&config).await?;

    let selectors = vec![format!("{}-BTC_USDT_Perp", config.sub_account_id)];
    let mut rx = ws::subscribe_state(&config.environment, &client.get_session_cookie().await, &client.account_id, selectors, 64).await?;

    while let Some(event) = rx.recv().await {
        println!("State event: {:?}", event);
    }
    Ok(())
}
