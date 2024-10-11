use axum::{routing::get, Router};
use tower_http::services::ServeDir;

use crate::app_config::AppConfig;

pub mod extractors;
mod not_found;

mod home;

/// Create the main `Router` for this app.
pub fn create_router(config: AppConfig) -> Router {
    Router::new()
        .route("/", get(home::placeholder))
        .nest(
            "/public",
            Router::new().fallback_service(ServeDir::new(config.public_dir)),
        )
        .fallback(not_found::not_found)
}
