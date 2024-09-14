use crate::services::shopify;
use axum::{
    extract::Query,
    response::{IntoResponse, Redirect},
};

#[utoipa::path(
    get,
    path = "/",
    params(
        ("shop" = Option<String>, Query, description = "Shopify store domain")
    ),
    responses(
        (status = 200, description = "Dashboard view", content_type = "text/plain"),
        (status = 302, description = "Redirect to authentication", content_type = "text/plain"),
        (status = 400, description = "Missing shop parameter", content_type = "text/plain")
    )
)]

pub async fn handle_dashboard(Query(params): Query<shopify::ShopQuery>) -> impl IntoResponse {
    if let Some(shop) = params.shop {
        if shopify::is_authenticated(&shop) {
            format!("Welcome to your dashboard for shop: {}", shop).into_response()
        } else {
            Redirect::to(&format!("/auth?shop={}", shop)).into_response()
        }
    } else {
        "Please provide a shop parameter".into_response()
    }
}
