pub mod models;

use std::path::PathBuf;

use reqwest::multipart::Form;
use reqwest::StatusCode;
use serde_json::json;

use crate::{
    errors::{RobloxApiError, RobloxApiResult},
    helpers::{get_file_part, handle, handle_as_json},
    models::{AssetId, UploadImageResponse},
    RobloxApi,
};

use self::models::{GetExperienceThumbnailResponse, GetExperienceThumbnailsResponse};

impl RobloxApi {
    pub async fn upload_icon(
        &self,
        experience_id: AssetId,
        icon_file: PathBuf,
    ) -> RobloxApiResult<UploadImageResponse> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self
                    .client
                    .post(&format!(
                        "https://publish.roblox.com/v1/games/{}/icon",
                        experience_id
                    ))
                    .multipart(Form::new().part("request.files", get_file_part(&icon_file).await?)))
            })
            .await;

        handle_as_json(res).await
    }

    pub async fn upload_thumbnail(
        &self,
        experience_id: AssetId,
        thumbnail_file: PathBuf,
    ) -> RobloxApiResult<UploadImageResponse> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self
                    .client
                    .post(&format!(
                        "https://publish.roblox.com/v1/games/{}/thumbnail/image",
                        experience_id
                    ))
                    .multipart(
                        Form::new().part("request.files", get_file_part(&thumbnail_file).await?),
                    ))
            })
            .await;

        handle_as_json(res).await
    }

    pub async fn remove_experience_icon(&self, experience_id: AssetId) -> RobloxApiResult<()> {
        let language_code = "en";
        let url = format!(
            "https://apis.roblox.com/legacy-game-internationalization/v1/game-icon/games/{}/language-codes/{}",
            experience_id, language_code
        );
        let client = self.open_cloud_client_required(
            format!("experience {} icon removal", experience_id),
            "legacy-universe:manage",
        )?;
        let result = self
            .send_open_cloud_request("DELETE", client.delete(url))
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(RobloxApiError::Roblox {
                status_code: StatusCode::NOT_FOUND,
                ..
            }) => Ok(()),
            Err(RobloxApiError::Roblox {
                status_code: StatusCode::BAD_REQUEST,
                reason,
                ..
            }) if reason.contains("source language") => Ok(()),
            Err(error) => Err(error.with_required_scope("legacy-universe:manage")),
        }
    }

    pub async fn get_experience_thumbnails(
        &self,
        experience_id: AssetId,
    ) -> RobloxApiResult<Vec<GetExperienceThumbnailResponse>> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self.client.get(format!(
                    "https://games.roblox.com/v2/games/{}/media",
                    experience_id
                )))
            })
            .await;

        Ok(handle_as_json::<GetExperienceThumbnailsResponse>(res)
            .await?
            .data)
    }

    pub async fn set_experience_thumbnail_order(
        &self,
        experience_id: AssetId,
        new_thumbnail_order: &[AssetId],
    ) -> RobloxApiResult<()> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self
                    .client
                    .post(format!(
                        "https://develop.roblox.com/v1/universes/{}/thumbnails/order",
                        experience_id
                    ))
                    .json(&json!({ "thumbnailIds": new_thumbnail_order })))
            })
            .await;

        handle(res).await?;

        Ok(())
    }

    pub async fn delete_experience_thumbnail(
        &self,
        experience_id: AssetId,
        thumbnail_id: AssetId,
    ) -> RobloxApiResult<()> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self.client.delete(format!(
                    "https://develop.roblox.com/v1/universes/{}/thumbnails/{}",
                    experience_id, thumbnail_id
                )))
            })
            .await;

        handle(res).await?;

        Ok(())
    }
}
