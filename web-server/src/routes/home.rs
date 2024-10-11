use maud::{html, Markup, PreEscaped};

use super::extractors::Layout;

#[cfg_attr(feature = "debug", axum::debug_handler)]
pub async fn placeholder(layout: Layout) -> Markup {
    layout.render( html! {
        p { "TODO: Esta página es temporal mientras experimento con wgpu, eventualmente voy a hacer el UI real" }
        div id="wasm-canvas" {
            style {
                "canvas { background-color: black }"
            }
            script type="module" {
                (PreEscaped(r#"
                  import init from "./public/pkg/uwgpu.js";

                  init().then(() => {
                      console.log("WASM Loaded and executing");
                  });
                "#))
                }
        }
    })
}

// TODO: WASM Loaded no se imprime idk xq
