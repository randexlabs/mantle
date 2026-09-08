use serde::Deserialize;

use crate::models::AssetId;

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetGamePassResponse {
    #[serde(rename = "gamePassId")]
    pub target_id: AssetId,
    pub name: String,
    pub description: String,
    #[serde(rename = "iconAssetId")]
    pub icon_image_asset_id: AssetId,
    pub price_information: Option<PriceInformation>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PriceInformation {
    pub default_price_in_robux: Option<u32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListGamePassesResponse {
    pub next_page_token: Option<String>,
    pub game_passes: Vec<GetGamePassResponse>,
}
