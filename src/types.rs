use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Generic response wrappers
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResult<T> {
    pub code: Option<String>,
    pub data: Option<T>,
    pub msg: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResultList<T> {
    pub result: Vec<T>,
}

// ---------------------------------------------------------------------------
// Order types (request)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    pub order: OrderPayload,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderPayload {
    pub sub_account_id: String,
    pub is_market: bool,
    pub time_in_force: String,
    pub post_only: bool,
    pub reduce_only: bool,
    pub legs: Vec<OrderLeg>,
    pub signature: OrderSignature,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<OrderMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub builder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub builder_fee: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderLeg {
    pub instrument: String,
    pub size: String,
    pub limit_price: Option<String>,
    pub is_buying_asset: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderSignature {
    pub signer: String,
    pub r: String,
    pub s: String,
    pub v: u8,
    pub expiration: String,
    pub nonce: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<OrderTrigger>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub broker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_position_transfer: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_crossing: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderTrigger {
    pub trigger_type: Option<String>,
    pub tpsl: Option<Tpsl>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tpsl {
    pub trigger_by: Option<String>,
    pub trigger_price: Option<String>,
    pub close_position: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CancelOrderRequest {
    pub sub_account_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CancelAllOrdersRequest {
    pub sub_account_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubAccountRequest {
    pub sub_account_id: String,
}

// ---------------------------------------------------------------------------
// Instrument types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct InstrumentRequest {
    pub instrument: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstrumentResponse {
    pub result: Option<InstrumentInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstrumentInfo {
    pub instrument: Option<String>,
    pub instrument_hash: Option<String>,
    pub base_decimals: Option<u32>,
    pub quote_decimals: Option<u32>,
    pub tick_size: Option<String>,
    pub min_size: Option<String>,
}

// ---------------------------------------------------------------------------
// Order response types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderResponse {
    pub result: OpenOrderItem,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenOrderItem {
    pub order_id: Option<String>,
    pub sub_account_id: Option<String>,
    pub is_market: Option<bool>,
    pub time_in_force: Option<String>,
    pub post_only: Option<bool>,
    pub reduce_only: Option<bool>,
    pub legs: Option<Vec<OrderLegResponse>>,
    pub signature: Option<OrderSignatureResponse>,
    pub metadata: Option<OrderMetadata>,
    pub state: Option<OrderState>,
    pub builder: Option<String>,
    pub builder_fee: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PositionItem {
    pub event_time: Option<String>,
    pub sub_account_id: Option<String>,
    pub instrument: Option<String>,
    pub size: Option<String>,
    pub notional: Option<String>,
    pub entry_price: Option<String>,
    pub exit_price: Option<String>,
    pub mark_price: Option<String>,
    pub unrealized_pnl: Option<String>,
    pub realized_pnl: Option<String>,
    pub total_pnl: Option<String>,
    pub roi: Option<String>,
    pub quote_index_price: Option<String>,
    pub est_liquidation_price: Option<String>,
    pub leverage: Option<String>,
    pub cumulative_fee: Option<String>,
    pub cumulative_realized_funding_payment: Option<String>,
    pub margin_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderLegResponse {
    pub instrument: Option<String>,
    pub size: Option<String>,
    pub limit_price: Option<String>,
    pub is_buying_asset: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderSignatureResponse {
    pub signer: Option<String>,
    pub r: Option<String>,
    pub s: Option<String>,
    pub v: Option<u8>,
    pub expiration: Option<String>,
    pub nonce: Option<u64>,
    pub chain_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderState {
    pub status: Option<String>,
    pub reject_reason: Option<String>,
    pub book_size: Option<Vec<String>>,
    pub traded_size: Option<Vec<String>>,
    pub update_time: Option<String>,
    pub avg_fill_price: Option<Vec<String>>,
}

// ---------------------------------------------------------------------------
// WebSocket event types
// ---------------------------------------------------------------------------

/// Envelope for `v1.state` stream events.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StateEvent {
    pub stream: Option<String>,
    pub selector: Option<String>,
    pub sequence_number: Option<String>,
    pub prev_sequence_number: Option<String>,
    pub feed: Option<StateFeed>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StateFeed {
    pub order_id: Option<String>,
    pub client_order_id: Option<String>,
    #[serde(rename = "order_state")]
    pub order_state: Option<OrderState>,
}

/// Typed market data events emitted by the WebSocket subscriber.
#[derive(Debug, Clone)]
pub enum MarketDataEvent {
    BookDelta {
        bids: Vec<PriceLevel>,
        asks: Vec<PriceLevel>,
    },
    Trade {
        price: f64,
        size: f64,
        event_time: i64,
        is_taker_buyer: Option<bool>,
    },
}

#[derive(Debug, Clone)]
pub struct PriceLevel {
    pub price: f64,
    pub size: f64,
}

/// Time-in-force constants (numeric codes used for EIP-712 signing).
/// Matches grvt-pysdk [`SignTimeInForce`](https://github.com/gravity-technologies/grvt-pysdk/blob/main/src/pysdk/grvt_raw_signing.py).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TimeInForce {
    GoodTillTime = 1,
    /// Block trades only (Python: `ALL_OR_NONE`).
    AllOrNone = 2,
    ImmediateOrCancel = 3,
    FillOrKill = 4,
}

impl TimeInForce {
    pub fn as_api_str(&self) -> &'static str {
        match self {
            Self::GoodTillTime => "GOOD_TILL_TIME",
            Self::AllOrNone => "ALL_OR_NONE",
            Self::ImmediateOrCancel => "IMMEDIATE_OR_CANCEL",
            Self::FillOrKill => "FILL_OR_KILL",
        }
    }

    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}
