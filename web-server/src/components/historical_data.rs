use super::benchmark_page::MicrobenchmarkKind;
use maud::{html, Markup, Render};

/// Component for viewing statistics of the historical data stored in the DB
/// Allows for filtering depending on hardware, OS, and "platform" (browser or
/// native drivers).
pub struct HistoricalData {
    pub microbenchmark: MicrobenchmarkKind,
}

impl Render for HistoricalData {
    fn render(&self) -> Markup {
        html! {

        label for="hardware-selector" { "Hardware Filter" }
        select id="hardware-selector" hx-get="/hardwares" hx-trigger="load" hx-swap="beforeend" {
            option { "--" }
        }

        }
    }
}
