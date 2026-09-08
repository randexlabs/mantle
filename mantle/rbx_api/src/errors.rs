use rbx_auth::CsrfTokenRequestError;
use reqwest::StatusCode;
use serde::Deserialize;
use thiserror::Error;

use crate::models::AssetTypeId;

// TODO: Improve some of these error messages.
#[derive(Error, Debug)]
pub enum RobloxApiError {
    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error(transparent)]
    RequestFactoryError(#[from] CsrfTokenRequestError),

    #[error("Authorization has been denied for this request. Check your ROBLOSECURITY cookie.")]
    Authorization,

    #[error(
        "Open Cloud API key is required for {operation}; configure ROBLOX_OPEN_CLOUD_API_KEY (or the legacy MANTLE_OPEN_CLOUD_API_KEY alias) with the {scope} scope."
    )]
    OpenCloudApiKeyRequired { operation: String, scope: String },

    #[error("Roblox API request failed: {request_method} {request_url} ({status_code}): {reason}")]
    Roblox {
        status_code: StatusCode,
        request_method: String,
        request_url: String,
        reason: String,
    },

    #[error("Failed to parse JSON response: {0}")]
    ParseJson(#[from] serde_json::Error),

    #[error("Failed to parse HTML response.")]
    ParseHtml,

    #[error("Failed to parse AssetId.")]
    ParseAssetId,

    #[error("Failed to read file: {0}")]
    ReadFile(#[from] std::io::Error),

    #[error("Failed to determine file name for path {0}.")]
    NoFileName(String),

    #[error("Invalid file extension for path {0}.")]
    InvalidFileExtension(String),

    #[error("Failed to read utf8 data: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),

    #[error("No create quotas found for asset type {0}")]
    MissingCreateQuota(AssetTypeId),

    #[error("Place file size is too large. Consider switching to the rbxl format.")]
    RbxlxPlaceFileSizeTooLarge,

    #[error("Place file size may be too large. Consider switching to the rbxl format.")]
    RbxlxPlaceFileSizeMayBeTooLarge,

    #[error("Place file size is too large.")]
    RbxlPlaceFileSizeTooLarge,

    #[error("Place file size may be too large.")]
    RbxlPlaceFileSizeMayBeTooLarge,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

// Temporary to make the new errors backwards compatible with the String errors throughout the project.
impl From<RobloxApiError> for String {
    fn from(e: RobloxApiError) -> Self {
        e.to_string()
    }
}

pub type RobloxApiResult<T> = Result<T, RobloxApiError>;

#[derive(Deserialize, Debug)]
pub struct RobloxApiErrorResponse {
    // There are some other possible properties but we currently have no use for them so they are not
    // included

    // Most error models have a `message` property
    #[serde(alias = "Message")]
    pub message: Option<String>,

    // Some error models (500) have a `title` property instead
    #[serde(alias = "Title")]
    pub title: Option<String>,

    // Some error models on older APIs have an errors array
    #[serde(alias = "Errors")]
    pub errors: Option<Vec<RobloxApiErrorResponse>>,

    // Some errors return a `success` property which can be used to check for errors
    #[serde(alias = "Success")]
    pub success: Option<bool>,

    // Open Cloud APIs use an errorCode/errorMessage pair.
    #[serde(rename = "errorCode", alias = "ErrorCode")]
    pub error_code: Option<String>,

    #[serde(rename = "errorMessage", alias = "ErrorMessage")]
    pub error_message: Option<String>,

    pub field: Option<String>,
    pub hint: Option<String>,
}

impl RobloxApiErrorResponse {
    pub fn reason(self) -> Option<String> {
        let mut reasons = Vec::new();

        if let Some(error_code) = self.error_code {
            reasons.push(format!("code: {}", error_code));
        }
        if let Some(error_message) = self.error_message {
            reasons.push(error_message);
        }
        if let Some(field) = self.field {
            reasons.push(format!("field: {}", field));
        }
        if let Some(hint) = self.hint {
            reasons.push(format!("hint: {}", hint));
        }
        if let Some(message) = self.message {
            reasons.push(message);
        }
        if let Some(title) = self.title {
            reasons.push(title);
        }
        if let Some(errors) = self.errors {
            for error in errors {
                if let Some(message) = error.reason() {
                    reasons.push(message);
                }
            }
        }
        if self.success == Some(false) {
            reasons.push("success: false".to_owned());
        }

        (!reasons.is_empty()).then(|| reasons.join("; "))
    }
}
