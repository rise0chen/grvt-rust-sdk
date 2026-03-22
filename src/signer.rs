use alloy_primitives::{keccak256, B256, U256};
use alloy_sol_types::{sol, Eip712Domain, SolStruct};
use secp256k1::{ecdsa::RecoverableSignature, Message, Secp256k1, SecretKey};

use crate::error::{GrvtError, Result};
use crate::types::{OrderSignature, TimeInForce};

const PRICE_SCALE: f64 = 1_000_000_000.0;

sol! {
    struct OrderLeg {
        uint256 assetID;
        uint64 contractSize;
        uint64 limitPrice;
        bool isBuyingContract;
    }

    struct Order {
        uint64 subAccountID;
        bool isMarket;
        uint8 timeInForce;
        bool postOnly;
        bool reduceOnly;
        OrderLeg[] legs;
        uint32 nonce;
        int64 expiration;
    }
}

/// Input parameters for EIP-712 order signing.
#[derive(Debug, Clone)]
pub struct SignOrderParams {
    pub sub_account_id: u64,
    pub is_market: bool,
    pub time_in_force: TimeInForce,
    pub post_only: bool,
    pub reduce_only: bool,
    pub legs: Vec<SignOrderLeg>,
    pub nonce: u32,
    /// Expiration as nanosecond unix timestamp.
    pub expiration_ns: i64,
    /// EIP-712 domain chain ID.
    pub chain_id: u64,
}

#[derive(Debug, Clone)]
pub struct SignOrderLeg {
    /// Instrument asset ID (from instrument_hash).
    pub asset_id: U256,
    /// Contract size in base-decimals integer units.
    pub contract_size: u64,
    /// Limit price in 1e9 scaled integer units.
    pub limit_price: u64,
    pub is_buying_contract: bool,
}

/// Signed result. Compatible with [`OrderSignature`] but also exposes raw digest.
#[derive(Debug, Clone)]
pub struct SignedOrder {
    pub signature: OrderSignature,
    pub signer_address: String,
    pub digest: B256,
}

/// Derive the Ethereum address from a secp256k1 secret key.
pub fn address_from_private_key(private_key_bytes: &[u8]) -> Result<String> {
    let secp = Secp256k1::new();
    let sk = SecretKey::from_slice(private_key_bytes)
        .map_err(|e| GrvtError::Signing(format!("invalid private key: {e}")))?;
    let pk = secp256k1::PublicKey::from_secret_key(&secp, &sk);
    let uncompressed = pk.serialize_uncompressed();
    let hash = keccak256(&uncompressed[1..]);
    Ok(format!("0x{}", hex::encode(&hash[12..])))
}

/// Parse a hex-encoded private key (with optional `0x` prefix) into raw bytes.
pub fn decode_private_key(hex_str: &str) -> Result<Vec<u8>> {
    let clean = hex_str.trim_start_matches("0x");
    hex::decode(clean).map_err(Into::into)
}

/// Parse an instrument_hash hex string into a [`U256`].
pub fn parse_instrument_hash(hash_hex: &str) -> Result<U256> {
    let raw = hash_hex.trim_start_matches("0x");
    let padded = if raw.len() % 2 == 1 {
        format!("0{raw}")
    } else {
        raw.to_string()
    };
    let bytes = hex::decode(&padded)?;
    if bytes.is_empty() {
        Ok(U256::ZERO)
    } else {
        Ok(U256::from_be_slice(&bytes))
    }
}

/// Convert a human-readable size to the integer representation used for signing.
pub fn scale_size(size: f64, base_decimals: u32) -> u64 {
    let multiplier = 10_u64.checked_pow(base_decimals).unwrap_or(1);
    (size * multiplier as f64) as u64
}

/// Convert a human-readable price to the 1e9-scaled integer used for signing.
pub fn scale_price(price: f64) -> u64 {
    (price * PRICE_SCALE) as u64
}

