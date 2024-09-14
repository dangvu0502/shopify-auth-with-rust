use axum::{
    extract::Query,
    response::{IntoResponse, Redirect},
};
use crate::services::shopify;

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