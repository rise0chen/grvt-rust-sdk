// Quick Start 1: REST Client (from environment)
use grvt_rust_sdk::{GrvtClient, GrvtConfig};

#[tokio::main]
async fn main() -> Result<(), grvt_rust_sdk::GrvtError> {
    dotenvy::dotenv().ok();
    let config = GrvtConfig::from_env()?;
    let client = GrvtClient::from_config(&config).await?;

    let account = client
        .account_summary_full(&grvt_rust_sdk::types::AccountSummaryRequest {
            sub_account_id: config.sub_account_id.clone(),
        })
        .await?;
    println!("Account: {:?}", account.result);

    let positions = client
        .positions_full(&grvt_rust_sdk::types::PosotionsRequest {
            sub_account_id: config.sub_account_id.clone(),
            base: Some(vec!["BTC".into()]),
            quote: None,
        })
        .await?;
    println!("Positions: {:?}", positions.result);

    let instrument = client
        .instrument_full(&grvt_rust_sdk::types::InstrumentRequest {
            instrument: "BTC_USDT_Perp".into(),
        })
        .await?;
    println!("instrument: {:?}", instrument.result);

    let ticker = client
        .ticker_full(&grvt_rust_sdk::types::InstrumentRequest {
            instrument: "BTC_USDT_Perp".into(),
        })
        .await?;
    println!("ticker: {:?}", ticker.result);

    let book = client
        .book_full(&grvt_rust_sdk::types::BookRequest {
            instrument: "BTC_USDT_Perp".into(),
            depth: 10,
        })
        .await?;
    println!("book: {:?}", book.result);

    let funding = client
        .funding_full(&grvt_rust_sdk::types::FundingRequest {
            instrument: "BTC_USDT_Perp".into(),
            start_time: None,
            end_time: None,
            limit: Some(3),
        })
        .await?;
    println!("funding: {:?}", funding.result);

    Ok(())
}