/// Sign an order using EIP-712 typed data.
///
/// The domain and struct definitions follow the GRVT Exchange specification
/// and are compatible with grvt-pysdk signing behaviour:
/// - `chain_id` is included in the EIP-712 domain but **omitted** from the
///   serialised `OrderSignature` payload (matching grvt-pysdk convention).
///
/// # Arguments
/// * `params`  - Structured signing parameters.
/// * `private_key_bytes` - Raw 32-byte secp256k1 private key.
pub fn sign_order(params: &SignOrderParams, private_key_bytes: &[u8]) -> Result<SignedOrder> {
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(private_key_bytes)
        .map_err(|e| GrvtError::Signing(format!("invalid private key: {e}")))?;

    let signer_address = address_from_private_key(private_key_bytes)?;

    let sol_legs: Vec<OrderLeg> = params
        .legs
        .iter()
        .map(|l| OrderLeg {
            assetID: l.asset_id,
            contractSize: l.contract_size,
            limitPrice: l.limit_price,
            isBuyingContract: l.is_buying_contract,
        })
        .collect();

    let order = Order {
        subAccountID: params.sub_account_id,
        isMarket: params.is_market,
        timeInForce: params.time_in_force.as_u8(),
        postOnly: params.post_only,
        reduceOnly: params.reduce_only,
        legs: sol_legs,
        nonce: params.nonce,
        expiration: params.expiration_ns,
    };

    let domain = Eip712Domain {
        name: Some("GRVT Exchange".into()),
        version: Some("0".into()),
        chain_id: Some(U256::from(params.chain_id)),
        verifying_contract: None,
        salt: None,
    };

    let digest: B256 = order.eip712_signing_hash(&domain);

    let msg = Message::from_slice(digest.as_ref())
        .map_err(|e| GrvtError::Signing(format!("message creation failed: {e}")))?;
    let recoverable: RecoverableSignature = secp.sign_ecdsa_recoverable(&msg, &secret_key);
    let (recid, compact) = recoverable.serialize_compact();
    let r = format!("0x{}", hex::encode(&compact[..32]));
    let s = format!("0x{}", hex::encode(&compact[32..]));
    let v: u8 = recid.to_i32() as u8 + 27;

    Ok(SignedOrder {
        signature: OrderSignature {
            signer: signer_address.clone(),
            r,
            s,
            v,
            expiration: params.expiration_ns.to_string(),
            nonce: params.nonce,
            chain_id: None, // grvt-pysdk compatible: omit chain_id from payload
        },
        signer_address,
        digest,
    })
}

/// Generate a random nonce suitable for order signing.
pub fn random_nonce() -> u32 {
    use rand::RngCore;
    rand::thread_rng().next_u32()
}

/// Compute a default expiration (now + 1 hour) in nanoseconds.
pub fn default_expiration_ns() -> i64 {
    let now_ns = chrono::Utc::now()
        .timestamp_nanos_opt()
        .unwrap_or(0);
    let exp = (now_ns as i128) + 60 * 60 * 1_000_000_000;
    exp.clamp(i64::MIN as i128, i64::MAX as i128) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PRIVATE_KEY: &str =
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

    #[test]
    fn test_address_derivation() {
        let key_bytes = decode_private_key(TEST_PRIVATE_KEY).unwrap();
        let addr = address_from_private_key(&key_bytes).unwrap();
        assert!(addr.starts_with("0x"));
        assert_eq!(addr.len(), 42);
        // Hardhat account #0
        assert_eq!(
            addr.to_lowercase(),
            "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
        );
    }

    #[test]
    fn test_parse_instrument_hash() {
        let hash = parse_instrument_hash("0x030501").unwrap();
        assert_eq!(hash, U256::from(0x030501u64));

        let hash_no_prefix = parse_instrument_hash("030501").unwrap();
        assert_eq!(hash_no_prefix, U256::from(0x030501u64));
    }

    #[test]
    fn test_scale_size() {
        assert_eq!(scale_size(1.5, 9), 1_500_000_000);
        assert_eq!(scale_size(0.001, 9), 1_000_000);
    }

    #[test]
    fn test_scale_price() {
        assert_eq!(scale_price(50000.0), 50_000_000_000_000);
    }

    #[test]
    fn test_sign_order_deterministic() {
        let key_bytes = decode_private_key(TEST_PRIVATE_KEY).unwrap();
        let params = SignOrderParams {
            sub_account_id: 12345,
            is_market: false,
            time_in_force: TimeInForce::GoodTillTime,
            post_only: false,
            reduce_only: false,
            legs: vec![SignOrderLeg {
                asset_id: U256::from(0x030501u64),
                contract_size: 1_000_000_000,
                limit_price: 50_000_000_000_000,
                is_buying_contract: true,
            }],
            nonce: 42,
            expiration_ns: 1_700_000_000_000_000_000,
            chain_id: 326,
        };

        let result1 = sign_order(&params, &key_bytes).unwrap();
        let result2 = sign_order(&params, &key_bytes).unwrap();

        assert_eq!(result1.digest, result2.digest);
        assert_eq!(result1.signature.r, result2.signature.r);
        assert_eq!(result1.signature.s, result2.signature.s);
        assert_eq!(result1.signature.v, result2.signature.v);
        assert!(result1.signature.chain_id.is_none());
    }

    #[test]
    fn test_tif_codes() {
        assert_eq!(TimeInForce::GoodTillTime.as_u8(), 1);
        assert_eq!(TimeInForce::ImmediateOrCancel.as_u8(), 2);
        assert_eq!(TimeInForce::FillOrKill.as_u8(), 3);
        assert_eq!(
            TimeInForce::GoodTillTime.as_api_str(),
            "GOOD_TILL_TIME"
        );
    }
}
