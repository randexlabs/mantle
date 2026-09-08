pub mod models;

use reqwest::{header, StatusCode};
use serde_json::json;

use crate::{
    errors::{RobloxApiError, RobloxApiResult},
    helpers::{handle, handle_as_json},
    models::AssetId,
    RobloxApi,
};

use self::models::{CreateExperienceResponse, ExperienceConfigurationModel, GetExperienceResponse};

impl RobloxApi {
    pub async fn create_experience(
        &self,
        group_id: Option<AssetId>,
    ) -> RobloxApiResult<CreateExperienceResponse> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                let mut req = self
                    .client
                    .post("https://apis.roblox.com/universes/v1/universes/create")
                    .json(&json!({
                        "templatePlaceId": 95206881,
                    }));
                if let Some(group_id) = group_id {
                    req = req.query(&[("groupId", group_id.to_string())]);
                }
                Ok(req)
            })
            .await;

        handle_as_json(res).await
    }

    pub async fn get_experience(
        &self,
        experience_id: AssetId,
    ) -> RobloxApiResult<GetExperienceResponse> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self.client.get(format!(
                    "https://develop.roblox.com/v1/universes/{}",
                    experience_id
                )))
            })
            .await;

        handle_as_json(res).await
    }

    pub async fn get_experience_configuration(
        &self,
        experience_id: AssetId,
    ) -> RobloxApiResult<ExperienceConfigurationModel> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self.client.get(format!(
                    "https://develop.roblox.com/v1/universes/{}/configuration",
                    experience_id
                )))
            })
            .await;

        handle_as_json(res).await
    }

    pub async fn configure_experience(
        &self,
        experience_id: AssetId,
        experience_configuration: &ExperienceConfigurationModel,
    ) -> RobloxApiResult<()> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self
                    .client
                    .patch(format!(
                        "https://develop.roblox.com/v2/universes/{}/configuration",
                        experience_id
                    ))
                    .json(experience_configuration))
            })
            .await;

        handle(res).await?;

        Ok(())
    }

    pub async fn set_experience_active(
        &self,
        experience_id: AssetId,
        active: bool,
    ) -> RobloxApiResult<()> {
        let endpoint = if active { "activate" } else { "deactivate" };
        let client =
            self.open_cloud_client()
                .ok_or_else(|| RobloxApiError::OpenCloudApiKeyRequired {
                    operation: format!("universe {} {}", experience_id, endpoint),
                    scope: "legacy-universe:manage".to_owned(),
                })?;
        let response = self
            .send_open_cloud_request(
                "POST",
                client
                    .post(format!(
                        "https://apis.roblox.com/legacy-develop/v1/universes/{}/{}",
                        experience_id, endpoint
                    ))
                    .header(header::CONTENT_LENGTH, 0),
            )
            .await
            .map_err(|error| match error {
                RobloxApiError::Roblox {
                    status_code,
                    request_method,
                    request_url,
                    reason,
                } if status_code == StatusCode::UNAUTHORIZED => RobloxApiError::Roblox {
                    status_code,
                    request_method,
                    request_url,
                    reason: format!(
                        "{reason}; verify the Open Cloud API key is active and was copied without quotes or a session-cookie value"
                    ),
                },
                RobloxApiError::Roblox {
                    status_code,
                    request_method,
                    request_url,
                    reason,
                } if status_code == StatusCode::FORBIDDEN => RobloxApiError::Roblox {
                    status_code,
                    request_method,
                    request_url,
                    reason: format!(
                        "{reason}; verify the key has the legacy-universe:manage scope and access to this universe"
                    ),
                },
                error => error,
            })?;
        drop(response);

        Ok(())
    }
}
