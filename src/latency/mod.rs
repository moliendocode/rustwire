use crate::utils::error::RustWireError;
use crate::utils::http;
use crate::utils::http::ProxyManager;

use futures::future::join_all;
use std::sync::Arc;
use std::time::Instant;

pub async fn test(
    url: &str,
    num_requests: usize,
    error_threshold: Option<f64>,
    proxy_manager: Option<Arc<ProxyManager>>,
) -> Result<(u128, usize), RustWireError> {
    let threshold: f64 = error_threshold.unwrap_or(0.9);
    let futures = (0..num_requests)
        .map(|_| {
            let url_clone = url.to_string();
            let manager_clone = proxy_manager.clone();
            tokio::spawn(async move {
                let start_time = Instant::now();

                let result = if let Some(manager) = manager_clone {
                    http::get_with_proxies(&url_clone, &*manager, 3).await
                } else {
                    http::get(&url_clone, None).await
                };

                match result {
                    Ok(_val) => {
                        println!("Success");
                        Ok(start_time.elapsed().as_millis())
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                        Err(err)
                    }
                }
            })
        })
        .collect::<Vec<_>>();

    let futures_results: Vec<Result<Result<u128, RustWireError>, tokio::task::JoinError>> =
        join_all(futures).await;

    let mut total_duration: u128 = 0;
    let mut errors_count = 0;

    for result in futures_results {
        match result {
            Ok(Ok(duration)) => total_duration += duration,
            Ok(Err(_)) | Err(_) => errors_count += 1,
        }
    }

    let error_rate = errors_count as f64 / num_requests as f64;

    if error_rate > threshold {
        return Err(RustWireError::HttpError(format!(
            "Error rate of {:.2}% exceeds the threshold",
            error_rate * 100.0
        )));
    }

    println!("Total duration: {}ms", total_duration);
    println!("Errors count: {}", errors_count);

    Ok((total_duration / num_requests as u128, errors_count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[tokio::test]
    async fn test_latency_for_url() {
        let mut server = Server::new();
        let _mock = server
            .mock("GET", "/users.json?page=2")
            .with_status(200)
            .with_body("mock body")
            .create();

        let url = &format!("{}/users.json?page=2", server.url());
        let result = test(url, 1, None, None).await;

        assert!(result.is_ok());
        _mock.assert();
    }

    #[tokio::test]
    async fn test_invalid_url() {
        let mut server = Server::new();
        let _mock = server
            .mock("GET", "/this-is-clearly-an-invalid-url.xyz")
            .with_status(404)
            .create();

        let url = &format!("{}/this-is-clearly-an-invalid-url.xyz", server.url());
        let result = test(url, 1, Some(0.1), None).await;

        assert!(result.is_err());
        _mock.assert();
    }

    #[tokio::test]
    async fn test_multiple_requests() {
        let mut server = Server::new();
        let _mock = server
            .mock("GET", "/users.json?page=2")
            .with_status(200)
            .with_body("mock body")
            .expect(2)
            .create();

        let url = &format!("{}/users.json?page=2", server.url());
        let result = test(url, 2, None, None).await;
        assert!(result.is_ok());

        let (avg_latency, errors) = result.unwrap();
        assert!(avg_latency < 500);
        assert_eq!(errors, 0);

        _mock.assert();
    }

    #[tokio::test]
    async fn test_error_threshold() {
        let mut server = Server::new();

        for _ in 0..3 {
            server
                .mock("GET", "/threshold-test")
                .with_status(200)
                .with_body("mock success body")
                .create();
        }

        for _ in 0..2 {
            server
                .mock("GET", "/threshold-test")
                .with_status(404)
                .with_body("mock error body")
                .create();
        }

        let url = &format!("{}/threshold-test", server.url());

        let result = test(url, 5, Some(0.3), None).await;

        assert!(result.is_err());
    }
}
