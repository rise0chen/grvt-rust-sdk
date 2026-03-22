// Quick Start 4: Create and Cancel Order (full flow)
use grvt_rust_sdk::signer::*;
use grvt_rust_sdk::types::*;
use grvt_rust_sdk::{GrvtClient, GrvtConfig};

#[tokio::main]
async fn main() -> Result<(), grvt_rust_sdk::GrvtError> {
    dotenvy::dotenv().ok();
    let config = GrvtConfig::from_env()?;
    let client = GrvtClient::from_config(&config).await?;
    let instrument = "BTC_USDT_Perp".to_string();
    let size = 0.01_f64;
    let limit_price = 50_000.0_f64;
    let is_buy = true;

    // 1. Fetch instrument metadata
    let inst_resp = client
        .instrument_full(&InstrumentRequest {
            instrument: instrument.clone(),
        })
        .await?;
    let info = inst_resp
        .result
        .ok_or(grvt_rust_sdk::GrvtError::Config(
            "instrument not found".into(),
        ))?;
    let instrument_hash = info
        .instrument_hash
        .as_deref()
        .unwrap_or("0x030501");
    let _base_decimals = info.base_decimals.unwrap_or(9);

    // 2. Sign the order
    let asset_id = parse_instrument_hash(instrument_hash)?;
    // GRVT signature uses 1e9 size scaling for BTC/ETH style contracts.
    let signed_contract_size = scale_size(size, 9);
    let signed_limit_price = scale_price(limit_price);
    let params = SignOrderParams {
        sub_account_id: config.sub_account_id.parse().unwrap_or(0),
        is_market: false,
        time_in_force: TimeInForce::GoodTillTime,
        post_only: false,
        reduce_only: false,
        legs: vec![SignOrderLeg {
            asset_id,
            contract_size: signed_contract_size,
            limit_price: signed_limit_price,
            is_buying_contract: is_buy,
        }],
        nonce: random_nonce(),
        expiration_ns: default_expiration_ns(),
        chain_id: config.effective_chain_id(),
    };
    let priv_key = decode_private_key(
        &config
            .private_key_hex
            .as_ref()
            .ok_or(grvt_rust_sdk::GrvtError::Config(
                "GRVT_PRIVATE_KEY required for signing".into(),
            ))?,
    )?;
    let signed = sign_order(&params, &priv_key)?;

    let client_order_id = chrono::Utc::now()
        .timestamp_nanos_opt()
        .unwrap_or(0)
        .to_string();

    // 3. Build and send create request
    let req = CreateOrderRequest {
        order: OrderPayload {
            sub_account_id: config.sub_account_id.clone(),
            is_market: false,
            time_in_force: TimeInForce::GoodTillTime.as_api_str().to_string(),
            post_only: false,
            reduce_only: false,
            legs: vec![grvt_rust_sdk::types::OrderLeg {
                instrument,
                size: size.to_string(),
                // full API expects human-readable decimal price string
                limit_price: Some(limit_price.to_string()),
                is_buying_asset: is_buy,
            }],
            signature: signed.signature,
            metadata: Some(OrderMetadata {
                client_order_id: Some(client_order_id.clone()),
                create_time: None,
                trigger: None,
                broker: None,
                is_position_transfer: None,
                allow_crossing: None,
            }),
            builder: None,
            builder_fee: None,
        },
    };

    let _create_resp = client.create_order_full(&req).await?;
    println!("Created order with client_order_id: {client_order_id}");

    // 4. Cancel the same order by client_order_id (order_id can be unavailable)
    let cancel_req = CancelOrderRequest {
        sub_account_id: config.sub_account_id.clone(),
        order_id: None,
        client_order_id: Some(client_order_id.clone()),
    };
    let cancel_resp = client.cancel_order_full(&cancel_req).await?;
    println!(
        "Cancel result: code={:?}, msg={:?}, client_order_id={client_order_id}",
        cancel_resp.code, cancel_resp.msg
    );
    Ok(())
}
