use crate::{config::Config, services::shopify};
use axum::{
    extract::Query,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use serde::Deserialize;
use tracing::info;

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
    info!("{}", auth_url);
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
        (status = 302, description = "Redirects to frontend on success", content_type = "text/plain"),
        (status = 400, description = "Authentication failed", content_type = "text/plain")
    )
)]
pub async fn handle_callback(
    Query(query): Query<CallbackQuery>,
    jar: CookieJar,
) -> impl IntoResponse {
    let config = Config::from_env();

    // Validate the query parameters
    if query.shop.is_empty() || !query.shop.ends_with(".myshopify.com") {
        return (StatusCode::BAD_REQUEST, "Invalid Shopify store domain").into_response();
    }

    if query.code.is_empty() {
        return (StatusCode::BAD_REQUEST, "Missing authorization code").into_response();
    }

    match shopify::exchange_code_for_token(&query.shop, &query.code, &config).await {
        Ok(access_token) => {
            let mut jar = jar;
            println!("{}", access_token.clone());
            jar = jar.add(Cookie::new("auth_token", access_token));
            let redirect_url = Redirect::to(&format!("/?shop={}", query.shop));
            let mut headers = HeaderMap::new();
            for cookie in jar.iter() {
                headers.insert(
                    header::SET_COOKIE,
                    header::HeaderValue::from_str(&cookie.to_string()).unwrap(),
                );
            }
            (headers, redirect_url).into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            format!("Authentication failed: {}", e),
        )
            .into_response(),
    }
}
