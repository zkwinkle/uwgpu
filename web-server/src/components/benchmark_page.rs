use maud::{html, Markup, PreEscaped, Render};

use super::historical_data::HistoricalData;
use crate::api_types::MicrobenchmarkKind::{self, *};

pub struct MicrobenchmarkPage {
    pub microbenchmark: MicrobenchmarkKind,
}

impl Render for MicrobenchmarkPage {
    fn render(&self) -> Markup {
        let title = self.title();

        html! {

        header { h1 { (title) } }
        p { (self.description()) }

        div class="microbenchmark-view-selectors" role="tablist" {
            a href="#execution" id="execution-view-selector" role="tab" aria-controls="execution-view" { "[Microbenchmark Execution]" }
            a href="#historical" id="historical-view-selector" role="tab" aria-controls="historical-view" { "[Historical Data]" }
        }

        div id="execution-view" role="tabpanel" {
            h2 { "Execution" }
            p { "Click the \"Start\" button to execute the microbenchmark suite. For more accurate results please close all other applications." }
            button id=(format!("run_{}_microbenchmark", title)){ "Start" }

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

                let button = document.getElementById('run_{name}_microbenchmark');
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
                name=title,
                run_microbenchmark=self.run_microbenchmark_fn(),
                )))
            }
        }

        div id="historical-view" role="tabpanel" {
            (HistoricalData {microbenchmark: self.microbenchmark})
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

    /// Crafts the call to the JS `run_microbenchmark` function defined on the
    /// [Layout](crate::routes::extractors::Layout) based on the specific
    /// microbenchmark.
    ///
    /// This function assumes the existence of 2 variables:
    ///
    /// - `results_div`: the element results will be appended to.
    /// - `disable_checkbox`: Checkbox element that if checked will disable
    ///                       POSTing results.
    fn run_microbenchmark_fn(&self) -> String {
        format!(r#"run_microbenchmark({microbenchmark_json},
                                      "{wasm_benchmark_fn_str}",
                                      {workgroups_array},
                                      "{custom_result_fn_str}",
                                      (result) => {create_custom_result},
                                      results_div,
                                      disable_checkbox)"#,
            microbenchmark_json=serde_json::to_string(&self.microbenchmark).unwrap(),
            wasm_benchmark_fn_str=self.wasm_benchmark_function(),
            workgroups_array=self.benchmark_workgroups(),
            custom_result_fn_str=self.custom_result_function(),
            create_custom_result = self.custom_result(),
        )
    }

    fn title(&self) -> &'static str {
        match self.microbenchmark {
            Matmul => "Matrix Multiplication",
            Reduction => "Reduction",
            Convolution => "Convolution",
            Scan => "Scan",
            BufferSequential => "Sequential Buffer Memory Access",
            BufferShuffled => "Shuffled Buffer Memory Accesses",
            BufferToTexture => "Memory Copy From Buffer To Texture",
            TextureToTexture => "Memory Copy From Texture To Texture",
        }
    }

    fn description(&self) -> &'static str {
        match self.microbenchmark {
            Matmul => "This microbenchmark tests the performance of multiplying two 1024x1024 matrices of 32bit floats together.",
            Reduction => todo!(),
            Convolution => todo!(),
            Scan => todo!(),
            BufferSequential => {
                "This microbenchmark tests the performance of accessing buffer elements in a sequential manner."
            }
            BufferShuffled => {
                todo!()
            }
            BufferToTexture => {
                todo!()
            }
            TextureToTexture => {
                todo!()
            }
        }
    }

    fn wasm_benchmark_function(&self) -> &'static str {
        match self.microbenchmark {
            Matmul => "wasm_matmul_benchmark",
            Reduction => todo!(),
            Convolution => todo!(),
            Scan => todo!(),
            BufferSequential => "wasm_buffer_sequential_benchmark",
            BufferShuffled => todo!(),
            BufferToTexture => todo!(),
            TextureToTexture => todo!(),
        }
    }

    fn benchmark_workgroups(&self) -> &'static str {
        match self.microbenchmark {
            // Accessing same-row should be faster than accessing different rows
            // which is why we use column-dominant workgroups
            Matmul => "[[4, 8], [2, 16], [1, 32], [8, 8], [4, 16], [2, 32], [1, 64], [8, 16], [4, 32], [2, 64], [1, 128], [16, 16], [8, 32], [4, 64], [2, 128], [1, 256]]",
            Reduction => todo!(),
            Convolution => todo!(),
            Scan => todo!(),
            BufferSequential => "[32, 64, 128, 256]",
            BufferShuffled => todo!(),
            BufferToTexture => todo!(),
            TextureToTexture => todo!(),
        }
    }

    /// JS instructions for getting an extra line with a custom result such as
    /// FLOPs or Bandwidth
    ///
    /// Assume there is a `result` variable in the script with the benchmark
    /// results.
    ///
    /// The code should be an expression to create a string for the line with
    /// the custom result.
    fn custom_result(&self) -> &'static str {
        match self.microbenchmark {
            Matmul | Reduction | Convolution | Scan => {
                r#"
                "GFLOPS: " + (result.flops() / 1_000_000_000).toFixed(3)
            "#
            }
            BufferSequential | BufferShuffled | BufferToTexture
            | TextureToTexture => {
                r#"
                "Bandwidth (GB/s): " + (result.bandwidth() / 1_000_000_000).toFixed(3)
            "#
            }
        }
    }

    fn custom_result_function(&self) -> &'static str {
        match self.microbenchmark {
            Matmul | Reduction | Convolution | Scan => "flops",
            BufferSequential | BufferShuffled | BufferToTexture
            | TextureToTexture => "bandwidth",
        }
    }
}
