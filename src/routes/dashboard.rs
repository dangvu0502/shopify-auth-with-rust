use axum::{
    routing::get,
    Router,
};
use crate::handlers;

pub fn dashboard_routes() -> Router {
    Router::new()
        .route("/", get(handlers::dashboard::handle_dashboard))
}