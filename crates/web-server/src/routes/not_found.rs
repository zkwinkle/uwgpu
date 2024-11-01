use axum::http::StatusCode;
use maud::{html, Markup};

use super::extractors::Layout;

#[cfg_attr(feature = "debug", axum::debug_handler)]
pub async fn not_found(layout: Layout) -> (StatusCode, Markup) {
    (
        StatusCode::NOT_FOUND,
        layout.render(html! {
            h1 { "404 Page Not Found" }
        }),
    )
}
