use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct ShopQuery {
    pub shop: Option<String>,
}
