use crate::data_store::platform::Platform;

use super::benchmark_page::MicrobenchmarkKind;
use maud::{html, Markup, Render};

/// Component for viewing statistics of the historical data stored in the DB
/// Allows for filtering depending on hardware, OS, and "platform" (browser or
/// native drivers).
pub struct HistoricalData {
    pub microbenchmark: MicrobenchmarkKind,
}

/// TODO: Include other browsers and native platforms
const PLATFORM_OPTIONS: &'static [Platform] =
    &[Platform::Chromium, Platform::Firefox];

impl Render for HistoricalData {
    fn render(&self) -> Markup {
        html! {

        div class="data-filter" {
            label for="hardware-selector" { "Hardware Filter" }
            select id="hardware-selector" hx-get="/hardwares" hx-trigger="load" hx-swap="beforeend" {
                option { "--" }
            }
        }

        div class="data-filter" {
            label for="os-selector" { "OS Filter" }
            select id="os-selector" hx-get="/operating_systems" hx-trigger="load" hx-swap="beforeend" {
                option { "--" }
            }
        }

        div class="data-filter" {
            label for="platform-selector" { "Platform Filter" }
            select id="platform-selector" {
                option { "--" }
                @for platform in PLATFORM_OPTIONS {
                        option value=(serde_json::to_string(platform).unwrap())
                            { (platform_label(platform)) }
                }
            }
        }

        }
    }
}

fn platform_label(platform: &Platform) -> &'static str {
    match platform {
        Platform::Chromium => "Chromium",
        Platform::Firefox => "Firefox",

        // TODO: Implement missing platform labels when they get added to
        // `PLATFORM_OPTIONS`
        Platform::OtherBrowser => todo!(),
        Platform::NativeVulkan => todo!(),
        Platform::NativeMetal => todo!(),
        Platform::NativeDx12 => todo!(),
    }
}
