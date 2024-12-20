use std::sync::LazyLock;

use axum::Extension;
use maud::{html, Markup, PreEscaped};
use rand::{seq::SliceRandom, thread_rng};

use crate::api_types::MicrobenchmarkKind::{self, *};

use super::extractors::Layout;

/// Make sure this array contains all the supported microbenchmarks.
const ALL_MICROBENCHMARKS: &[MicrobenchmarkKind] = &[
    Matmul,
    Convolution,
    Reduction,
    Scan,
    BufferToBuffer,
    BufferToTexture,
    TextureToTexture,
];

#[cfg_attr(feature = "debug", axum::debug_handler)]
pub async fn home(
    layout: Layout,
    Extension(server_url): Extension<&'static str>,
) -> Markup {
    // Array of [title, run_microbenchmark_fn] pairs.
    // As a js string.
    static MICROBENCHMARK_DATA: LazyLock<String> = LazyLock::new(|| {
        // randomize order to avoid biasing results towards the first
        // microbenchmarks due to people refreshing or whatever
        let mut microbenchmarks = ALL_MICROBENCHMARKS.to_vec();
        microbenchmarks.shuffle(&mut thread_rng());
        // (title, run_microbenchmark callback)
        let microbenchmarks: Vec<(&'static str, String)> = microbenchmarks
            .iter()
            .map(|mb| {
                (
                    mb.title(),
                    format!(
                        "async ()=>{{await {}}}",
                        mb.run_microbenchmark_fn()
                    ),
                )
            })
            .collect();

        let mut microbenchmark_data = String::from("[");
        for (title, callback) in microbenchmarks {
            microbenchmark_data += &format!("[\"{}\",{}],", title, callback);
        }
        microbenchmark_data.push(']');

        microbenchmark_data
    });

    layout.render( html! {
        header {
            h1 { "µwgpu" }
        }
        p { "This page lets you run GPU microbenchmarks to measure your hardware's performance and help us build a GPU performance dataset. You can check out stats for each microbenchmark on its own page." }
        p { "Part of the "
            a href="https://github.com/zkwinkle/uwgpu" {
                "µwgpu project "
                img class="inline-image"
                    src=(format!("{}/github-mark.svg", server_url))
                    alt="github repository" {}
            }
            ", which aims to enable GPU microbenchmarking and gather performance stats across a wide range of hardware and platforms." }
        // TODO: CSV download of dataset.
        //p { "To download the dataset for your analysis, "
        //    a href="/dataset.csv" {"click here (TODO)"}
        //    " to obtain it as a CSV file." }
        h3 { "Seeking academic advisor!" }
        p { "I’m a Computer Engineering student preparing for my graduation project next semester and believe this tool has potential for interesting and useful research. However, it's really hard to find experts in this area, specially in my country." }
        p { "If you'd be interested in exploring an advisory role, or even just to chat, reach out to me at ignaevc [at] gmail [dot] com." }
        h3 { "Browser compatibility" }
        p { "Currently supported on Firefox Nightly and Chrome / Chromium-based browsers (Linux users may need to "
            a href="https://github.com/gpuweb/gpuweb/wiki/Implementation-Status#chromium-chrome-edge-etc"
            { "enable specific flags" }
        ")."}

        h2 { "Execution" }
        p { "Click the \"Start\" button to execute the full microbenchmark suite. For more accurate results please close all other applications. Don't refresh; it will stop execution." }
        p { b { "Time estimate: " } "takes around 3 minutes on 12th gen intel integrated graphics." }

        button id="run_all_button" { "Start" }

        div class="disable-checkbox" {
            input type="checkbox" id="disable_data_collection" ;
            label for="disable_data_collection" {
            "Select this checkbox to opt out of data collection. Benchmark results contribute to a growing database of performance data. Please consider submitting your data to support this project."
            }
        }
        p id="execution-results" {}
        script defer {
            (PreEscaped(format!(r#"
            let results_div = document.getElementById('execution-results');

            let button = document.getElementById('run_all_button');
            let disable_checkbox = document.getElementById('disable_data_collection');
            let microbenchmarks = {microbenchmarks};

            button.addEventListener('click', async () => {{
                button.disabled = true;
                results_div.innerHTML = "";
                // Shuffle so first ones don't have more executions than later
                // ones.
                shuffle(microbenchmarks);
                try {{
                    for (let microbenchmark of microbenchmarks) {{
                        let [title, run_microbenchmark_fn] = microbenchmark;

                        let microbenchmark_header = document.createElement('h3');
                        microbenchmark_header.textContent = title;
                        results_div.appendChild(microbenchmark_header);

                        await run_microbenchmark_fn();
                    }}
                    alert("Execution finished");
                }} finally {{
                    button.disabled = false;
                }}
            }});

            "#,
            microbenchmarks=&*MICROBENCHMARK_DATA,
            )))
        }
    })
}
