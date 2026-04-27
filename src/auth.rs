use reqwest::Client;
use serde::Serialize;

use crate::config::{Environment, GrvtConfig};
use crate::error::{GrvtError, Result};

/// Authenticated session obtained after API-key login.
#[derive(Debug, Clone)]
pub struct AuthSession {
    pub account_id: String,
    pub session_cookie: String,
}

#[derive(Debug, Serialize)]
struct ApiKeyLoginRequest {
    api_key: String,
}

/// Perform API-key login against the GRVT edge endpoint and return an [`AuthSession`].
pub async fn login(client: &Client, env: &Environment, api_key: &str) -> Result<AuthSession> {
    let url = format!("{}/auth/api_key/login", env.auth_base());
    let body = ApiKeyLoginRequest {
        api_key: api_key.to_string(),
    };

    let resp = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Cookie", "rm=true;")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(GrvtError::ApiStatus { status, body: text });
    }

    let session_cookie = resp
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find_map(|v| v.split(';').next().filter(|c| c.starts_with("gravity=")).map(|c| c.to_string()))
        .ok_or(GrvtError::MissingSessionCookie)?;

    let account_id = resp
        .headers()
        .get("x-grvt-account-id")
        .ok_or(GrvtError::MissingAccountId)?
        .to_str()
        .map_err(|_| GrvtError::MissingAccountId)?
        .to_string();

    Ok(AuthSession { account_id, session_cookie })
}

/// Convenience: log in using the provided [`GrvtConfig`] and return an [`AuthSession`].
pub async fn login_from_config(config: &GrvtConfig) -> Result<AuthSession> {
    let client = Client::new();
    login(&client, &config.environment, &config.api_key).await
}
