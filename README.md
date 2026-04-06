# grvt-rust-sdk

Rust SDK for [Gravity Markets (GRVT)](https://grvt.io) REST API, WebSocket streams, and EIP-712 order signing. Compatible with the [grvt-pysdk](https://github.com/gravity-technologies/grvt-pysdk) signing conventions.

> **Disclaimer:** This is an **unofficial**, community-maintained project. It is **not** affiliated with, endorsed by, or supported by Gravity Markets or Gravity Technologies. **The SDK is incomplete and under active development**—APIs may change, behavior may be wrong, and production use is at your own risk. There is no warranty; refer to upstream GRVT documentation for authoritative behavior.

## Features

- **REST API**: Order creation/cancellation, positions, open orders, instrument metadata (full and lite variants)
- **Authentication**: API-key login with session cookie management
- **EIP-712 Signing**: Order signing compatible with GRVT Exchange specification
- **WebSocket**: Market data (`v1.book.d`, `v1.trade`) and authenticated state stream (`v1.state`)

## Requirements

- Rust 1.70+
- Tokio runtime (async)

## Installation

Add as a path dependency in your `Cargo.toml`:

```toml
[dependencies]
grvt-rust-sdk = { path = "../grvt-rust-sdk" }
tokio = { version = "1", features = ["full"] }
```

## Environment Variables

The SDK can be configured via environment variables (compatible with grvt-pysdk):

| Variable | Required | Description |
|----------|----------|-------------|
| `GRVT_ENV` | No | `prod`, `testnet`, `staging`, or `dev` (default: `testnet`) |
| `GRVT_API_KEY` | Yes* | API key (generic fallback) |
| `GRVT_API_KEY_TESTNET` | Yes* | Testnet API key |
| `GRVT_API_KEY_PROD` | Yes* | Production API key |
| `GRVT_TRADING_ACCOUNT_ID` | Yes* | Sub-account ID for trading |
| `GRVT_SUB_ACCOUNT_ID` | Yes* | Alternative to `GRVT_TRADING_ACCOUNT_ID` |
| `GRVT_PRIVATE_KEY` | For signing | Hex-encoded private key for order signing |
| `GRVT_CHAIN_ID` | No | Chain ID (default per environment) |

\* At least one API key and one account ID variable must be set.

## Quick Start

Runnable examples live in the `examples/` directory. Copy `.env.example` to `.env` and fill in your credentials.

```bash
# Configure environment variables (copy and edit)
cp .env.example .env

# Run examples
cargo run --example rest_client_env      # List positions (config from environment)
cargo run --example rest_client_config  # REST client (programmatic configuration)
cargo run --example order_signing       # EIP-712 order signing
cargo run --example create_and_cancel   # Place order → cancel by client_order_id
cargo run --example ws_market_data      # WebSocket market data
cargo run --example ws_state            # WebSocket state stream (authenticated)
```

| Example | Description |
|---------|-------------|
| `rest_client_env` | Load configuration from the environment and list positions |
| `rest_client_config` | Configure the client with `GrvtConfig::builder()` |
| `order_signing` | EIP-712 order signing (`GRVT_PRIVATE_KEY` required) |
| `create_and_cancel` | Fetch instrument metadata → sign → submit order → cancel by `client_order_id` |
| `ws_market_data` | Subscribe to order book and trade streams |
| `ws_state` | Subscribe to order state (authenticated) |

## API Overview

### REST Client (`GrvtClient`)

| Method | Description |
|--------|-------------|
| `create_order_full` / `create_order_lite` | Create limit/market order |
| `cancel_order_full` / `cancel_order_lite` | Cancel by order_id or client_order_id |
| `cancel_all_orders_full` / `cancel_all_orders_lite` | Cancel all orders for sub-account |
| `open_orders_full` / `open_orders_lite` | List open orders |
| `positions_full` / `positions_lite` | List positions |
| `instrument_full` | Get instrument metadata (hash, decimals, tick size) |

### Signer (`grvt_rust_sdk::signer`)

| Function | Description |
|----------|-------------|
| `sign_order` | EIP-712 sign order params |
| `address_from_private_key` | Derive signer address from secret key |
| `decode_private_key` | Parse hex private key to bytes |
| `parse_instrument_hash` | Parse instrument_hash to U256 |
| `scale_size` | Convert size to contract units |
| `scale_price` | Convert price to 1e9 units |
| `random_nonce` | Generate random nonce |
| `default_expiration_ns` | Default expiration (now + 1h) |

### WebSocket (`grvt_rust_sdk::ws`)

| Function | Description |
|----------|-------------|
| `subscribe_market_data` | Subscribe to book delta + trades |
| `subscribe_state` | Subscribe to order state (auth required) |

## Error Handling

All SDK operations return `Result<T, GrvtError>`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum GrvtError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API returned {status}: {body}")]
    ApiStatus { status: StatusCode, body: String },
    #[error("JSON deserialization failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("missing session cookie in login response")]
    MissingSessionCookie,
    #[error("missing account ID in login response")]
    MissingAccountId,
    #[error("configuration error: {0}")]
    Config(String),
    #[error("signing error: {0}")]
    Signing(String),
    #[error("WebSocket error: {0}")]
    WebSocket(String),
    #[error("hex decode error: {0}")]
    Hex(#[from] hex::FromHexError),
}
```

## Testing

```bash
cd grvt-rust-sdk
cargo test
```

## Compatibility Notes

- **grvt-pysdk**: Signature payload omits `chain_id` (included in EIP-712 domain only). TIF codes for EIP-712 match `SignTimeInForce`: `GOOD_TILL_TIME=1`, `ALL_OR_NONE=2`, `IMMEDIATE_OR_CANCEL=3`, `FILL_OR_KILL=4`.
- **Environments**: Prod, Testnet, Staging, Dev with distinct endpoints for trades, market-data, and auth.
