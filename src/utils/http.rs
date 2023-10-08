use super::error::RustWireError;
use reqwest;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub async fn get(url: &str, proxy_url: Option<&str>) -> Result<String, RustWireError> {
    let mut client_builder = reqwest::Client::builder();

    if let Some(p_url) = proxy_url {
        let proxy =
            reqwest::Proxy::all(p_url).map_err(|err| RustWireError::HttpError(err.to_string()))?;
        client_builder = client_builder.proxy(proxy);
    }

    let client = client_builder
        .build()
        .map_err(|err| RustWireError::HttpError(err.to_string()))?;

    let response = client
        .get(url)
        .send()
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

pub struct ProxyManager {
    proxies: Vec<String>,
    current: Arc<AtomicUsize>,
}

impl ProxyManager {
    pub fn new(proxies: Vec<String>) -> Self {
        Self {
            proxies,
            current: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn get_next(&self) -> &str {
        let index = self.current.fetch_add(1, Ordering::Relaxed) % self.proxies.len();
        &self.proxies[index]
    }
}

pub async fn get_with_proxies(url: &str, manager: &ProxyManager) -> Result<String, RustWireError> {
    let mut attempts = manager.proxies.len();

    while attempts > 0 {
        let proxy_url = manager.get_next();
        let result = get(url, Some(proxy_url)).await;

        match result {
            Ok(body) => return Ok(body),
            Err(_) => attempts -= 1,
        }
    }

    Err(RustWireError::HttpError("All proxies failed.".to_string()))
}
