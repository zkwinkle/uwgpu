use maud::{html, Markup, PreEscaped};

use super::extractors::Layout;

#[cfg_attr(feature = "debug", axum::debug_handler)]
pub async fn placeholder(layout: Layout) -> Markup {
    layout.render( html! {
        header {
            h1 { "wgpu microbenchmarks" }
        }
        p { "This page lets you execute GPU microbenchmarks to measure your hardware's performance and also help us collect a dataset of GPU performance characteristics." }
        p { "To download the dataset for your analysis, "
            a href="/dataset.csv" {"click here (TODO)"}
            " to obtain it as a CSV file." }
        p { "Click the \"Start\" button to execute the entire suite of
            microbenchmarks." }
        button id="run_all_button" { "Start" }

        p id="results_log" {}

        script type="module" {
            (PreEscaped(r#"
              import init, {wasm_matmul_benchmark} from "./public/pkg/microbenchmarks.js";

              async function run_all_microbenchmarks() {
                  await init();
                  console.log("wuT");
                  let results = await wasm_matmul_benchmark(64, 1);
                  console.log("yay!");
                  console.debug(results);
                  console.log("results?", results);
                  console.log("time per iter", results.time_per_iteration_ms());
                  console.log("adapter info", results.adapter_info);
              }

              document.getElementById('run_all_button').addEventListener('click', run_all_microbenchmarks);
            "#))
            }
    })
}
