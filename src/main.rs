use axum::{
    middleware::from_fn_with_state,
    routing::{any, get},
    Router,
    extract::connect_info::ConnectInfo,
    Extension,
};
use hyper::{Body, Request};
use std::{
    net::SocketAddr,
    sync::Arc,
    time::Duration,
};
use tokio::signal;
use tower_http::{
    cors::{Any, CorsLayer},
};
use tracing::{debug, error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use colored::*;

mod config;
mod context;
mod error;
mod handlers;
mod providers;
mod proxy;
mod telemetry;

use crate::{
    config::{AppConfig, TelemetryConfig},
    context::RequestContext,
    handlers::{health_check, proxy_request},
    telemetry::{
        MetricsRegistry, 
        metrics_middleware, 
        ConsolePlugin,
        plugins::elasticsearch::ElasticsearchPlugin,
    },
};

#[tokio::main]
async fn main() {
    // Display startup animation
    let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    print!("\n    Starting Noveum AI Gateway ");
    for frame in frames.iter().cycle().take(15) {
        print!("\r    Starting Noveum AI Gateway {}  ", frame.bright_cyan());
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(120)).await;
    }
    println!("\r    Starting Noveum AI Gateway ✓  \n");
    
    // Display Noveum ASCII Art Logo
    println!("{}", r#"
    
     _   _                               
    | \ | | _____   _____ _   _ _ __ ___ 
    |  \| |/ _ \ \ / / _ \ | | | '_ ` _ \
    | |\  | (_) \ V /  __/ |_| | | | | | |
    |_| \_|\___/ \_/ \___|\__,_|_| |_| |_|
                                         
             AI Gateway v1.0.0
    ========================================
    "#.bright_cyan());
    
    println!("{}", "🚀 Starting Noveum AI Gateway...".bright_green());
    println!("{}", "📡 Your unified interface to multiple AI providers".bright_yellow());
    println!("{}\n", "========================================".bright_cyan());

    // Initialize tracing
    info!("Initializing tracing system");
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer().compact())
        .init();

    // Load configuration
    info!("Loading application configuration");
    let config = Arc::new(AppConfig::new());
    debug!(
        "Configuration loaded: port={}, host={}, worker_threads={}",
        config.port, config.host, config.worker_threads
    );

    // Optimize tokio runtime
    info!(
        "Configuring tokio runtime with {} worker threads",
        config.worker_threads
    );
    std::env::set_var("TOKIO_WORKER_THREADS", config.worker_threads.to_string());
    std::env::set_var("TOKIO_THREAD_STACK_SIZE", (2 * 1024 * 1024).to_string());

    // Setup CORS
    debug!("Setting up CORS layer with 1-hour max age");
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .max_age(Duration::from_secs(3600));

    let telemetry_config = TelemetryConfig::default();
    debug!(
        "Telemetry configuration: debug_mode={}",
        telemetry_config.debug_mode
    );
    let metrics_registry = Arc::new(MetricsRegistry::new(telemetry_config.debug_mode));

    // Register exporters based on configuration
    if telemetry_config.debug_mode {
        debug!("Registering Console plugin for metrics");
        metrics_registry
            .register_exporter(Box::new(ConsolePlugin::new()))
            .await;
    }

    if telemetry_config.elasticsearch_enabled {
        debug!("Registering Elasticsearch exporter");
        
        let elasticsearch_url = std::env::var("ELASTICSEARCH_URL")
            .unwrap_or_else(|_| "http://localhost:9200".to_string());
            
        let elasticsearch_username = std::env::var("ELASTICSEARCH_USERNAME").ok();
        let elasticsearch_password = std::env::var("ELASTICSEARCH_PASSWORD").ok();
        
        let elasticsearch_index = std::env::var("ELASTICSEARCH_INDEX")
            .unwrap_or_else(|_| "ai-gateway-metrics".to_string());
        
        match ElasticsearchPlugin::new(
            elasticsearch_url, 
            elasticsearch_username, 
            elasticsearch_password, 
            elasticsearch_index
        ) {
            Ok(plugin) => {
                metrics_registry.register_exporter(Box::new(plugin)).await;
                info!("Elasticsearch exporter registered successfully");
            },
            Err(e) => {
                error!("Failed to initialize Elasticsearch exporter: {}", e);
            }
        }
    }

    debug!("Registering middleware for metrics collection and telemetry");
    
    // Register request handlers and middleware
    info!("Registering request handlers and API routes");
    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/v1/*path", any(handlers::proxy_request))
        .layer(from_fn_with_state(
            metrics_registry.clone(),
            metrics_middleware,
        ))
        .with_state(config.clone())
        .layer(cors);

    // Start server with optimized TCP settings
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Setting up TCP listener with non-blocking mode");
    let tcp_listener = std::net::TcpListener::bind(addr).expect("Failed to bind address");
    tcp_listener
        .set_nonblocking(true)
        .expect("Failed to set non-blocking");

    debug!("Converting to tokio TCP listener");
    let listener = tokio::net::TcpListener::from_std(tcp_listener)
        .expect("Failed to create Tokio TCP listener");

    // Print server started ASCII art
    println!("{}", r#"
    ╔══════════════════════════════════════════════╗
    ║                                              ║
    ║  🌟 Noveum AI Gateway is now ONLINE! 🌟      ║
    ║                                              ║
    ╚══════════════════════════════════════════════╝"#.bright_green());

    info!(
        "AI Gateway listening on {}:{} with {} worker threads",
        config.host, config.port, config.worker_threads
    );

    println!("{}", format!("    🔗 Listening at http://{}:{}", config.host, config.port).bright_cyan());
    println!("{}", "    🔄 Press Ctrl+C to shutdown gracefully".bright_yellow());
    println!();

    debug!("Starting server with graceful shutdown");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap_or_else(|e| {
        error!("Server error: {}", e);
        std::process::exit(1);
    });
}

async fn shutdown_signal() {
    info!("Registering shutdown signal handler");
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler")
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            println!("{}", r#"
    ╔══════════════════════════════════════════════╗
    ║                                              ║
    ║  🛑 Noveum AI Gateway shutting down... 🛑    ║
    ║                                              ║
    ╚══════════════════════════════════════════════╝"#.bright_yellow());
            info!("SIGINT (Ctrl+C) received, starting graceful shutdown");
        },
        _ = terminate => {
            println!("{}", r#"
    ╔══════════════════════════════════════════════╗
    ║                                              ║
    ║  🛑 Noveum AI Gateway shutting down... 🛑    ║
    ║                                              ║
    ╚══════════════════════════════════════════════╝"#.bright_yellow());
            info!("SIGTERM received, starting graceful shutdown");
        },
    }
}