use axum::{body::Body, http::Request, middleware::{self, Next}, response::{IntoResponse, Redirect}, Router};

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
        .merge(routes::auth::auth_routes())
        .layer(middleware::from_fn(auth_middleware)) 
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );
    
    let config = config::Config::from_env();
    let addr = format!("{}:{}", config.backend_host, config.backend_port);
    println!("Server running at: {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

pub async fn auth_middleware(req: Request<Body>, next: Next) -> impl IntoResponse
{
    let path = req.uri().path();

    // Bypass the auth middleware for specific routes
    if path == "/auth" || path == "/auth/callback" {
        return next.run(req).await;
    }

    if is_authenticated(&req) {
        next.run(req).await
    } else {
        Redirect::temporary("/auth").into_response()
    }
}

fn is_authenticated<B>(req: &Request<B>) -> bool {
    if let Some(cookie) = req.headers().get("cookie") {
        return cookie.to_str().unwrap_or("").contains("auth_token");
    }
    false
}