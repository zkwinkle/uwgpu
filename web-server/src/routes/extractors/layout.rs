use std::ops::Deref;

use axum::{
    async_trait,
    extract::{FromRequestParts, OriginalUri},
    http::request::Parts,
};
use maud::{html, Markup, PreEscaped, DOCTYPE};

use crate::components::{css::STYLESHEET, navbar::Navbar};
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
        html! {

        (DOCTYPE)
        head {
            ( STYLESHEET )
            meta name="viewport" content="width=device-width, initial-scale=1";
            script src="/public/htmx.min.js" {}

            // Utility functions shared across pages. All functions in here
            // should get used in all pages, which is fine because we just have
            // the home page and microbenchmark pages (October 2024)
            script {(PreEscaped( format!(r##"
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
                }}"##, url=self.public_server_url) ))
            }
        }
        div id="theme-container" class="light" {
            div class="container" {
                (Navbar::from_uri(self.request_uri.deref()))
                div class="content-container" {
                    (content)
                }
            }
        }

        }
    }
}
