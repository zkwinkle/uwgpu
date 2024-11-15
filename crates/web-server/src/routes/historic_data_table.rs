use std::sync::Arc;

use axum::extract::Query;
use axum::Extension;
use maud::{html, Markup};
use serde::Deserialize;

use crate::api_types::{
    BenchmarkResultsFilters, Hardware, MicrobenchmarkKind, Platform,
};
use crate::data_store::DataStore;
use crate::error::ServerError;

#[derive(Debug, Deserialize)]
pub struct QueryParams {
    hardware: Option<String>,
    operating_system: Option<String>,
    platform: String,
    /// Only field that can be decoded by axum because it can't be an empty
    /// string and it's manually set from hx-vals instead of grabbing it from
    /// a <select> element (htmx will automatically wrap it with "quotes",
    /// making serde_json think it's a string and not an object)
    microbenchmark: MicrobenchmarkKind,
}

#[cfg_attr(feature = "debug", axum::debug_handler)]
/// This endpoint returns a table with statistics of the historical data for
/// benchmark results.
/// The type of result taken into account can be filtered down.
pub async fn historica_data_table(
    Extension(data_store): Extension<Arc<dyn DataStore>>,
    Query(qp): Query<QueryParams>,
) -> Result<Markup, ServerError> {
    let hardware: Option<Hardware> = qp
        .hardware
        .and_then(|hardware| {
            if hardware.is_empty() {
                None
            } else {
                Some(serde_json::from_str(&hardware))
            }
        })
        .transpose()?;

    let platform: Option<Platform> = if qp.platform.is_empty() {
        None
    } else {
        serde_json::from_str(&qp.platform)?
    };

    let operating_system: Option<String> = qp.operating_system.and_then(|os| {
        if os.is_empty() {
            None
        } else {
            Some(os)
        }
    });

    let filters = BenchmarkResultsFilters {
        hardware,
        operating_system,
        platform,
        microbenchmark: qp.microbenchmark,
    };

    let mut result_stats = data_store
        .get_benchmark_results_statistics(filters.clone())
        .await?;

    result_stats.sort_by(|a, b| {
        let wg_a = a.workgroup_size;
        let wg_b = b.workgroup_size;

        let sum_a = wg_a.0 * wg_a.1 * wg_a.2;
        let sum_b = wg_b.0 * wg_b.1 * wg_b.2;

        sum_a.cmp(&sum_b).then_with(|| wg_a.cmp(&wg_b))
    });

    if result_stats.is_empty() {
        Ok(html! {
           h2 { "There are no records that match the chosen filters." }
        })
    } else {
        Ok(html! {
            table {
                tr {
                    th { "Workgroup size" }
                    th { "Number of Entries" }
                    th { "Average time per iteration (ms)" }
                    th { (format!("Average {}",custom_metric_name(filters.microbenchmark))) }
                }
                @for result in result_stats {
                tr {
                    td { ( format!("{}x{}x{}", result.workgroup_size.0, result.workgroup_size.1, result.workgroup_size.2) ) }
                    td { ( format!("{}", result.result_count) ) }
                    td { ( format!("{:.3}", result.average_time_per_iter / 1_000_000.0) ) }
                    td { ( format!("{:.3}", result.average_custom_result / 1_000_000_000.0) ) }
                }
                }
            }
        })
    }
}

fn custom_metric_name(microbenchmark: MicrobenchmarkKind) -> &'static str {
    use MicrobenchmarkKind::*;
    match microbenchmark {
        Matmul | Reduction | Convolution | Scan => "GFLOPS",
        BufferSequential | BufferShuffled | BufferToBuffer
        | BufferToTexture | TextureToTexture => "Bandwidth (GB/s)",
    }
}
