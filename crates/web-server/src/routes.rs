use axum::{
    handler::HandlerWithoutStateExt,
    routing::{get, post},
    Extension, Router,
};
use extractors::Layout;
use tower_http::services::ServeDir;

use crate::api_types::MicrobenchmarkKind::*;
use crate::app_config::AppConfig;

mod extractors;
mod not_found;

mod hardware_options;
mod historic_data_table;
mod home;
mod microbenchmark_page;
mod os_options;
mod post_results;

use microbenchmark_page::microbenchmark_page;

/// Create the main `Router` for this app.
pub fn create_router(config: AppConfig) -> Router {
    let url = config.server_url;

    Router::new()
        .route("/", get(home::home))
        .route(
            Matmul.path(),
            get(|l: Layout| async { microbenchmark_page(Matmul)(l, url) }),
        )
        .route(
            Reduction.path(),
            get(|l: Layout| async { microbenchmark_page(Reduction)(l, url) }),
        )
        .route(
            BufferToBuffer.path(),
            get(|l: Layout| async {
                microbenchmark_page(BufferToBuffer)(l, url)
            }),
        )
        .route("/results", post(post_results::post_results))
        .route("/hardwares", get(hardware_options::hardware_options))
        .route("/operating_systems", get(os_options::os_options))
        .route(
            "/statistic_table",
            get(historic_data_table::historica_data_table),
        )
        .fallback_service(
            ServeDir::new(config.public_dir)
                .fallback(not_found::not_found.into_service()),
        )
        .layer(Extension(config.server_url))
        .layer(Extension(config.data_store))
        .layer(Extension(config.ua_parser))
}
