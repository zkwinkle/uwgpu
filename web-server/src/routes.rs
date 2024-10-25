use axum::{
    routing::{get, post},
    Extension, Router,
};
use extractors::Layout;
use tower_http::services::ServeDir;

use crate::{
    app_config::AppConfig, components::benchmark_page::MicrobenchmarkKind::*,
};

pub mod extractors;
mod not_found;

mod home;
mod microbenchmark_page;
mod post_results;

use microbenchmark_page::microbenchmark_page;

/// Create the main `Router` for this app.
pub fn create_router(config: AppConfig) -> Router {
    let url = config.server_url;

    Router::new()
        .route("/", get(home::placeholder))
        .route(
            Matmul.path(),
            get(|l: Layout| async { microbenchmark_page(url, Matmul)(l) }),
        )
        .route(
            BufferSequential.path(),
            get(|l: Layout| async {
                microbenchmark_page(url, BufferSequential)(l)
            }),
        )
        .route("/results", post(post_results::post_results))
        .nest(
            "/public",
            Router::new().fallback_service(ServeDir::new(config.public_dir)),
        )
        .fallback(not_found::not_found)
        .layer(Extension(config.data_store))
        .layer(Extension(config.ua_parser))
}
