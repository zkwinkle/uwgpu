use maud::{html, Markup};

use super::extractors::Layout;

#[cfg_attr(feature = "debug", axum::debug_handler)]
pub async fn placeholder(layout: Layout) -> Markup {
    layout.render( html! {
        p { "TODO: Esta página es temporal mientras experimento con wgpu, eventualmente voy a hacer el UI real" }
    })
}
