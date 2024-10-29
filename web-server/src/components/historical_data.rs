use maud::{html, Markup, Render};

use crate::api_types::{MicrobenchmarkKind, Platform};

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
        form hx-get="/statistic_table" hx-trigger="load, change" hx-target="#stats-table" {
            div hx-target="unset" hx-include="unset" {
            div class="data-filter" {
                label for="hardware-selector" { "Hardware Filter" }
                // TODO: Include benchmark kind with hx-vals
                select name="hardware" id="hardware-selector" hx-get="/hardwares" hx-trigger="load" hx-swap="beforeend" {
                    option value="" { "--" }
                }
            }

            div class="data-filter" {
                label for="os-selector" { "OS Filter" }
                select name="operating_system" id="os-selector" hx-get="/operating_systems" hx-trigger="load" hx-swap="beforeend" {
                    option value="" { "--" }
                }
            }

            div class="data-filter" {
                label for="platform-selector" { "Platform Filter" }
                select name="platform" id="platform-selector" {
                    option value="" { "--" }
                    @for platform in PLATFORM_OPTIONS {
                            option value=(serde_json::to_string(platform).unwrap())
                                { (platform_label(platform)) }
                    }
                }
            }
            }
        }

        span id="stats-table" {}

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
