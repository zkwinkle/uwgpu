use maud::{html, Markup, Render};

use crate::api_types::{MicrobenchmarkKind, Platform};

/// Component for viewing statistics of the historical data stored in the DB
/// Allows for filtering depending on hardware, OS, and "platform" (browser or
/// native drivers).
pub struct HistoricalData {
    pub microbenchmark: MicrobenchmarkKind,
    pub server_url: &'static str,
}

/// TODO: Include other browsers and native platforms
const PLATFORM_OPTIONS: &'static [Platform] =
    &[Platform::Chromium, Platform::Firefox];

impl Render for HistoricalData {
    fn render(&self) -> Markup {
        let link = |path: &str| format!("{}{}", self.server_url, path);

        html! {
        h2 { "Historical Data" }
        form
            id="stats-filters"
            hx-get=(link("/statistic_table"))
            hx-trigger="load, change, click from:#refresh-button"
            hx-target="#stats-table"
            hx-vals=(format!(
                r#"{{"microbenchmark": {}}}"#,
                serde_json::to_string(&self.microbenchmark).unwrap()
                ))
            {
            div hx-target="unset" {
            div class="data-filter" {
                label for="hardware-selector" { "Hardware Filter" }
                select name="hardware" id="hardware-selector" hx-get=(link("/hardwares")) hx-trigger="load" hx-swap="beforeend" {
                    option value="" { "--" }
                }
            }

            div class="data-filter" {
                label for="os-selector" { "OS Filter" }
                select name="operating_system" id="os-selector" hx-get=(link("/operating_systems")) hx-trigger="load" hx-swap="beforeend" {
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

        button id="refresh-button" {
            svg xmlns="http://www.w3.org/2000/svg" x-bind:width="size" x-bind:height="size" viewBox="0 0 24 24" fill="none" stroke="currentColor" x-bind:stroke-width="stroke" stroke-linecap="round" stroke-linejoin="round" width="24" height="24" stroke-width="2" {
              path d="M20 11a8.1 8.1 0 0 0 -15.5 -2m-.5 -4v4h4"{}
              path d="M4 13a8.1 8.1 0 0 0 15.5 2m.5 4v-4h-4"{}
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
