use super::error::RustWireError;
use reqwest;

pub async fn get(url: &str) -> Result<String, RustWireError> {
    let response = reqwest::get(url)
        .await
        .map_err(|err| RustWireError::HttpError(err.to_string()))?;
    if !response.status().is_success() {
        return Err(RustWireError::HttpStatusCodeError(format!(
            "HTTP error: {}",
            response.status()
        )));
    }
    let body = response
        .text()
        .await
        .map_err(|err| RustWireError::HttpError(err.to_string()))?;
    Ok(body)
}
