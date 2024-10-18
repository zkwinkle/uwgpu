use maud::{html, Markup};

use crate::components::benchmark_page::MicrobenchmarkPage;

use super::extractors::Layout;

pub fn microbenchmark_page(
    microbenchmark: MicrobenchmarkPage,
) -> impl Fn(Layout) -> Markup {
    move |layout: Layout| layout.render(html! {(microbenchmark)})
}
