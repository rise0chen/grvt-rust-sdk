use futures::{SinkExt, StreamExt};
use serde_json::json;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};

use crate::config::Environment;
use crate::error::{GrvtError, Result};
use crate::types::{MarketDataEvent, PriceLevel, StateEvent};

fn ws_err(e: impl std::fmt::Debug) -> GrvtError {
    GrvtError::WebSocket(format!("{e:?}"))
}

// ---------------------------------------------------------------------------
// Market data WebSocket
// ---------------------------------------------------------------------------

/// Subscribe to `v1.book.d` and `v1.trade` streams for the given instrument.
///
/// Returns a channel receiver that yields [`MarketDataEvent`] values.
/// The WebSocket read loop runs in a spawned task; dropping the receiver
/// will cause the task to terminate on the next send attempt.
pub async fn subscribe_market_data(env: &Environment, instrument: &str, buffer: usize) -> Result<mpsc::Receiver<MarketDataEvent>> {
    let ws_url = env.market_data_ws();
    let request = ws_url.into_client_request().map_err(ws_err)?;

    let (ws_stream, _resp) = connect_async(request).await.map_err(ws_err)?;
    let (mut write, mut read) = ws_stream.split();

    let selector_book = format!("{instrument}@500");
    let selector_trade = instrument.to_string();

    let sub_book = json!({
        "jsonrpc": "2.0",
        "method": "subscribe",
        "params": { "stream": "v1.book.d", "selectors": [selector_book] },
        "id": 1
    })
    .to_string();
    let sub_trade = json!({
        "jsonrpc": "2.0",
        "method": "subscribe",
        "params": { "stream": "v1.trade", "selectors": [selector_trade] },
        "id": 2
    })
    .to_string();

    write.send(WsMessage::Text(sub_book.into())).await.map_err(ws_err)?;
    write.send(WsMessage::Text(sub_trade.into())).await.map_err(ws_err)?;

    let (tx, rx) = mpsc::channel(buffer);

    tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            let msg = match msg {
                Ok(m) => m,
                Err(e) => {
                    tracing::warn!(error = ?e, "market data ws read error");
                    break;
                }
            };

            if let WsMessage::Text(text) = msg {
                tracing::trace!(raw = %text, "market data raw");
                if let Some(event) = parse_market_data_message(&text) {
                    if tx.send(event).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    Ok(rx)
}

fn parse_market_data_message(text: &str) -> Option<MarketDataEvent> {
    let v: serde_json::Value = serde_json::from_str(text).ok()?;

    let stream = v
        .get("stream")
        .and_then(|s| s.as_str())
        .or_else(|| v.get("params").and_then(|p| p.get("stream")).and_then(|s| s.as_str()))
        .unwrap_or("");

    match stream {
        "v1.book.d" => {
            let book = v.get("feed").or_else(|| v.get("result"))?;
            let bids = parse_price_levels(book.get("bids"));
            let asks = parse_price_levels(book.get("asks"));
            Some(MarketDataEvent::BookDelta { bids, asks })
        }
        "v1.trade" => {
            let feed = v.get("feed")?;
            let price = feed.get("price").and_then(|x| x.as_str()).and_then(|p| p.parse::<f64>().ok())?;
            let size = feed.get("size").and_then(|x| x.as_str()).and_then(|s| s.parse::<f64>().ok())?;
            let event_time = feed.get("event_time").and_then(|x| x.as_str()).and_then(|t| t.parse::<i64>().ok())?;
            let is_taker_buyer = feed.get("is_taker_buyer").and_then(|x| x.as_bool());

            Some(MarketDataEvent::Trade {
                price,
                size,
                event_time,
                is_taker_buyer,
            })
        }
        _ => None,
    }
}

fn parse_price_levels(arr: Option<&serde_json::Value>) -> Vec<PriceLevel> {
    let Some(arr) = arr.and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|entry| {
            let price = entry.get("price").and_then(|x| x.as_str()).and_then(|p| p.parse::<f64>().ok())?;
            let size = entry.get("size").and_then(|x| x.as_str()).and_then(|s| s.parse::<f64>().ok())?;
            Some(PriceLevel { price, size })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// State (authenticated) WebSocket
// ---------------------------------------------------------------------------

/// Subscribe to the authenticated `v1.state` stream.
///
/// Returns a channel receiver that yields [`StateEvent`] values.
/// The caller must supply the session cookie and account ID for auth headers.
pub async fn subscribe_state(
    env: &Environment,
    session_cookie: &str,
    account_id: &str,
    selectors: Vec<String>,
    buffer: usize,
) -> Result<mpsc::Receiver<StateEvent>> {
    let mut request = env.full_ws().into_client_request().map_err(ws_err)?;

    {
        let headers = request.headers_mut();
        headers.insert("Cookie", session_cookie.parse().map_err(ws_err)?);
        headers.insert("X-Grvt-Account-Id", account_id.parse().map_err(ws_err)?);
    }

    let (ws_stream, _resp) = connect_async(request).await.map_err(ws_err)?;
    let (mut write, mut read) = ws_stream.split();

    let subscribe_msg = json!({
        "jsonrpc": "2.0",
        "method": "subscribe",
        "params": {
            "stream": "v1.state",
            "selectors": selectors,
        },
        "id": 1
    })
    .to_string();

    write.send(WsMessage::Text(subscribe_msg.into())).await.map_err(ws_err)?;

    let (tx, rx) = mpsc::channel(buffer);

    tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            let msg = match msg {
                Ok(m) => m,
                Err(e) => {
                    tracing::warn!(error = ?e, "state ws read error");
                    break;
                }
            };

            if let WsMessage::Text(text) = msg {
                tracing::trace!(raw = %text, "state event raw");
                if let Ok(event) = serde_json::from_str::<StateEvent>(&text) {
                    if tx.send(event).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    Ok(rx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_book_delta() {
        let msg = r#"{
            "stream": "v1.book.d",
            "feed": {
                "bids": [
                    {"price": "50000", "size": "1.5"},
                    {"price": "49900", "size": "0"}
                ],
                "asks": [
                    {"price": "50100", "size": "2.0"}
                ]
            }
        }"#;

        let event = parse_market_data_message(msg).unwrap();
        match event {
            MarketDataEvent::BookDelta { bids, asks } => {
                assert_eq!(bids.len(), 2);
                assert_eq!(bids[0].price, 50000.0);
                assert_eq!(bids[0].size, 1.5);
                assert_eq!(bids[1].size, 0.0);
                assert_eq!(asks.len(), 1);
                assert_eq!(asks[0].price, 50100.0);
            }
            _ => panic!("expected BookDelta"),
        }
    }

    #[test]
    fn test_parse_trade() {
        let msg = r#"{
            "stream": "v1.trade",
            "feed": {
                "price": "50050",
                "size": "0.5",
                "event_time": "1700000000000",
                "is_taker_buyer": true
            }
        }"#;

        let event = parse_market_data_message(msg).unwrap();
        match event {
            MarketDataEvent::Trade {
                price,
                size,
                event_time,
                is_taker_buyer,
            } => {
                assert_eq!(price, 50050.0);
                assert_eq!(size, 0.5);
                assert_eq!(event_time, 1_700_000_000_000);
                assert_eq!(is_taker_buyer, Some(true));
            }
            _ => panic!("expected Trade"),
        }
    }

    #[test]
    fn test_parse_unknown_stream_returns_none() {
        let msg = r#"{"stream": "v1.candle", "feed": {}}"#;
        assert!(parse_market_data_message(msg).is_none());
    }

    #[test]
    fn test_parse_state_event() {
        let msg = r#"{
            "stream": "v1.state",
            "selector": "123-BTC_USDT_Perp",
            "sequence_number": "1",
            "feed": {
                "order_id": "0xabc",
                "order_state": {
                    "status": "FILLED",
                    "avg_fill_price": ["50000000000000"]
                }
            }
        }"#;

        let event: StateEvent = serde_json::from_str(msg).unwrap();
        assert_eq!(event.stream.as_deref(), Some("v1.state"));
        let feed = event.feed.unwrap();
        assert_eq!(feed.order_id.as_deref(), Some("0xabc"));
        let state = feed.order_state.unwrap();
        assert_eq!(state.status.as_deref(), Some("FILLED"));
    }
}
