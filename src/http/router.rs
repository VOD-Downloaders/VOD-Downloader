use std::sync::Arc;

use serde_json::json;
use thiserror::Error;
use url::Url;
use axum::{routing, response};
use axum::http::{header, HeaderValue, StatusCode};
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;

use super::api;
use crate::env;
use crate::config;
use crate::request;

/////////////////////////////////////////////////////
// RouteError
/////////////////////////////////////////////////////
#[derive(Debug, Error)]
pub enum RouteError {
    #[error("Failed to bind to port {port} with error: {error}.")]
    FailedToBind { port: u16, error: std::io::Error },
}

/////////////////////////////////////////////////////
// Router
/////////////////////////////////////////////////////
pub struct Router {
    router: axum::Router,
    listener: tokio::net::TcpListener,
}

impl Router {
    const WEB_SRC_DIRECTORY: &str = "/app/web/";

    pub async fn new(environment: env::EnvOptions, state: config::State) -> Result<Self, RouteError> {
        let address = "0.0.0.0:".to_string() + environment.webui_port.to_string().as_str();
        let listener = tokio::net::TcpListener::bind(address.as_str())
            .await
            .map_err(|error| RouteError::FailedToBind {
                port: environment.webui_port,
                error: error,
            })?;

        info!("HTTP server listening on {}.", address.as_str());

        let router = Self::init_router(environment, state);

        Ok(Self {
            router: router,
            listener: listener,
        })
    }

    pub async fn serve(self) {
        // NOTE: Never returns an error
        axum::serve(self.listener, self.router)
            .with_graceful_shutdown(Self::shutdown_signal())
            .await
            .unwrap();
    }

    async fn shutdown_signal() {
        let ctrl_c = async {
            tokio::signal::ctrl_c().await.expect("Failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("Failed to install terminate signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }
    }

    fn init_router(environment: env::EnvOptions, state: config::State) -> axum::Router {
        let web_source_service = ServeDir::new(Self::WEB_SRC_DIRECTORY).append_index_html_on_directories(true);
        let no_cache = SetResponseHeaderLayer::overriding(header::CACHE_CONTROL, HeaderValue::from_static("no-cache")); // Force revalidation

        let router = axum::Router::new()
            // API calls
            .route("/health", routing::get(Self::health))
            .route("/api/indexers", routing::get(api::get_indexers))
            .route("/api/indexers/create", routing::post(api::post_create_indexer))
            .route("/api/indexers/delete", routing::post(api::post_delete_indexer))
            .route("/api/indexers/specifications", routing::get(api::get_indexer_specifications))
            .route("/api/indexers/specifications/refresh", routing::post(api::post_refresh_indexer_specifications))
            .route("/api/streams", routing::post(api::post_streams))
            .route("/api/download", routing::post(api::post_download))

            // HTML, CSS, JS
            .fallback_service(web_source_service)
            .layer(no_cache)

            // State
            .with_state(Arc::new(api::AppState::new(environment, state)));

        trace!("Created HTTP router.");

        router
    }

    async fn health() -> (StatusCode, response::Json<serde_json::Value>) {
        trace!("Got /health request.");

        let response = {
            if Self::check_internet().await.is_err() {
                response::Json(json!({ "health": "no internet" }))
            } else {
                response::Json(json!({ "health": "healthy" }))
            }
        };

        trace!("Responding with: {}", response.to_string());
        (StatusCode::OK, response)
    }

    async fn check_internet() -> Result<(), request::RequestError> {
        const CONNECTIVITY_CHECK_URL: &str = "https://www.google.com";

        let url = Url::parse(CONNECTIVITY_CHECK_URL).unwrap();

        let specification = request::RequesterSpecification {
            connect_timeout: 5,
            max_timeout: 5,
            ..request::RequesterSpecification::default()
        };
        let requester = request::Requester::get_native(specification)?;

        requester.get_bytes(&url, None).await?;
        Ok(())
    }
}
