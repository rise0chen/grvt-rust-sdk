// Quick Start 5: WebSocket – Market Data
use grvt_rust_sdk::types::MarketDataEvent;
use grvt_rust_sdk::{ws, Environment};

#[tokio::main]
async fn main() -> Result<(), grvt_rust_sdk::GrvtError> {
    let env = Environment::Testnet;
    let mut rx = ws::subscribe_market_data(&env, "BTC_USDT_Perp", 64).await?;

    while let Some(event) = rx.recv().await {
        match event {
            MarketDataEvent::BookDelta { bids, asks } => {
                println!("Book update: {} bids, {} asks", bids.len(), asks.len());
            }
            MarketDataEvent::Trade {
                price,
                size,
                event_time,
                ..
            } => {
                println!("Trade: price={} size={} at {}", price, size, event_time);
            }
        }
    }
    Ok(())
}
