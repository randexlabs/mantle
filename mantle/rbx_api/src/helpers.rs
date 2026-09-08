use std::{ffi::OsStr, path::Path};

use log::{debug, trace};
use rbx_auth::CsrfTokenRequestError;
use reqwest::{header::HeaderMap, multipart::Part, Body, StatusCode};
use scraper::{Html, Selector};
use serde::de;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};
use url::Url;

use crate::{errors::RobloxApiErrorResponse, RobloxApiError, RobloxApiResult};

const MAX_ERROR_BODY_LENGTH: usize = 2048;

pub async fn get_roblox_api_error_from_response_with_method(
    response: reqwest::Response,
    request_method: &str,
) -> RobloxApiError {
    let status_code = response.status();
    let request_url = sanitize_url(response.url());
    let headers = response.headers().clone();
    let body = response.text().await.unwrap_or_default();

    parse_roblox_api_error(status_code, request_method, &request_url, &headers, &body)
}

fn sanitize_url(url: &Url) -> String {
    let mut sanitized = url.clone();
    sanitized.set_query(None);
    sanitized.to_string()
}

fn truncate_body(body: &str) -> Option<String> {
    let body = body.trim();
    if body.is_empty() {
        return None;
    }

    let mut truncated = body.chars().take(MAX_ERROR_BODY_LENGTH).collect::<String>();
    if body.chars().count() > MAX_ERROR_BODY_LENGTH {
        truncated.push('…');
    }
    Some(truncated)
}

fn parse_html_error(body: &str) -> Option<String> {
    let html = Html::parse_fragment(body);
    let selector = Selector::parse(".request-error-page-content .error-message").ok()?;
    html.select(&selector)
        .next()
        .map(|element| {
            element
                .text()
                .map(|text| text.trim())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|message| !message.is_empty())
}

fn parse_roblox_api_error(
    status_code: StatusCode,
    request_method: &str,
    request_url: &str,
    headers: &HeaderMap,
    body: &str,
) -> RobloxApiError {
    let content_type = headers
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();

    let reason = if content_type.starts_with("application/json") {
        serde_json::from_str::<RobloxApiErrorResponse>(body)
            .ok()
            .and_then(RobloxApiErrorResponse::reason)
    } else if content_type.starts_with("text/html") {
        parse_html_error(body)
    } else {
        None
    };

    let reason = reason
        .or_else(|| truncate_body(body))
        .unwrap_or_else(|| "Roblox returned no diagnostic details.".to_owned());

    RobloxApiError::Roblox {
        status_code,
        request_method: request_method.to_owned(),
        request_url: request_url.to_owned(),
        reason,
    }
}

pub async fn handle_response_with_method(
    response: reqwest::Response,
    request_method: &str,
) -> RobloxApiResult<reqwest::Response> {
    // Check for redirects to the login page
    let url = response.url();
    if matches!(url.domain(), Some("www.roblox.com")) && url.path() == "/NewLogin" {
        return Err(RobloxApiError::Authorization);
    }

    if response.status().is_success() {
        Ok(response)
    } else {
        Err(get_roblox_api_error_from_response_with_method(response, request_method).await)
    }
}

pub async fn handle_with_method(
    result: Result<reqwest::Response, CsrfTokenRequestError>,
    request_method: &str,
) -> RobloxApiResult<reqwest::Response> {
    match result {
        Ok(response) => handle_response_with_method(response, request_method).await,
        Err(CsrfTokenRequestError::RequestError(error)) => Err(error.into()),
        Err(error) => Err(error.into()),
    }
}

pub async fn handle(
    result: Result<reqwest::Response, CsrfTokenRequestError>,
) -> RobloxApiResult<reqwest::Response> {
    handle_with_method(result, "UNKNOWN").await
}

pub async fn handle_response_as_json_with_method<T>(
    response: reqwest::Response,
    request_method: &str,
) -> RobloxApiResult<T>
where
    T: de::DeserializeOwned,
{
    let full = handle_response_with_method(response, request_method)
        .await?
        .text()
        .await?;
    trace!("Handle JSON: {}", full);
    serde_json::from_str::<T>(&full).map_err(|e| e.into())
}

pub async fn handle_as_json<T>(
    result: Result<reqwest::Response, CsrfTokenRequestError>,
) -> RobloxApiResult<T>
where
    T: de::DeserializeOwned,
{
    let res = handle(result).await?;
    let full = res.text().await?;
    trace!("Handle JSON: {}", full);
    serde_json::from_str::<T>(&full).map_err(|e| e.into())
}

pub async fn get_file_part(file_path: &Path) -> RobloxApiResult<Part> {
    debug!("stream read {:?}", &file_path);
    let file = File::open(file_path).await?;
    let reader = Body::wrap_stream(FramedRead::new(file, BytesCodec::new()));

    let file_name = file_path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| RobloxApiError::NoFileName(file_path.display().to_string()))?
        .to_owned();
    let mime = mime_guess::from_path(file_path).first_or_octet_stream();

    Ok(Part::stream(reader)
        .file_name(file_name)
        .mime_str(mime.as_ref())
        .unwrap())
}

pub async fn get_file_data(file_path: &Path) -> RobloxApiResult<(Vec<u8>, String, String)> {
    let data = tokio::fs::read(file_path).await?;
    let file_name = file_path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| RobloxApiError::NoFileName(file_path.display().to_string()))?
        .to_owned();
    let mime = mime_guess::from_path(file_path)
        .first_or_octet_stream()
        .to_string();

    Ok((data, file_name, mime))
}

#[cfg(test)]
mod tests {
    use super::parse_roblox_api_error;
    use reqwest::{header::HeaderMap, StatusCode};

    #[test]
    fn reports_open_cloud_error_fields() {
        let mut headers = HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );

        let error = parse_roblox_api_error(
            StatusCode::NOT_FOUND,
            "POST",
            "https://apis.roblox.com/game-passes/v1/universes/123/game-passes",
            &headers,
            r#"{"errorCode":"UniverseNotFound","errorMessage":"The universe was not found.","field":"universeId"}"#,
        );

        assert_eq!(
            error.to_string(),
            "Roblox API request failed: POST https://apis.roblox.com/game-passes/v1/universes/123/game-passes (404 Not Found): code: UniverseNotFound; The universe was not found.; field: universeId"
        );
    }

    #[test]
    fn reports_url_and_empty_response() {
        let error = parse_roblox_api_error(
            StatusCode::NOT_FOUND,
            "GET",
            "https://apis.roblox.com/example",
            &HeaderMap::new(),
            "",
        );

        assert_eq!(
            error.to_string(),
            "Roblox API request failed: GET https://apis.roblox.com/example (404 Not Found): Roblox returned no diagnostic details."
        );
    }
}
