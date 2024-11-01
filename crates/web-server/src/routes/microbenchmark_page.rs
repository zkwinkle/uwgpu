use maud::{html, Markup};

use crate::{
    api_types::MicrobenchmarkKind,
    components::benchmark_page::MicrobenchmarkPage,
};

use super::extractors::Layout;

pub fn microbenchmark_page(
    microbenchmark: MicrobenchmarkKind,
) -> impl Fn(Layout) -> Markup {
    let page = MicrobenchmarkPage { microbenchmark };

    move |layout: Layout| layout.render(html! {(page)})
}
