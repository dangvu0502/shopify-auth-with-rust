use crate::config::Config;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct ShopQuery {
    pub shop: Option<String>,
}

pub fn build_auth_url(shop: &str, config: &Config) -> String {
    format!(
        "https://{}/admin/oauth/authorize?client_id={}&scope={}&redirect_uri={}/auth/callback",
        shop, config.shopify_api_key, config.shopify_scopes, config.backend_host,
    )
}

pub async fn exchange_code_for_token(
    shop: &str,
    code: &str,
    config: &Config,
) -> Result<String, reqwest::Error> {
    let client = Client::new();
    let res = client
        .post(format!("https://{}/admin/oauth/access_token", shop))
        .json(&json!({
            "client_id": config.shopify_api_key,
            "client_secret": config.shopify_api_secret,
            "code": code,
        }))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    Ok(res["access_token"].as_str().unwrap_or("").to_string())
}

pub fn is_authenticated(shop: &str) -> bool {
    // TODO: Implement actual authentication check
    // This could involve checking a database for a valid access token
    false // For now, always return false to force authentication
}
