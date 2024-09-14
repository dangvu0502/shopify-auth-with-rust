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

#[utoipa::path(
    get,
    path = "/auth",
    params(
        ("shop" = String, Query, description = "Shopify store domain")
    ),
    responses(
        (status = 302, description = "Redirect to Shopify OAuth page", content_type = "text/plain")
    )
)]
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


#[utoipa::path(
    get,
    path = "/auth/callback",
    params(
        ("shop" = String, Query, description = "Shopify store domain"),
        ("code" = String, Query, description = "Authorization code")
    ),
    responses(
        (status = 200, description = "Successfully authenticated", content_type = "text/plain"),
        (status = 400, description = "Authentication failed", content_type = "text/plain")
    )
)]
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