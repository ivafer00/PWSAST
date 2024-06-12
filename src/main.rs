mod config;
mod controllers;
mod prelude;
mod error;
mod cmd;

use crate::config::config::load_config;
use std::path::PathBuf;
use hyper::Method;
use std::time::Duration;
use axum_server::tls_rustls::RustlsConfig;
use log::info;
use tokio::signal;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use crate::config::config::Configuration;
use crate::controllers::html::{html};

#[derive(Clone)]
struct AppState {
    pub config : Configuration,
}

#[tokio::main]
async fn main() {

    log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    info!("Reading configuration file...");
    let args = cmd::parse();
    let config = load_config(PathBuf::from(args.config_path)).expect("Couldn't load configuration. Aborting...");

    let shared_state = AppState { config };

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    let serve_dir = ServeDir::new("assets").not_found_service(ServeFile::new("assets/error.html"));

    let routes = html::router()
        .nest_service("/assets", serve_dir.clone())
        .with_state(shared_state.clone())
        .layer(cors);

    let bind_address = format!("{}:{}", shared_state.config.app.host,
                               shared_state.config.app.port);

    let handle = axum_server::Handle::new();
    let shutdown_future = shutdown_signal(handle.clone());

    info!("🚀 Server starting...");

    info!("Server configured to listen connections on {}:{}...",
            shared_state.config.app.host,
            shared_state.config.app.port
    );

    if shared_state.config.tls.key_path.len() != 0 && shared_state.config.tls.cert_path.len() != 0 {
        info!("Configuring server to use HTTPS...");
        let listener = bind_address.parse().expect("HOST or PORT not specified");

        let config = RustlsConfig::from_pem_file(
            shared_state.config.tls.cert_path,
            shared_state.config.tls.key_path
        )
            .await
            .unwrap();

        axum_server::bind_rustls(listener, config)
            .handle(handle)
            .serve(routes.into_make_service())
            .await
            .unwrap();
    } else {
        info!("Defaulting to HTTP...");
        let listener = tokio::net::TcpListener::bind(bind_address)
            .await
            .unwrap();

        axum::serve(listener, routes)
            .with_graceful_shutdown(shutdown_future)
            .await
            .unwrap();
    }
    info!("Server stopped.");

}

async fn shutdown_signal(handle: axum_server::Handle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
        let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Received termination signal shutting down");
    handle.graceful_shutdown(Some(Duration::from_secs(10)));
}