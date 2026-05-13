use reqwest::header::{HeaderMap, HeaderValue};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering::SeqCst};
use std::sync::Arc;
use time::OffsetDateTime;
use tokio::sync::RwLock;

use crate::auth;
use crate::config::{Environment, GrvtConfig};
use crate::error::{GrvtError, Result};
use crate::types::*;

/// Authenticated REST client for the GRVT API.
///
/// Supports both full and lite endpoint variants for order management,
/// position queries, and instrument metadata.
#[derive(Debug, Clone)]
pub struct GrvtClient {
    pub env: Environment,
    pub account_id: String,
    pub session_time: Arc<AtomicU64>,
    pub session_cookie: Arc<RwLock<Arc<String>>>,
    inner: reqwest::Client,
    api_key: String,
}

impl GrvtClient {
    /// Create a client by logging in with an API key.
    pub async fn from_api_key(env: Environment, api_key: &str) -> Result<Self> {
        let inner = reqwest::Client::new();
        let session = auth::login(&inner, &env, api_key).await?;
        Ok(Self {
            env,
            account_id: session.account_id,
            session_time: Arc::new(AtomicU64::new((OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000) as u64)),
            session_cookie: Arc::new(RwLock::new(Arc::new(session.session_cookie))),
            inner,
            api_key: api_key.into(),
        })
    }

    /// Create a client from a [`GrvtConfig`] (reads API key, performs login).
    pub async fn from_config(config: &GrvtConfig) -> Result<Self> {
        Self::from_api_key(config.environment, &config.api_key).await
    }

    pub async fn get_session_cookie(&self) -> Arc<String> {
        let now = (OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000) as u64;
        if now > self.session_time.load(SeqCst) + 600_000 {
            self.session_time.store(now, SeqCst);
            let Ok(session) = auth::login(&self.inner, &self.env, &self.api_key).await else {
                return self.session_cookie.read().await.clone();
            };
            let session_cookie = Arc::new(session.session_cookie);
            *self.session_cookie.write().await = session_cookie.clone();
            session_cookie
        } else {
            self.session_cookie.read().await.clone()
        }
    }

    pub async fn auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if let Ok(v) = HeaderValue::from_str(&self.account_id) {
            headers.insert("X-Grvt-Account-Id", v);
        }
        if let Ok(v) = HeaderValue::from_str(&self.get_session_cookie().await) {
            headers.insert("Cookie", v);
        }
        headers
    }

    // --- Full API ---

    pub async fn funding_account_summary_full(&self) -> Result<FundingAccountSummaryResponse> {
        self.post(self.env.full_base(), "/full/v1/funding_account_summary", &None::<()>).await
    }

    pub async fn account_summary_full(&self, body: &AccountSummaryRequest) -> Result<ResultItem<AccountSummary>> {
        self.post(self.env.full_base(), "/full/v1/account_summary", body).await
    }

    pub async fn create_order_full(&self, body: &CreateOrderRequest) -> Result<CreateOrderResponse> {
        self.post(self.env.full_base(), "/full/v1/create_order", body).await
    }

    pub async fn get_order_full(&self, body: &GetOrderRequest) -> Result<CreateOrderResponse> {
        self.post(self.env.full_base(), "/full/v1/order", body).await
    }

    pub async fn cancel_order_full(&self, body: &CancelOrderRequest) -> Result<ResultItem<Ack>> {
        self.post(self.env.full_base(), "/full/v1/cancel_order", body).await
    }

    pub async fn cancel_all_orders_full(&self, body: &CancelAllOrdersRequest) -> Result<ResultItem<Ack>> {
        self.post(self.env.full_base(), "/full/v1/cancel_all_orders", body).await
    }

    pub async fn open_orders_full(&self, body: &SubAccountRequest) -> Result<ResultList<OpenOrderItem>> {
        self.post(self.env.full_base(), "/full/v1/open_orders", body).await
    }

    pub async fn positions_full(&self, body: &PosotionsRequest) -> Result<ResultList<PositionItem>> {
        self.post(self.env.full_base(), "/full/v1/positions", body).await
    }

    pub async fn instrument_full(&self, body: &InstrumentRequest) -> Result<ResultItem<InstrumentInfo>> {
        self.post(self.env.market_data_base(), "/full/v1/instrument", body).await
    }

    pub async fn ticker_full(&self, body: &InstrumentRequest) -> Result<ResultItem<TickerInfo>> {
        self.post(self.env.market_data_base(), "/full/v1/ticker", body).await
    }

    pub async fn book_full(&self, body: &BookRequest) -> Result<ResultItem<Book>> {
        self.post(self.env.market_data_base(), "/full/v1/book", body).await
    }

    pub async fn funding_full(&self, body: &FundingRequest) -> Result<ResultList<FundingRate>> {
        self.post(self.env.market_data_base(), "/full/v1/funding", body).await
    }

    // --- Lite API ---

    pub async fn create_order_lite(&self, body: &CreateOrderRequest) -> Result<CreateOrderResponse> {
        self.post(self.env.lite_base(), "/lite/v1/create_order", body).await
    }

    pub async fn cancel_order_lite(&self, body: &CancelOrderRequest) -> Result<ResultItem<Ack>> {
        self.post(self.env.lite_base(), "/lite/v1/cancel_order", body).await
    }

    pub async fn cancel_all_orders_lite(&self, body: &CancelAllOrdersRequest) -> Result<ResultItem<Ack>> {
        self.post(self.env.lite_base(), "/lite/v1/cancel_all_orders", body).await
    }

    pub async fn open_orders_lite(&self, body: &SubAccountRequest) -> Result<ResultList<OpenOrderItem>> {
        self.post(self.env.lite_base(), "/lite/v1/open_orders", body).await
    }

    pub async fn positions_lite(&self, body: &PosotionsRequest) -> Result<ResultList<PositionItem>> {
        self.post(self.env.lite_base(), "/lite/v1/positions", body).await
    }

    // --- internal helpers ---

    async fn post<T: Serialize, U: DeserializeOwned>(&self, base: &str, path: &str, body: &T) -> Result<U> {
        let url = format!("{base}{path}");

        if tracing::enabled!(tracing::Level::TRACE) {
            if let Ok(raw) = serde_json::to_string(body) {
                tracing::trace!(path, body = %truncate(&raw, 4096), "GRVT request body");
            }
        }
        tracing::debug!(method = "POST", path, "GRVT HTTP request");

        let raw_body = serde_json::to_vec(body)?;
        let resp = self
            .inner
            .post(&url)
            .headers(self.auth_headers().await)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(raw_body)
            .send()
            .await?;

        handle_response(path, resp).await
    }
}

async fn handle_response<T: DeserializeOwned>(path: &str, resp: reqwest::Response) -> Result<T> {
    let status = resp.status();
    let text = resp.text().await?;
    if status.is_success() {
        tracing::debug!(status = %status, path, body_len = text.len(), "GRVT response OK");
        tracing::trace!(body = %truncate(&text, 4096), "GRVT response body");
        let parsed: T = serde_json::from_str(&text)?;
        Ok(parsed)
    } else {
        tracing::warn!(status = %status, path, body = %truncate(&text, 4096), "GRVT error response");
        Err(GrvtError::ApiStatus { status, body: text })
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}... (truncated, {} bytes total)", &s[..max], s.len())
    }
}
