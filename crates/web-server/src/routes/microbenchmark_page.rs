use maud::{html, Markup};

use crate::{
    api_types::MicrobenchmarkKind,
    components::benchmark_page::MicrobenchmarkPage,
};

use super::extractors::Layout;

pub fn microbenchmark_page(
    microbenchmark: MicrobenchmarkKind,
) -> impl Fn(Layout, &'static str) -> Markup {
    move |layout: Layout, server_url: &'static str| {
        layout.render(
            html! {( MicrobenchmarkPage { microbenchmark, server_url } )},
        )
    }
}
