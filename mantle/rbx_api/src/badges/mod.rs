pub mod models;

use std::path::PathBuf;

use reqwest::multipart::Form;
use serde_json::json;

use crate::{
    errors::{RobloxApiError, RobloxApiResult},
    helpers::{get_file_part, handle_as_json, handle_response_as_json_with_method},
    models::{AssetId, CreatorType, UploadImageResponse},
    RobloxApi,
};

use self::models::{CreateBadgeResponse, ListBadgeResponse, ListBadgesResponse};

impl RobloxApi {
    pub async fn create_badge(
        &self,
        experience_id: AssetId,
        name: String,
        description: String,
        icon_file_path: PathBuf,
        payment_source: CreatorType,
        expected_cost: u32,
    ) -> RobloxApiResult<CreateBadgeResponse> {
        let icon_file = get_file_part(&icon_file_path).await?;
        let client =
            self.open_cloud_client()
                .ok_or_else(|| RobloxApiError::OpenCloudApiKeyRequired {
                    operation: format!("badge creation for universe {}", experience_id),
                    scope: "legacy-universe.badge:manage-and-spend-robux".to_owned(),
                })?;
        let payment_source_type = match payment_source {
            CreatorType::User => 1,
            CreatorType::Group => 2,
        };
        let response = self
            .send_open_cloud_request(
                "POST",
                client
                    .post(format!(
                        "https://apis.roblox.com/legacy-badges/v1/universes/{}/badges",
                        experience_id
                    ))
                    .multipart(
                        Form::new()
                            .part("files", icon_file)
                            .text("name", name)
                            .text("description", description)
                            .text("paymentSourceType", payment_source_type.to_string())
                            .text("expectedCost", expected_cost.to_string())
                            .text("isActive", "true"),
                    ),
            )
            .await
            .map_err(|error| {
                error.with_required_scope("legacy-universe.badge:manage-and-spend-robux")
            })?;

        handle_response_as_json_with_method(response, "POST").await
    }

    pub async fn update_badge(
        &self,
        badge_id: AssetId,
        name: String,
        description: String,
        enabled: bool,
    ) -> RobloxApiResult<()> {
        let client =
            self.open_cloud_client()
                .ok_or_else(|| RobloxApiError::OpenCloudApiKeyRequired {
                    operation: format!("badge {} update", badge_id),
                    scope: "legacy-universe.badge:write".to_owned(),
                })?;
        let response = self
            .send_open_cloud_request(
                "PATCH",
                client
                    .patch(format!(
                        "https://apis.roblox.com/legacy-badges/v1/badges/{}",
                        badge_id
                    ))
                    .json(&json!({
                        "name": name,
                        "description": description,
                        "enabled": enabled,
                    })),
            )
            .await
            .map_err(|error| error.with_required_scope("legacy-universe.badge:write"))?;
        drop(response);

        Ok(())
    }

    pub async fn get_create_badge_free_quota(
        &self,
        experience_id: AssetId,
    ) -> RobloxApiResult<i32> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self.client.get(format!(
                    "https://badges.roblox.com/v1/universes/{}/free-badges-quota",
                    experience_id
                )))
            })
            .await;

        handle_as_json(res).await
    }

    pub async fn list_badges(
        &self,
        experience_id: AssetId,
        page_cursor: Option<String>,
    ) -> RobloxApiResult<ListBadgesResponse> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                let mut req = self.client.get(format!(
                    "https://badges.roblox.com/v1/universes/{}/badges",
                    experience_id
                ));
                if let Some(page_cursor) = &page_cursor {
                    req = req.query(&[("cursor", page_cursor)]);
                }
                Ok(req)
            })
            .await;

        handle_as_json(res).await
    }

    pub async fn get_all_badges(
        &self,
        experience_id: AssetId,
    ) -> RobloxApiResult<Vec<ListBadgeResponse>> {
        let mut all_badges = Vec::new();

        let mut page_cursor: Option<String> = None;
        loop {
            let res = self.list_badges(experience_id, page_cursor).await?;
            all_badges.extend(res.data);

            if res.next_page_cursor.is_none() {
                break;
            }

            page_cursor = res.next_page_cursor;
        }

        Ok(all_badges)
    }

    pub async fn update_badge_icon(
        &self,
        badge_id: AssetId,
        icon_file: PathBuf,
    ) -> RobloxApiResult<UploadImageResponse> {
        let icon_file = get_file_part(&icon_file).await?;
        let client =
            self.open_cloud_client()
                .ok_or_else(|| RobloxApiError::OpenCloudApiKeyRequired {
                    operation: format!("badge {} icon update", badge_id),
                    scope: "legacy-badge:manage".to_owned(),
                })?;
        let response = self
            .send_open_cloud_request(
                "POST",
                client
                    .post(format!(
                        "https://apis.roblox.com/legacy-publish/v1/badges/{}/icon",
                        badge_id
                    ))
                    .multipart(Form::new().part("Files", icon_file)),
            )
            .await
            .map_err(|error| error.with_required_scope("legacy-badge:manage"))?;

        handle_response_as_json_with_method(response, "POST").await
    }
}
