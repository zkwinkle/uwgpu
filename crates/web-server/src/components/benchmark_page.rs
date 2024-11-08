use maud::{html, Markup, PreEscaped, Render};

use super::historical_data::HistoricalData;
use crate::api_types::MicrobenchmarkKind::{self, *};

pub struct MicrobenchmarkPage {
    pub microbenchmark: MicrobenchmarkKind,
    pub server_url: &'static str,
}

impl Render for MicrobenchmarkPage {
    fn render(&self) -> Markup {
        html! {

        header { h1 { (self.microbenchmark.title()) } }
        p { (self.description()) }

        div class="microbenchmark-view-selectors" role="tablist" {
            a href="#execution" id="execution-view-selector" role="tab" aria-controls="execution-view" { "[Microbenchmark Execution]" }
            a href="#historical" id="historical-view-selector" role="tab" aria-controls="historical-view" { "[Historical Data]" }
        }

        div id="execution-view" role="tabpanel" {
            h2 { "Execution" }
            p { "Click the \"Start\" button to execute the microbenchmark suite. For more accurate results please close all other applications." }
            button id="run_microbenchmark" { "Start" }

            div class="disable-checkbox" {
                input type="checkbox" id="disable_data_collection" ;
                label for="disable_data_collection" {
                "Select this checkbox to opt out of data collection. Benchmark results contribute to a growing database of performance data. Please consider submitting your data to support this project."
                }
            }
            div id="execution-results" {}
            script defer {
                (PreEscaped(format!(r#"
                let results_div = document.getElementById('execution-results');

                let button = document.getElementById('run_microbenchmark');
                let disable_checkbox = document.getElementById('disable_data_collection');
                button.addEventListener('click', async () => {{
                    button.disabled = true;
                    results_div.innerHTML = "";
                    try {{
                        await {run_microbenchmark};
                    }} finally {{
                        button.disabled = false;
                    }}
                }});

                "#,
                run_microbenchmark=self.microbenchmark.run_microbenchmark_fn(),
                )))
            }
        }

        div id="historical-view" role="tabpanel" {
            (HistoricalData {
                microbenchmark:self.microbenchmark,
                server_url: self.server_url
            })
        }

        script { (PreEscaped(r##"
        const tabs = document.querySelectorAll("[role='tab']");
        const views = document.querySelectorAll("[role='tabpanel']");

        function update_view_from_fragment() {
            // Default to execution view
            const hash = window.location.hash || "#execution";
            let active_view_id;

            tabs.forEach(tab => {
                const is_active = tab.getAttribute("href") === hash;

                tab.classList.toggle("active", is_active);
                tab.setAttribute("aria-selected", is_active);

                if (is_active) {
                    active_view_id = tab.getAttribute("aria-controls");
                }
            });

            views.forEach(view => {
                const is_active = view.id === active_view_id;
                view.setAttribute("aria-selected", is_active);
                view.toggleAttribute("hidden", !is_active);

                if (is_active) {
                    view.removeAttribute("style");
                } else {
                    view.style.display = "none";
                }
            });
        }

        update_view_from_fragment();

        window.addEventListener("hashchange", update_view_from_fragment);
        "##)) }

        }
    }
}

impl MicrobenchmarkPage {
    fn description(&self) -> &'static str {
        match self.microbenchmark {
            Matmul => "This microbenchmark tests the performance of a naive matrix multiplication between two 1024x1024 matrices of 32bit floats.",
            Reduction => "This microbenchmark tests the performance of a naive single-pass reduction sum on a 1MiB buffer of random data.",
            Convolution => "This microbenchmark tests the performance of a naive convolution between a 1024x1024 matrix and a 3x3 kernel.",
            Scan => "This microbenchmark tests the performance of a naive multi-pass scan using the Sklansky technique over a 1MB buffer of random data.",
            BufferToBuffer => {
                "This microbenchmark tests the performance of copying memory between buffers in the GPU. It's a naive un-optimized implementation so it won't reflect the true bandwidth of your GPU."
            }
            BufferToTexture => {
                "This microbenchmark tests the performance of copying memory from a buffer to a storage texture in the GPU. It's a naive un-optimized implementation so it won't reflect the true bandwidth of your GPU."
            }
            TextureToTexture => {
                "This microbenchmark tests the performance of copying memory from a texture to a storage texture in the GPU. It's a naive un-optimized implementation so it won't reflect the true bandwidth of your GPU."
            }
            BufferSequential => {
                "This microbenchmark tests the performance of accessing buffer elements in a sequential manner."
            }
            BufferShuffled => {
                todo!()
            }
        }
    }
}
