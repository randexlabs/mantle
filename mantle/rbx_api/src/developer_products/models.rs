use serde::Deserialize;

use crate::models::AssetId;

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeveloperProductConfigResponse {
    pub product_id: AssetId,
    pub name: String,
    pub description: String,
    pub icon_image_asset_id: Option<AssetId>,
    pub price_information: Option<PriceInformation>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PriceInformation {
    pub default_price_in_robux: Option<u32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDeveloperProductsResponse {
    pub developer_products: Vec<DeveloperProductConfigResponse>,
    pub next_page_token: Option<String>,
}
