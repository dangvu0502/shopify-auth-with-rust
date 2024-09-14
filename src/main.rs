use axum::Router;

use dotenv::dotenv;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

mod config;
mod handlers;
mod routes;
mod services;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let app = Router::new()
        .merge(routes::dashboard::dashboard_routes())
        .merge(routes::auth::auth_routes())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    let addr = "localhost:3000";
    println!("Server running at: {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
