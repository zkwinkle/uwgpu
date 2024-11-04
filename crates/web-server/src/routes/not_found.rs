use axum::{
    extract::{OriginalUri, Path},
    http::StatusCode,
};
use maud::{html, Markup};

use super::extractors::Layout;

#[cfg_attr(feature = "debug", axum::debug_handler)]
pub async fn not_found(
    layout: Layout,
    OriginalUri(original_uri): OriginalUri,
    Path(path): Path<String>,
) -> (StatusCode, Markup) {
    (
        StatusCode::NOT_FOUND,
        layout.render(html! {
            h1 { "404 Page Not Found" }
            p {"Original uri:" (original_uri.to_string())}
            p {"Path:" (path.to_string())}
        }),
    )
}
