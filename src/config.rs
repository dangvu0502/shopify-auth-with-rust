use std::env;

pub struct Config {
    pub shopify_api_key: String,
    pub shopify_api_secret: String,
    pub shopify_scopes: String,
    pub base_uri: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            shopify_api_key: env::var("SHOPIFY_API_KEY").expect("SHOPIFY_API_KEY must be set"),
            shopify_api_secret: env::var("SHOPIFY_API_SECRET").expect("SHOPIFY_API_SECRET must be set"),
            shopify_scopes: env::var("SHOPIFY_SCOPES").expect("SHOPIFY_SCOPES must be set"),
            base_uri: env::var("BASE_URL").expect("BASE_URI must be set"),
        }
    }
}
