use maud::{html, Markup, PreEscaped, Render};

use super::historical_data::HistoricalData;
use crate::api_types::MicrobenchmarkKind::{self, *};

pub struct MicrobenchmarkPage<'a> {
    pub microbenchmark: MicrobenchmarkKind,
    pub server_url: &'a str,
}

impl Render for MicrobenchmarkPage<'_> {
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
                input type="checkbox" id=(format!("disable_{}_data_collection", title));
                label for=(format!("disable_{}_data_collection", title)) {
                "Select this checkbox to opt out of data collection. Benchmark results contribute to a growing database of performance data. Please consider submitting your data to support this project."
            }
            }
            div id="execution-results" {}
            script type="module" {
                (PreEscaped(format!(r#"
                import init, {{ {benchmark_name}, TimeUnit }} from "./public/pkg/microbenchmarks.js";

                let results_div = document.getElementById('execution-results');
                let results_header_interval = null;
                let results_header;

                let button = document.getElementById('run_{name}_microbenchmark');
                button.addEventListener('click', async () => {{
                    button.disabled = true;
                    try {{
                        results_div.innerHTML = "";
                        await run_microbenchmark();
                    }} catch (error) {{
                        let error_header = document.createElement('h2');
                        error_header.textContent = "An error has ocurred!";
                        results_div.appendChild(error_header);
                        let error_message = document.createElement('p');
                        error_message.textContent = error.toString();
                        results_div.appendChild(error_message);

                        setTimeout(() => {{ throw error; }}, 100);
                    }} finally {{
                        button.disabled = false;

                        if (results_header_interval != null) {{
                            clearInterval(results_header_interval)
                        }}
                    }}
                }});

                async function run_microbenchmark() {{
                    await init();

                    for (const workgroup_size of {workgroups_array}) {{
                        results_header = document.createElement('h4');
                        results_div.appendChild(results_header);

                        results_header.textContent = "...";

                        let dotCount = 0;

                        // Animation for ... (while executing next benchmark)
                        results_header_interval = setInterval(() => {{
                          dotCount = (dotCount % 3) + 1;
                          results_header.textContent = ".".repeat(dotCount);
                        }}, 300);

                        let result;
                        if (Array.isArray(workgroup_size)) {{
                            result = await {benchmark_name}(...workgroup_size);
                        }} else {{
                            result = await {benchmark_name}(workgroup_size);
                        }}

                        post_results(result, workgroup_size);

                        clearInterval(results_header_interval);
                        results_header_interval = null;

                        if (Array.isArray(workgroup_size)) {{
                            results_header.textContent = "Workgroup size: " + workgroup_size.join('x');
                        }} else {{
                            results_header.textContent = "Workgroup size: " + workgroup_size;
                        }}

                        let total_time_spent_p = document.createElement('p');
                        total_time_spent_p.textContent = "Total time spent: " + result[0].total_time(TimeUnit.Second).toFixed(3) + "s";
                        results_div.appendChild(total_time_spent_p);

                        let time_per_iter_p = document.createElement('p');
                        time_per_iter_p.textContent = "Time per iteration: " + result[0].time_per_iteration(TimeUnit.Milli).toFixed(3) + "ms";
                        results_div.appendChild(time_per_iter_p);

                        let custom_result_p = document.createElement('p');
                        custom_result_p.textContent = {create_custom_result};
                        results_div.appendChild(custom_result_p);
                    }};
                }}

                const adapter = await navigator.gpu.requestAdapter();
                const webgpu_adapter_info = {{
                    architecture: adapter.info.architecture,
                    description: adapter.info.description,
                    device: adapter.info.device,
                    vendor: adapter.info.vendor,
                }}

                async function post_results(result, workgroup_size) {{
                    let workgroup_size_array;
                    if (Array.isArray(workgroup_size)) {{
                        workgroup_size_array = workgroup_size;
                    }} else {{
                        workgroup_size_array = [workgroup_size];
                    }}

                    const platform_info = {{
                        wgpu_adapter_info: result[0].adapter_info.to_js(),
                        webgpu_adapter_info,
                    }}

                    const post_result_request = {{
                        platform_info,
                        workgroup_size: workgroup_size_array,
                        benchmark_kind: {microbenchmark_json},
                        count: result[0].count,
                        total_time_spent: result[0].total_time_spent,
                        custom_result: result.{custom_result_fn}(),
                    }}

                    fetch("{url}/results", {{
                      method: 'POST',
                      headers: {{
                        'Content-Type': 'application/json',
                      }},
                      body: JSON.stringify(post_result_request),
                    }})
                }}
                "#,
                name=title,
                benchmark_name=self.wasm_benchmark_function(),
                workgroups_array=self.benchmark_workgroups(),
                create_custom_result = self.custom_result(),
                microbenchmark_json=serde_json::to_string(&self.microbenchmark).unwrap(),
                url=self.server_url,
                custom_result_fn=self.custom_result_function(),
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

impl MicrobenchmarkPage<'_> {
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
