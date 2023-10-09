# RustWire: API Testing Suite

A robust and efficient suite of tools written in Rust for testing APIs. Facilitates load testing, latency measurements, and response validations on your endpoints.

![RustWire Logo](https://i.ibb.co/f2vLGcr/rustwire.png)

## Features

- ✅ **Latency Testing**: Measure the latency of your endpoints and obtain detailed statistics.
- 🚧 **Load Testing**: Simulate traffic to gauge the capacity of your API. (TODO)
- 🚧 **Connection Time**: Evaluate the time taken to establish a connection with your endpoints. (TODO)
- ✅ **Proxy Integration**: Run tests across different proxies to simulate varied user scenarios.

## Installation

If you wish to use the latest version directly from the repository, add the following line to the dependencies in your `Cargo.toml`:

```toml
[dependencies]
rustwire = { git = "https://github.com/moliendocode/rustwire.git" }
```

## Basic Usage

Here's a quick guide on how to use the latency test:

### Latency Testing

This allows you to measure the latency of a specific endpoint over multiple requests.

```rust
use rustwire::{test_latency, ProxyManager};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let url = "https://example.com/api/data";
    let num_requests = 5;
    
    // Using direct call without proxy
    let result = test_latency(url, num_requests, None, None).await;
    
    match result {
        Ok((average_latency, error_count)) => {
            println!("Average Latency: {}ms", average_latency);
            println!("Errors: {}", error_count);
        }
        Err(err) => {
            eprintln!("Error: {:?}", err);
        }
    }

    // Using with a proxy (sample usage)
    let proxies = vec!["http://proxy1.com".to_string(), "http://proxy2.com".to_string()];
    let proxy_manager = Some(Arc::new(ProxyManager::new(proxies)));
    let result_with_proxy = test_latency(url, num_requests, None, proxy_manager).await;

    // ... handle the result as before
}
```

## Detailed Documentation

For a more detailed breakdown of each module and advanced features, please visit the [project's Wiki](https://github.com/moliendocode/rustwire/wiki).

## Contribution

We'd love for you to contribute to enhancing `RustWire`. Please read our [contribution guidelines](url-to-your-contribution-guidelines) before submitting a pull request.

## License

This library is licensed under the [MIT License](https://github.com/moliendocode/rustwire/blob/main/LICENSE).
