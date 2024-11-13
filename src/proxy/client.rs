use once_cell::sync::Lazy;
use std::time::Duration;
use crate::config::AppConfig;
use tracing::info;
use tracing::debug;

pub fn create_client(config: &AppConfig) -> reqwest::Client {
    info!("Creating HTTP client with optimized settings");
    debug!(
        "Client config: max_connections={}, keepalive={}s, nodelay={}",
        config.max_connections,
        config.tcp_keepalive_interval,
        config.tcp_nodelay
    );

    reqwest::Client::builder()
        .pool_max_idle_per_host(config.max_connections)
        .pool_idle_timeout(Duration::from_secs(30))
        .http2_prior_knowledge()
        .http2_keep_alive_interval(Duration::from_secs(config.tcp_keepalive_interval))
        .http2_keep_alive_timeout(Duration::from_secs(30))
        .http2_adaptive_window(true)
        .tcp_keepalive(Duration::from_secs(config.tcp_keepalive_interval))
        .tcp_nodelay(config.tcp_nodelay)
        .use_rustls_tls()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .gzip(true)
        .brotli(true)
        .build()
        .expect("Failed to create HTTP client")
}

pub static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    let config = AppConfig::new();
    create_client(&config)
});
