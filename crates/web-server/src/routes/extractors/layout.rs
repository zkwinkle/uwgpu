use std::ops::Deref;

use axum::{
    async_trait,
    extract::{FromRequestParts, OriginalUri},
    http::request::Parts,
};
use maud::{html, Markup, PreEscaped, DOCTYPE};

use crate::components::navbar::Navbar;
//use crate::components::{navbar::Navbar, theme_selector::ThemeSelector};

/// Defines the base layout of a page that will wrap its contents with container
/// divs, headers, footers.
///
/// Usage:
/// ```ignore
/// async fn endpoint(layout: Layout) -> Markup {
///    layout.render(html! { "Hello, World!" })
/// }
/// ```
pub struct Layout {
    /// Note that the public server URL might be different from the request uri
    /// that the server receives due to the server being behind a proxy.
    ///
    /// I'm not confident on this, but decided to be safe just in case.
    public_server_url: &'static str,
    request_uri: OriginalUri,
}

#[async_trait]
impl<S> FromRequestParts<S> for Layout
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self {
            public_server_url: parts.extensions.get::<&'static str>().unwrap(),
            request_uri: OriginalUri::from_request_parts(parts, state)
                .await
                .unwrap(),
        })
    }
}

impl Layout {
    pub fn render(self, content: Markup) -> Markup {
        let link = |path: &str| format!("{}{}", self.public_server_url, path);

        html! {

        (DOCTYPE)
        head {
            link rel="stylesheet" type="text/css" href=(link("/stylesheet.css"));
            meta name="viewport" content="width=device-width, initial-scale=1";
            script defer src=(link("/htmx.min.js")) {}

            // Utility functions shared across pages. All functions in here
            // should get used in all pages, which is fine because we just have
            // the home page and microbenchmark pages (October 2024)
            script defer type="module" {(PreEscaped( format!(r##"
                let wasm_module = await import("./pkg/microbenchmarks.js");
                const TimeUnit = wasm_module.TimeUnit;
                const init = wasm_module.default;

                window.run_microbenchmark = run_microbenchmark;
                window.shuffle = shuffle;

                async function run_microbenchmark(microbenchmark_json,
                                                  wasm_benchmark_fn_str,
                                                  workgroups_array,
                                                  custom_result_fn_str,
                                                  create_custom_result_fn,
                                                  results_div,
                                                  disable_checkbox,
                                                  ) {{
                    let results_header_interval = null;
                    let error_binding = null;
                    try {{
                        await init();

                        // Shuffle so first ones don't have more executions than
                        // later ones.
                        shuffle(workgroups_array);
                        for (const workgroup_size of workgroups_array) {{
                            let results_header = document.createElement('h4');
                            results_div.appendChild(results_header);

                            results_header.textContent = "...";

                            let dotCount = 0;

                            // Animation for ... (while executing next benchmark)
                            results_header_interval = setInterval(() => {{
                              dotCount = (dotCount % 3) + 1;
                              results_header.textContent = ".".repeat(dotCount);
                            }}, 300);

                            let result;
                            const wasm_benchmark_fn = wasm_module[wasm_benchmark_fn_str];
                            if (Array.isArray(workgroup_size)) {{
                                result = await wasm_benchmark_fn(...workgroup_size);
                            }} else {{
                                result = await wasm_benchmark_fn(workgroup_size);
                            }}

                            if (!disable_checkbox.checked) {{
                                post_results(result, workgroup_size, microbenchmark_json, result[custom_result_fn_str]());
                            }}

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
                            custom_result_p.textContent = create_custom_result_fn(result);
                            results_div.appendChild(custom_result_p);
                        }};
                    }} catch (error) {{
                        let error_header = document.createElement('h2');
                        error_header.textContent = "An error has ocurred!";
                        results_div.appendChild(error_header);
                        let error_message = document.createElement('p');
                        error_message.textContent = error.toString();
                        results_div.appendChild(error_message);

                        error_binding = error;
                    }} finally {{
                        if (results_header_interval != null) {{
                            clearInterval(results_header_interval)
                        }}

                        if (error_binding != null) {{
                            throw error_binding;
                        }}
                    }}
                }}

                async function post_results(result, workgroup_size, microbenchmark_kind_json, custom_result) {{

                    const adapter = await navigator.gpu.requestAdapter();
                    const webgpu_adapter_info = {{
                        architecture: adapter.info.architecture,
                        description: adapter.info.description,
                        device: adapter.info.device,
                        vendor: adapter.info.vendor,
                    }}

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
                        benchmark_kind: microbenchmark_kind_json,
                        count: result[0].count,
                        total_time_spent: result[0].total_time_spent,
                        custom_result,
                    }}

                    fetch("{url}/results", {{
                      method: 'POST',
                      headers: {{
                        'Content-Type': 'application/json',
                      }},
                      body: JSON.stringify(post_result_request),
                    }})
                }}

                function shuffle(a) {{
                    for (let i = a.length - 1; i >= 0; i--) {{
                        const j = Math.floor(Math.random() * (i + 1));
                        [a[i], a[j]] = [a[j], a[i]];
                    }}
                }}

                "##, url=self.public_server_url) ))
            }
        }
        div id="theme-container" class="light" {
            div class="container" {
                (Navbar::from_urls(self.public_server_url, self.request_uri.deref()))
                div class="content-container" {
                    (content)
                }
            }
        }

        }
    }
}
