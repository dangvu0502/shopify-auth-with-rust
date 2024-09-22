use std::{
    env,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    path::PathBuf, str::FromStr,
};

use axum::{
    body::{Body, Bytes},
    extract::State,
    http::{Request, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::get_service,
    Router,
};

use dotenv::dotenv;
use reqwest::Client;
use tower_http::{
    services::ServeDir,
    trace::{self, TraceLayer},
    cors::CorsLayer,
};
use tracing::{info, error, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Set up the router
    let app = Router::new()
        .fallback(handle_request)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any)
        );

        let config = config::Config::from_env();
        let addr = SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::from_str(&config.backend_host).expect("Invalid v4 address"),
            config
                .backend_port
                .parse::<u16>()
                .expect("Invalid port number"),
        ));
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        tracing::debug!("listening on {}", listener.local_addr().unwrap());
        axum::serve(listener, app.layer(TraceLayer::new_for_http()))
            .await
            .unwrap();
}

async fn handle_request(req: Request<Body>) -> Response {
    let path = req.uri().path();
    let query = req.uri().query().unwrap_or("");
    
    info!("Handling request for path: {}, query: {}", path, query);

    // Proxy all requests to the Vite development server
    proxy_to_vite(req).await
}

async fn proxy_to_vite(req: Request<Body>) -> Response {
    let client = Client::new();
    let vite_server = "http://localhost:5173";
    let uri = format!("{}{}", vite_server, req.uri().path_and_query().map(|x| x.as_str()).unwrap_or(""));

    info!("Proxying request to Vite server: {}", uri);

    match client.get(&uri).send().await {
        Ok(response) => {
            let status = response.status();
            let headers = response.headers().clone();
            let body = response.bytes().await.unwrap_or_default();

            info!("Received response from Vite server. Status: {}", status);

            let mut res = Response::builder().status(status);
            for (key, value) in headers.iter() {
                res = res.header(key, value);
            }
            res.body(Body::from(body)).unwrap_or_else(|_| Response::new(Body::empty()))
        }
        Err(e) => {
            error!("Error proxying to Vite server: {:?}", e);
            let error_message = format!("Failed to proxy request to Vite server at {}. Error: {}", vite_server, e);
            (StatusCode::BAD_GATEWAY, error_message).into_response()
        }
    }
}
