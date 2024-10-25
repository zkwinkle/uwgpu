use maud::{html, Markup};

use crate::components::benchmark_page::{
    MicrobenchmarkKind, MicrobenchmarkPage,
};

use super::extractors::Layout;

pub fn microbenchmark_page(
    server_url: &'static str,
    microbenchmark: MicrobenchmarkKind,
) -> impl Fn(Layout) -> Markup {
    let page = MicrobenchmarkPage {
        microbenchmark,
        server_url,
    };

    move |layout: Layout| layout.render(html! {(page)})
}
