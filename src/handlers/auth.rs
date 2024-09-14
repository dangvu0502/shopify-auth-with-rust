use axum::{
    extract::Query,
    response::{IntoResponse, Redirect},
};
use serde::Deserialize;
use crate::{config::Config, services::shopify};

#[derive(Deserialize)]
pub struct AuthQuery {
    shop: String,
}

pub async fn handle_auth(Query(query): Query<AuthQuery>) -> impl IntoResponse {
    let config = Config::from_env();
    let auth_url = shopify::build_auth_url(&query.shop, &config);
    Redirect::to(&auth_url)
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    shop: String,
    code: String,
}

pub async fn handle_callback(Query(query): Query<CallbackQuery>) -> impl IntoResponse {
    let config = Config::from_env();
    match shopify::exchange_code_for_token(&query.shop, &query.code, &config).await {
        Ok(access_token) => {
            format!("Authentication successful! Access token: {}", access_token)
        }
        Err(e) => {
            format!("Authentication failed: {}", e)
        }
    }
}