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
    failures: Arc<AtomicUsize>,
    current: Arc<AtomicUsize>,
}

impl ProxyManager {
    pub fn new(proxies: Vec<String>) -> Self {
        Self {
            proxies,
            failures: Arc::new(AtomicUsize::new(0)),
            current: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn get_next(&self) -> Option<&str> {
        if self.failures.load(Ordering::Relaxed) > 3 {
            return None;
        }

        let index = self.current.fetch_add(1, Ordering::Relaxed) % self.proxies.len();
        Some(&self.proxies[index])
    }

    pub fn mark_failure(&self) {
        self.failures.fetch_add(1, Ordering::Relaxed);
    }
}

pub async fn get_with_proxies(
    url: &str,
    manager: &ProxyManager,
    max_attempts: usize,
) -> Result<String, RustWireError> {
    let mut attempts = 0;

    while attempts < max_attempts {
        if let Some(proxy_url) = manager.get_next() {
            match get(url, Some(proxy_url)).await {
                Ok(body) => return Ok(body),
                Err(_) => {
                    manager.mark_failure();
                    attempts += 1;
                }
            }
        } else {
            break;
        }
    }

    Err(RustWireError::HttpError("All proxies failed.".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[tokio::test]
    async fn test_get_without_proxy() {
        let mut server = Server::new();
        let _mock = server
            .mock("GET", "/test")
            .with_status(200)
            .with_body("test body")
            .create();

        let url = format!("{}/test", server.url());
        let result = get(&url, None).await;
        assert_eq!(result.unwrap(), "test body");
    }

    #[tokio::test]
    async fn test_get_with_http_error() {
        let mut server = Server::new();
        let _mock = server.mock("GET", "/error").with_status(404).create();

        let url = format!("{}/error", server.url());
        let result = get(&url, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_proxy_manager_new() {
        let proxies = vec![
            "http://localhost:8000".to_string(),
            "http://localhost:8001".to_string(),
        ];
        let manager = ProxyManager::new(proxies.clone());
        assert_eq!(manager.proxies, proxies);
        assert_eq!(manager.failures.load(Ordering::Relaxed), 0);
        assert_eq!(manager.current.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn test_proxy_manager_rotation() {
        let manager = ProxyManager::new(vec![
            "http://localhost:8000".to_string(),
            "http://localhost:8001".to_string(),
        ]);
        assert_eq!(manager.get_next().unwrap(), "http://localhost:8000");
        assert_eq!(manager.get_next().unwrap(), "http://localhost:8001");
        assert_eq!(manager.get_next().unwrap(), "http://localhost:8000"); // It should rotate back
    }

    #[tokio::test]
    async fn test_proxy_manager_mark_failure() {
        let manager = ProxyManager::new(vec!["http://localhost:8000".to_string()]);
        manager.mark_failure();
        assert_eq!(manager.failures.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn test_proxy_manager_failure_limit() {
        let manager = ProxyManager::new(vec!["http://localhost:8000".to_string()]);
        for _ in 0..4 {
            manager.mark_failure();
        }
        assert_eq!(manager.get_next(), None);
    }

    #[tokio::test]
    async fn test_get_with_proxies_all_fail() {
        let mut server = Server::new();
        let _mock = server
            .mock("GET", "/test")
            .with_status(200)
            .with_body("test body")
            .create();

        let manager = ProxyManager::new(vec!["http://invalid-proxy".to_string()]); // Invalid proxy for demonstration
        let url = format!("{}/test", server.url());
        let result = get_with_proxies(&url, &manager, 3).await;
        assert!(result.is_err());
    }
}
