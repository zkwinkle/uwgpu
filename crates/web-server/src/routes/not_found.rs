use axum::{extract::OriginalUri, http::StatusCode};
use maud::{html, Markup};

use super::extractors::Layout;

#[cfg_attr(feature = "debug", axum::debug_handler)]
pub async fn not_found(
    layout: Layout,
    OriginalUri(original_uri): OriginalUri,
) -> (StatusCode, Markup) {
    (
        StatusCode::NOT_FOUND,
        layout.render(html! {
            h1 { "404 Page Not Found" }
            p {"Original uri:" (original_uri.to_string())}
        }),
    )
}
