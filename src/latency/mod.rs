use crate::utils::error::RustWireError;
use crate::utils::http;

use futures::future::join_all;
use std::time::Instant;

pub async fn test(url: &str, num_requests: usize) -> Result<(u128, usize), RustWireError> {
    let mut futures = Vec::with_capacity(num_requests);

    for _ in 0..num_requests {
        let url_clone = url.to_string();
        let future = tokio::spawn(async move {
            let start_time = Instant::now();
            match http::get(&url_clone).await {
                Ok(_val) => {
                    println!("Success");
                    Ok(start_time.elapsed().as_millis())
                }
                Err(err) => {
                    println!("Error: {}", err);
                    Err(err.clone())
                }
            }
        });

        futures.push(future);
    }

    let futures_results: Vec<Result<Result<u128, RustWireError>, tokio::task::JoinError>> =
        join_all(futures).await;
    let mut durations = Vec::new();
    let mut errors_count = 0;

    for result in &futures_results {
        match result {
            Ok(inner_result) => match inner_result {
                Ok(duration) => durations.push(*duration),
                Err(_) => {
                    errors_count += 1;
                }
            },
            Err(_) => {
                errors_count += 1;
            }
        }
    }

    let error_rate = errors_count as f64 / num_requests as f64;
    const ERROR_THRESHOLD: f64 = 0.9;

    if error_rate > ERROR_THRESHOLD {
        return Err(RustWireError::HttpError(format!(
            "Error rate of {:.2}% exceeds the threshold",
            error_rate * 100.0
        )));
    }

    let total_duration: u128 = durations.iter().sum();
    println!("Total duration: {}", total_duration);
    println!("Errors count: {}", errors_count);

    Ok((total_duration / num_requests as u128, errors_count))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_latency_for_url() {
        let url = "https://24pullrequests.com/users.json?page=2";
        let result = test(url, 1).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_url() {
        let url = "https://this-is-clearly-an-invalid-url.xyz";
        let result = test(url, 1).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_multiple_requests() {
        let url = "https://24pullrequests.com/users.json?page=2";
        let result = test(url, 2).await;
        assert!(result.is_ok());

        let (avg_latency, errors) = result.unwrap();
        assert!(avg_latency < 500000);
        assert_eq!(errors, 0);
    }
}
