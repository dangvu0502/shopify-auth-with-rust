use axum::{
    body::Body,
    extract::Query,
    http::{Request, Response},
    middleware::{self, Next},
    response::{Html, Redirect},
    routing::get,
    Router,
};
use reqwest::{header::CONTENT_TYPE, Client, StatusCode};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const APPLICATION_GRAPHQL: &str = "application/graphql";
const SHOPIFY_ACCESS_TOKEN: &str = "X-Shopify-Access-Token";

const QUERY: &str = r#"
{
  shop {
    name
  }
}"#;

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber for logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // build our application with a single route
    let app = Router::new()
        .route("/", get(index))
        .route("/install", get(auth))
        .route("/api/auth/callback", get(auth_callback))
        .layer(middleware::from_fn(log_requests));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Sample handler for the root route
async fn hello_world() -> &'static str {
    "Hello, World!"
}

#[derive(Debug, serde::Deserialize)]
struct ShopifyAuthenticationQueryParams {
    hmac: String,
    shop: String,
    timestamp: String,
}

async fn auth(Query(params): Query<ShopifyAuthenticationQueryParams>) -> Redirect {
    println!("The route matches, that's progress...");
    let api_key = "986cd6ee289ffc61da6dbd77539ec120"; // Obviously don't use _MY_ key, swap this out for your own
    let scopes = "read_products,read_product_listings,read_orders,read_customers"; // These will be custom for you depending on your goals
    let nonce = "random-value";

    let redirect = format!("https://{shop}/admin/oauth/authorize?client_id={client_id}&scope={scopes}&redirect_uri={redirect_uri}&state={nonce}&grant_options[]={access_mode}",
		shop = params.shop,
		client_id = api_key,
		scopes = scopes,
		redirect_uri = "https://pf.dangvh.com/api/auth/callback", // Redirecting to oblivion
		nonce = nonce,
		access_mode = "value");

    Redirect::to(&redirect)
}

#[derive(Debug, serde::Deserialize)]
struct ShopifyAuthenticationCallbackQueryParams {
    code: String,
    shop: String,
}

#[derive(serde::Serialize)]
struct AccessTokenRequest<'a> {
    client_id: &'a str,
    client_secret: &'a str,
    code: &'a str,
}

#[derive(serde::Deserialize, Debug)]
struct AccessTokenResponse {
    access_token: String,
    scope: String,
}

async fn auth_callback(
    Query(params): Query<ShopifyAuthenticationCallbackQueryParams>,
) -> Result<Redirect, (StatusCode, &'static str)> {
    let access_token_uri = format!(
        "https://{shop}/admin/oauth/access_token",
        shop = params.shop
    );

    let body = AccessTokenRequest {
        client_id: "find it in shopify partner dashboard",
        client_secret: "find it in shopify partner dashboard",
        code: &params.code,
    };

    let client = Client::new();
    let response = client
        .post(&access_token_uri)
        .json(&body)
        .send()
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Error sending request"))?;

    println!("Response status: {}", response.status());

    let response_body: AccessTokenResponse = response.json().await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error getting response body as text",
        )
    })?;

    println!("{:?}", response_body);

    Ok(Redirect::to("/"))
}

async fn index() -> Result<Html<String>, (StatusCode, &'static str)> {
    let admin_api_uri = format!(
        "https://{}.myshopify.com/admin/api/2024-07/graphql.json",
        "dang06"
    );

    let client = reqwest::Client::new();
    let response = client
        .post(admin_api_uri)
        .header(CONTENT_TYPE, APPLICATION_GRAPHQL)
        .header(
            SHOPIFY_ACCESS_TOKEN,
            "use your own token",
        )
        .body(QUERY)
        .send()
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Error sending request"))?;

    let response_body: serde_json::Value = response
        .json()
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Error sending request"))?;

    let pretty_json = serde_json::to_string_pretty(&response_body)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Error sending request"))?;

    println!("Response is: {:#?}", pretty_json);
    let shop_name = &response_body["data"]["shop"]["name"].as_str().unwrap();

    Ok(Html(build_index_html(shop_name)))
}

fn build_index_html(shop: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>R'al cool Shopify app</title>
  <script>
    function doFancyFrontEndThings() {{
      window.alert("Ya dun clicked the button");
    }}
  </script>
</head>
<body>
  <h1>Welcome to this Shopify Application, {}</h1>
  <button onclick="doFancyFrontEndThings()">Click me!</button>
</body>
</html>"#,
        shop,
    )
}

// Custom middleware function to log requests and responses
async fn log_requests(req: Request<Body>, next: Next) -> Response<Body> {
    // Log the incoming request
    let method = req.method().clone();
    let uri = req.uri().clone();
    info!("Received request: {} {}", method, uri);

    // Call the next middleware or handler
    let response = next.run(req).await;

    // Log the response status
    let status = response.status();
    info!("Response status: {}", status);

    response
}
