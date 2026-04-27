use reqwest::header::{HeaderMap, HeaderValue};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::auth::{self, AuthSession};
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
    pub session_cookie: String,
    inner: reqwest::Client,
}

impl GrvtClient {
    /// Create a client from explicit session credentials.
    pub fn new(env: Environment, account_id: String, session_cookie: String) -> Self {
        Self {
            env,
            account_id,
            session_cookie,
            inner: reqwest::Client::new(),
        }
    }

    /// Create a client by logging in with an API key.
    pub async fn from_api_key(env: Environment, api_key: &str) -> Result<Self> {
        let inner = reqwest::Client::new();
        let session = auth::login(&inner, &env, api_key).await?;
        Ok(Self {
            env,
            account_id: session.account_id,
            session_cookie: session.session_cookie,
            inner,
        })
    }

    /// Create a client from a [`GrvtConfig`] (reads API key, performs login).
    pub async fn from_config(config: &GrvtConfig) -> Result<Self> {
        Self::from_api_key(config.environment, &config.api_key).await
    }

    /// Create a client from a pre-existing [`AuthSession`].
    pub fn from_session(env: Environment, session: AuthSession) -> Self {
        Self::new(env, session.account_id, session.session_cookie)
    }

    pub fn auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if let Ok(v) = HeaderValue::from_str(&self.account_id) {
            headers.insert("X-Grvt-Account-Id", v);
        }
        if let Ok(v) = HeaderValue::from_str(&self.session_cookie) {
            headers.insert("Cookie", v);
        }
        headers
    }

    // --- Full API ---

    pub async fn create_order_full(&self, body: &CreateOrderRequest) -> Result<CreateOrderResponse> {
        self.post(self.env.full_base(), "/full/v1/create_order", body).await
    }

    pub async fn cancel_order_full(&self, body: &CancelOrderRequest) -> Result<ApiResult<()>> {
        self.post(self.env.full_base(), "/full/v1/cancel_order", body).await
    }

    pub async fn cancel_all_orders_full(&self, body: &CancelAllOrdersRequest) -> Result<ApiResult<()>> {
        self.post(self.env.full_base(), "/full/v1/cancel_all_orders", body).await
    }

    pub async fn open_orders_full(&self, body: &SubAccountRequest) -> Result<ResultList<OpenOrderItem>> {
        self.post(self.env.full_base(), "/full/v1/open_orders", body).await
    }

    pub async fn positions_full(&self, body: &SubAccountRequest) -> Result<ResultList<PositionItem>> {
        self.post(self.env.full_base(), "/full/v1/positions", body).await
    }

    pub async fn instrument_full(&self, body: &InstrumentRequest) -> Result<InstrumentResponse> {
        self.post(self.env.market_data_base(), "/full/v1/instrument", body).await
    }

    // --- Lite API ---

    pub async fn create_order_lite(&self, body: &CreateOrderRequest) -> Result<CreateOrderResponse> {
        self.post(self.env.lite_base(), "/lite/v1/create_order", body).await
    }

    pub async fn cancel_order_lite(&self, body: &CancelOrderRequest) -> Result<ApiResult<()>> {
        self.post(self.env.lite_base(), "/lite/v1/cancel_order", body).await
    }

    pub async fn cancel_all_orders_lite(&self, body: &CancelAllOrdersRequest) -> Result<ApiResult<()>> {
        self.post(self.env.lite_base(), "/lite/v1/cancel_all_orders", body).await
    }

    pub async fn open_orders_lite(&self, body: &SubAccountRequest) -> Result<ResultList<OpenOrderItem>> {
        self.post(self.env.lite_base(), "/lite/v1/open_orders", body).await
    }

    pub async fn positions_lite(&self, body: &SubAccountRequest) -> Result<ResultList<PositionItem>> {
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
            .headers(self.auth_headers())
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
