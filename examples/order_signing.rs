// Quick Start 3: Order Signing (EIP-712)
use grvt_rust_sdk::signer::{
    decode_private_key, default_expiration_ns, parse_instrument_hash, random_nonce, scale_price, scale_size, sign_order, SignOrderLeg,
    SignOrderParams,
};
use grvt_rust_sdk::types::TimeInForce;

#[tokio::main]
async fn main() -> Result<(), grvt_rust_sdk::GrvtError> {
    // Load instrument metadata (e.g. from instrument_full API)
    let instrument_hash = "0x030501"; // from API
    let base_decimals = 9u32;

    let asset_id = parse_instrument_hash(instrument_hash)?;
    let contract_size = scale_size(0.1, base_decimals);
    let limit_price = scale_price(50000.0);

    let params = SignOrderParams {
        sub_account_id: 12345,
        is_market: false,
        time_in_force: TimeInForce::GoodTillTime,
        post_only: false,
        reduce_only: false,
        legs: vec![SignOrderLeg {
            asset_id,
            contract_size,
            limit_price,
            is_buying_contract: true,
        }],
        nonce: random_nonce(),
        expiration_ns: default_expiration_ns(),
        chain_id: 326,
    };

    let private_key_hex = std::env::var("GRVT_PRIVATE_KEY").map_err(|e| grvt_rust_sdk::GrvtError::Config(e.to_string()))?;
    let private_key_bytes = decode_private_key(&private_key_hex)?;
    let signed = sign_order(&params, &private_key_bytes)?;

    println!("Signed order - use signed.signature in CreateOrderRequest");
    println!("Signature: {:?}", signed.signature);

    Ok(())
}
