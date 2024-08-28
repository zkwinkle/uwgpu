use axum::{routing::get, Router};
use tower_http::services::ServeDir;

use crate::app_config::AppConfig;

pub mod extractors;
mod not_found;

// TODO: Eventually this will get deleted when the real UI is created
mod placeholder;

/// Create the main `Router` for this app.
pub fn create_router(config: AppConfig) -> Router {
    Router::new()
        // TODO: This is a placeholder while I mess around with wgpu,
        // eventually it'll get replaced with the real route.
        .route("/", get(placeholder::placeholder))
        .nest(
            "/public",
            Router::new().fallback_service(ServeDir::new(config.public_dir)),
        )
        .fallback(not_found::not_found)
}
