use axum::Router;

use dotenv::dotenv;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod api_docs;
mod config;
mod handlers;
mod routes;
mod services;

#[tokio::main]
async fn main() {
    dotenv().ok();

    #[derive(OpenApi)]
    #[openapi(
        paths(
            handlers::dashboard::handle_dashboard,
            handlers::auth::handle_auth,
            handlers::auth::handle_callback
        ),
        components(
            schemas(api_docs::ShopQuery)
        ),
        tags(
            (name = "shopify", description = "Shopify authentication API")
        )
    )]
    struct ApiDoc;

    let filter = filter::Targets::new()
        .with_target("tower_http::trace::on_response", Level::TRACE)
        .with_target("tower_http::trace::on_request", Level::TRACE)
        .with_target("tower_http::trace::make_span", Level::DEBUG)
        .with_default(Level::INFO);

    let tracing_layer = tracing_subscriber::fmt::layer();

    tracing_subscriber::registry()
        .with(tracing_layer)
        .with(filter)
        .init();

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
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
