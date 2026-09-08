pub mod models;

use std::path::PathBuf;

use reqwest::multipart::{Form, Part};

use crate::{
    errors::RobloxApiResult,
    helpers::{get_file_data, handle_response_as_json_with_method},
    models::AssetId,
    RobloxApi,
};

use self::models::{GetGamePassResponse, ListGamePassesResponse};

impl RobloxApi {
    pub async fn list_game_passes(
        &self,
        experience_id: AssetId,
        page_token: Option<String>,
    ) -> RobloxApiResult<ListGamePassesResponse> {
        let url = format!(
            "https://apis.roblox.com/game-passes/v1/universes/{}/game-passes/creator",
            experience_id
        );
        let client = self.open_cloud_client_required(
            format!("game pass listing for universe {}", experience_id),
            "game-pass:read",
        )?;
        let mut request = client.get(url).query(&[("pageSize", "100")]);
        if let Some(page_token) = &page_token {
            request = request.query(&[("pageToken", page_token)]);
        }
        let response = self
            .send_open_cloud_request("GET", request)
            .await
            .map_err(|error| error.with_required_scope("game-pass:read"))?;

        handle_response_as_json_with_method(response, "GET").await
    }

    pub async fn get_game_pass(
        &self,
        experience_id: AssetId,
        game_pass_id: AssetId,
    ) -> RobloxApiResult<GetGamePassResponse> {
        let url = format!(
            "https://apis.roblox.com/game-passes/v1/universes/{}/game-passes/{}/creator",
            experience_id, game_pass_id
        );
        let client = self.open_cloud_client_required(
            format!("game pass {} lookup", game_pass_id),
            "game-pass:read",
        )?;
        let response = self
            .send_open_cloud_request("GET", client.get(url))
            .await
            .map_err(|error| error.with_required_scope("game-pass:read"))?;

        handle_response_as_json_with_method(response, "GET").await
    }

    pub async fn get_all_game_passes(
        &self,
        experience_id: AssetId,
    ) -> RobloxApiResult<Vec<GetGamePassResponse>> {
        let mut all_game_passes = Vec::new();
        let mut page_token = None;

        loop {
            let response = self.list_game_passes(experience_id, page_token).await?;
            all_game_passes.extend(response.game_passes);

            if response.next_page_token.is_none() {
                break;
            }
            page_token = response.next_page_token;
        }

        Ok(all_game_passes)
    }

    pub async fn create_game_pass(
        &self,
        experience_id: AssetId,
        name: String,
        description: String,
        price: Option<u32>,
        icon_file: PathBuf,
    ) -> RobloxApiResult<GetGamePassResponse> {
        let (file_data, file_name, mime) = get_file_data(&icon_file).await?;
        let url = format!(
            "https://apis.roblox.com/game-passes/v1/universes/{}/game-passes",
            experience_id
        );
        let client = self.open_cloud_client_required(
            format!("game pass creation for universe {}", experience_id),
            "game-pass:write",
        )?;
        let image = Part::bytes(file_data)
            .file_name(file_name)
            .mime_str(&mime)?;
        let mut form = Form::new()
            .text("name", name)
            .text("description", description)
            .part("imageFile", image);
        if let Some(price) = price {
            form = form
                .text("isForSale", "true")
                .text("price", price.to_string());
        } else {
            form = form.text("isForSale", "false");
        }
        let response = self
            .send_open_cloud_request("POST", client.post(url).multipart(form))
            .await
            .map_err(|error| error.with_required_scope("game-pass:write"))?;

        handle_response_as_json_with_method(response, "POST").await
    }

    pub async fn update_game_pass(
        &self,
        experience_id: AssetId,
        game_pass_id: AssetId,
        name: String,
        description: String,
        price: Option<u32>,
        icon_file: Option<PathBuf>,
    ) -> RobloxApiResult<GetGamePassResponse> {
        let file_data = match icon_file {
            Some(path) => Some(get_file_data(&path).await?),
            None => None,
        };
        let url = format!(
            "https://apis.roblox.com/game-passes/v1/universes/{}/game-passes/{}",
            experience_id, game_pass_id
        );
        let client = self.open_cloud_client_required(
            format!("game pass {} update", game_pass_id),
            "game-pass:write",
        )?;
        let mut form = Form::new()
            .text("name", name)
            .text("description", description)
            .text("isForSale", price.is_some().to_string());
        if let Some(price) = price {
            form = form.text("price", price.to_string());
        }
        if let Some((file_data, file_name, mime)) = file_data {
            let image = Part::bytes(file_data)
                .file_name(file_name)
                .mime_str(&mime)?;
            form = form.part("imageFile", image);
        }
        let response = self
            .send_open_cloud_request("PATCH", client.patch(url).multipart(form))
            .await
            .map_err(|error| error.with_required_scope("game-pass:write"))?;
        drop(response);

        self.get_game_pass(experience_id, game_pass_id)
            .await
            .map_err(|error| error.with_required_scope("game-pass:read"))
    }
}
