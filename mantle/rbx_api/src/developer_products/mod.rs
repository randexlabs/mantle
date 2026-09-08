pub mod models;

use std::path::PathBuf;

use reqwest::multipart::{Form, Part};

use crate::{
    errors::RobloxApiResult,
    helpers::{get_file_data, handle_response_as_json_with_method},
    models::AssetId,
    RobloxApi,
};

use self::models::{DeveloperProductConfigResponse, ListDeveloperProductsResponse};

impl RobloxApi {
    pub async fn create_developer_product(
        &self,
        experience_id: AssetId,
        name: String,
        price: u32,
        description: String,
    ) -> RobloxApiResult<DeveloperProductConfigResponse> {
        let url = format!(
            "https://apis.roblox.com/developer-products/v2/universes/{}/developer-products",
            experience_id
        );
        let client = self.open_cloud_client_required(
            format!("developer product creation for universe {}", experience_id),
            "developer-product:write",
        )?;
        let response = self
            .send_open_cloud_request(
                "POST",
                client.post(url).multipart(
                    Form::new()
                        .text("name", name)
                        .text("description", description)
                        .text("isForSale", "true")
                        .text("price", price.to_string()),
                ),
            )
            .await
            .map_err(|error| error.with_required_scope("developer-product:write"))?;

        handle_response_as_json_with_method(response, "POST").await
    }

    pub async fn list_developer_products(
        &self,
        experience_id: AssetId,
        page_token: Option<String>,
    ) -> RobloxApiResult<ListDeveloperProductsResponse> {
        let url = format!(
            "https://apis.roblox.com/developer-products/v2/universes/{}/developer-products/creator",
            experience_id
        );
        let client = self.open_cloud_client_required(
            format!("developer product listing for universe {}", experience_id),
            "developer-product:read",
        )?;
        let mut request = client.get(url).query(&[("pageSize", "100")]);
        if let Some(page_token) = &page_token {
            request = request.query(&[("pageToken", page_token)]);
        }
        let response = self
            .send_open_cloud_request("GET", request)
            .await
            .map_err(|error| error.with_required_scope("developer-product:read"))?;

        handle_response_as_json_with_method(response, "GET").await
    }

    pub async fn get_all_developer_products(
        &self,
        experience_id: AssetId,
    ) -> RobloxApiResult<Vec<DeveloperProductConfigResponse>> {
        let mut all_products = Vec::new();
        let mut page_token = None;

        loop {
            let response = self
                .list_developer_products(experience_id, page_token)
                .await?;
            all_products.extend(response.developer_products);

            if response.next_page_token.is_none() {
                break;
            }
            page_token = response.next_page_token;
        }

        Ok(all_products)
    }

    pub async fn update_developer_product(
        &self,
        experience_id: AssetId,
        product_id: AssetId,
        name: String,
        price: u32,
        description: String,
    ) -> RobloxApiResult<()> {
        let url = format!(
            "https://apis.roblox.com/developer-products/v2/universes/{}/developer-products/{}",
            experience_id, product_id
        );
        let client = self.open_cloud_client_required(
            format!("developer product {} update", product_id),
            "developer-product:write",
        )?;
        let response = self
            .send_open_cloud_request(
                "PATCH",
                client.patch(url).multipart(
                    Form::new()
                        .text("name", name)
                        .text("description", description)
                        .text("isForSale", "true")
                        .text("price", price.to_string()),
                ),
            )
            .await
            .map_err(|error| error.with_required_scope("developer-product:write"))?;
        drop(response);

        Ok(())
    }

    pub async fn create_developer_product_icon(
        &self,
        experience_id: AssetId,
        product_id: AssetId,
        icon_file: PathBuf,
    ) -> RobloxApiResult<DeveloperProductConfigResponse> {
        let (file_data, file_name, mime) = get_file_data(&icon_file).await?;
        let url = format!(
            "https://apis.roblox.com/developer-products/v2/universes/{}/developer-products/{}",
            experience_id, product_id
        );
        let client = self.open_cloud_client_required(
            format!("developer product {} icon update", product_id),
            "developer-product:write",
        )?;
        let image = Part::bytes(file_data)
            .file_name(file_name)
            .mime_str(&mime)?;
        let response = self
            .send_open_cloud_request(
                "PATCH",
                client
                    .patch(url)
                    .multipart(Form::new().part("imageFile", image)),
            )
            .await
            .map_err(|error| error.with_required_scope("developer-product:write"))?;
        drop(response);

        self.get_developer_product(experience_id, product_id).await
    }

    pub async fn get_developer_product(
        &self,
        experience_id: AssetId,
        product_id: AssetId,
    ) -> RobloxApiResult<DeveloperProductConfigResponse> {
        let url = format!(
            "https://apis.roblox.com/developer-products/v2/universes/{}/developer-products/{}/creator",
            experience_id, product_id
        );
        let client = self.open_cloud_client_required(
            format!("developer product {} lookup", product_id),
            "developer-product:read",
        )?;
        let response = self
            .send_open_cloud_request("GET", client.get(url))
            .await
            .map_err(|error| error.with_required_scope("developer-product:read"))?;

        handle_response_as_json_with_method(response, "GET").await
    }
}
