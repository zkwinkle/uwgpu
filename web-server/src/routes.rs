use axum::{routing::get, Router};
use extractors::Layout;
use tower_http::services::ServeDir;

use crate::{
    app_config::AppConfig, components::benchmark_page::MicrobenchmarkPage::*,
};

pub mod extractors;
mod not_found;

mod home;
mod microbenchmark_page;
use microbenchmark_page::microbenchmark_page;

/// Create the main `Router` for this app.
pub fn create_router(config: AppConfig) -> Router {
    Router::new()
        .route("/", get(home::placeholder))
        .route(
            Matmul.path(),
            get(|l: Layout| async { microbenchmark_page(Matmul)(l) }),
        )
        .route(
            BufferSequential.path(),
            get(|l: Layout| async { microbenchmark_page(BufferSequential)(l) }),
        )
        .nest(
            "/public",
            Router::new().fallback_service(ServeDir::new(config.public_dir)),
        )
        .fallback(not_found::not_found)
}
