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
        let response = self
            .send_authenticated_request("POST", move |client| {
                Ok(client.post(url.clone()).multipart(
                    Form::new()
                        .text("name", name.clone())
                        .text("description", description.clone())
                        .text("isForSale", "true")
                        .text("price", price.to_string()),
                ))
            })
            .await?;

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
        let response = self
            .send_authenticated_request("GET", move |client| {
                let mut request = client.get(url.clone()).query(&[("pageSize", "100")]);
                if let Some(page_token) = &page_token {
                    request = request.query(&[("pageToken", page_token)]);
                }
                Ok(request)
            })
            .await?;

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
        let response = self
            .send_authenticated_request("PATCH", move |client| {
                Ok(client.patch(url.clone()).multipart(
                    Form::new()
                        .text("name", name.clone())
                        .text("description", description.clone())
                        .text("isForSale", "true")
                        .text("price", price.to_string()),
                ))
            })
            .await?;
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
        let response = self
            .send_authenticated_request("PATCH", move |client| {
                let image = Part::bytes(file_data.clone())
                    .file_name(file_name.clone())
                    .mime_str(&mime)?;
                Ok(client
                    .patch(url.clone())
                    .multipart(Form::new().part("imageFile", image)))
            })
            .await?;
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
        let response = self
            .send_authenticated_request("GET", move |client| Ok(client.get(url.clone())))
            .await?;

        handle_response_as_json_with_method(response, "GET").await
    }
}
