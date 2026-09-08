pub mod models;

use std::{ffi::OsStr, fs, path::PathBuf, time::Duration};

use anyhow::anyhow;
use models::{Asset, CreateAssetRequest, Creator, OperationResult};
use reqwest::{header, multipart::Form};
use serde::de;
use serde_json::json;
use tokio::time::sleep;

use crate::{
    errors::{RobloxApiError, RobloxApiResult},
    helpers::{get_file_part, handle, handle_as_json, handle_with_method},
    models::{AssetId, AssetTypeId, CreatorType},
    RobloxApi,
};

use self::models::{CreateAssetQuota, CreateAssetQuotasResponse, CreateAudioAssetResponse};

async fn handle_operation<T: de::DeserializeOwned>(
    result: RobloxApiResult<reqwest::Response>,
) -> RobloxApiResult<OperationResult<T>> {
    let body = result?.text().await?;
    let operation_result: OperationResult<T> = serde_json::from_str(&body)?;

    match operation_result.error {
        None => Ok(operation_result),
        Some(error) => Err(anyhow!("{}: {}", error.error, error.message).into()),
    }
}

impl RobloxApi {
    pub async fn create_image_asset(
        &self,
        file_path: PathBuf,
        creator: Creator,
    ) -> RobloxApiResult<AssetId> {
        let request = serde_json::to_string(&CreateAssetRequest {
            asset_type: "Image".to_string(),
            display_name: file_path.display().to_string(),
            description: file_path.display().to_string(),
            creation_context: models::CreateAssetContext { creator },
        })?;

        let res = if let Some(client) = self.open_cloud_client() {
            let response = self
                .send_open_cloud_request(
                    "POST",
                    client
                        .post("https://apis.roblox.com/assets/v1/assets")
                        .multipart(
                            Form::new()
                                .text("request", request.clone())
                                .part("fileContent", get_file_part(&file_path).await?),
                        ),
                )
                .await
                .map_err(|error| error.with_required_scope("asset:write"))?;

            Ok(response)
        } else {
            let result = self
                .csrf_token_store
                .send_request(|| async {
                    Ok(self
                        .client
                        .post("https://apis.roblox.com/assets/user-auth/v1/assets")
                        .multipart(
                            Form::new()
                                .text("request", request.clone())
                                .part("fileContent", get_file_part(&file_path).await?),
                        ))
                })
                .await;

            handle_with_method(result, "POST").await
        };

        let mut attempts_remaining = 5;
        let mut sleep_duration = Duration::from_millis(500);
        let mut operation_result: OperationResult<Asset> = handle_operation(res).await?;
        while !operation_result.done && attempts_remaining > 0 {
            sleep(sleep_duration).await;
            let operation_path = operation_result.path.as_ref().ok_or_else(|| {
                RobloxApiError::Other(anyhow!(
                    "Roblox asset operation is pending but did not return a polling path."
                ))
            })?;
            let res = if let Some(client) = self.open_cloud_client() {
                let response = self
                    .send_open_cloud_request(
                        "GET",
                        client.get(format!(
                            "https://apis.roblox.com/assets/v1/{}",
                            operation_path
                        )),
                    )
                    .await
                    .map_err(|error| error.with_required_scope("asset:read"))?;

                Ok(response)
            } else {
                let result = self
                    .csrf_token_store
                    .send_request(|| async {
                        Ok(self.client.get(format!(
                            "https://apis.roblox.com/assets/user-auth/v1/{}",
                            operation_path
                        )))
                    })
                    .await;

                handle_with_method(result, "GET").await
            };

            operation_result = handle_operation(res).await?;
            sleep_duration = sleep_duration.mul_f32(1.5);
            attempts_remaining -= 1;
        }

        if !operation_result.done {
            return Err(RobloxApiError::Other(anyhow!(
                "Roblox asset operation did not complete after polling."
            )));
        }

        let asset = operation_result.response.ok_or_else(|| {
            RobloxApiError::Other(anyhow!(
                "Roblox asset operation completed without returning an asset."
            ))
        })?;

        asset
            .asset_id
            .parse()
            .map_err(|_| RobloxApiError::ParseAssetId)
    }

    pub async fn get_create_asset_quota(
        &self,
        asset_type: AssetTypeId,
    ) -> RobloxApiResult<CreateAssetQuota> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self
                    .client
                    .get("https://publish.roblox.com/v1/asset-quotas")
                    .query(&[
                        // TODO: Understand what this parameter does
                        ("resourceType", "1"),
                        ("assetType", &asset_type.to_string()),
                    ]))
            })
            .await;

        // TODO: Understand how to interpret multiple quota objects (rather than just using the first one)
        (handle_as_json::<CreateAssetQuotasResponse>(res).await?)
            .quotas
            .first()
            .cloned()
            .ok_or(RobloxApiError::MissingCreateQuota(asset_type))
    }

    pub async fn create_audio_asset(
        &self,
        file_path: PathBuf,
        group_id: Option<AssetId>,
        payment_source: CreatorType,
    ) -> RobloxApiResult<CreateAudioAssetResponse> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                let data = fs::read(&file_path)?;

                let file_name = format!(
                    "Audio/{}",
                    file_path.file_stem().and_then(OsStr::to_str).unwrap()
                );

                Ok(self
                    .client
                    .post("https://publish.roblox.com/v1/audio")
                    .json(&json!({
                        "name": file_name,
                        "file": base64::encode(data),
                        "groupId": group_id,
                        "paymentSource": payment_source
                    })))
            })
            .await;

        handle_as_json(res).await
    }

    pub async fn archive_asset(&self, asset_id: AssetId) -> RobloxApiResult<()> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self
                    .client
                    .post(format!(
                        "https://develop.roblox.com/v1/assets/{}/archive",
                        asset_id
                    ))
                    .header(header::CONTENT_LENGTH, 0))
            })
            .await;

        handle(res).await?;

        Ok(())
    }
}
