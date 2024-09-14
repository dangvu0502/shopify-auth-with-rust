use crate::handlers;
use axum::{routing::get, Router};

pub fn auth_routes() -> Router {
    Router::new()
        .route("/auth", get(handlers::auth::handle_auth))
        .route("/auth/callback", get(handlers::auth::handle_callback))
}
