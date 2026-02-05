//! Asciidork REST API Server
//!
//! This crate provides a REST API for converting Asciidoc documents to HTML
//! using the asciidork parser and backends.

pub mod config;
pub mod error;
pub mod handlers;
pub mod models;
pub mod state;

use axum::{
    routing::{get, post},
    Router,
};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tower_http::{
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

use config::Config;
use state::AppState;

/// Build the application router with all endpoints configured.
#[allow(deprecated)]
pub fn app(config: Config) -> Router {
    let state = AppState {
        config: Arc::new(config.clone()),
    };

    // Build CORS layer
    let cors = if config.cors_origins.iter().any(|o| o == "*") {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        CorsLayer::new()
            .allow_origin(
                config
                    .cors_origins
                    .iter()
                    .filter_map(|o| o.parse().ok())
                    .collect::<Vec<_>>(),
            )
            .allow_methods(Any)
            .allow_headers(Any)
    };

    // Build timeout layer
    let timeout = TimeoutLayer::new(Duration::from_secs(config.request_timeout_secs));

    Router::new()
        .route("/api/v1/convert", post(handlers::convert::convert))
        .route(
            "/api/v1/convert/multipart",
            post(handlers::convert::convert_multipart),
        )
        .route("/api/v1/health", get(handlers::health::health))
        .route("/api/v1/info", get(handlers::info::info))
        .layer(TraceLayer::new_for_http())
        .layer(timeout)
        .layer(RequestBodyLimitLayer::new(config.max_content_size))
        .layer(cors)
        .with_state(state)
}

/// Start the server with the given configuration.
///
/// The server will gracefully shutdown on SIGINT (Ctrl+C) or SIGTERM signals.
pub async fn serve(config: Config) -> Result<(), std::io::Error> {
    let addr = SocketAddr::from((
        config
            .host
            .parse::<std::net::IpAddr>()
            .expect("Invalid host address"),
        config.port,
    ));

    tracing::info!("Starting asciidork server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app(config))
        .with_graceful_shutdown(shutdown_signal())
        .await
}

/// Wait for shutdown signals (SIGINT or SIGTERM).
///
/// This function returns when either signal is received, allowing
/// the server to gracefully shutdown.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received SIGINT, shutting down gracefully...");
        }
        _ = terminate => {
            tracing::info!("Received SIGTERM, shutting down gracefully...");
        }
    }
}
