use std::sync::Arc;

use axum::extract::Query;
use axum::Extension;
use maud::{html, Markup};
use serde::Deserialize;

use crate::api_types::{BenchmarkResultsFilters, Hardware, Platform};
use crate::data_store::DataStore;
use crate::error::ServerError;

#[derive(Debug, Deserialize)]
pub struct QueryParams {
    hardware: String,
    operating_system: String,
    platform: String,
}

#[cfg_attr(feature = "debug", axum::debug_handler)]
/// This endpoint returns a table with statistics of the historical data for
/// benchmark results.
/// The type of result taken into account can be filtered down.
pub async fn historica_data_table(
    Extension(data_store): Extension<Arc<dyn DataStore>>,
    Query(qp): Query<QueryParams>,
) -> Result<Markup, ServerError> {
    let hardware: Option<Hardware> = if qp.hardware.is_empty() {
        None
    } else {
        serde_json::from_str(&qp.hardware)?
    };

    let platform: Option<Platform> = if qp.platform.is_empty() {
        None
    } else {
        serde_json::from_str(&qp.platform)?
    };

    let operating_system: Option<String> = if qp.operating_system.is_empty() {
        None
    } else {
        Some(qp.operating_system)
    };

    let filters = BenchmarkResultsFilters {
        hardware,
        operating_system,
        platform,
    };

    Ok(html! {
        h1 { "TODO table with historic data!" }
        h2 { "Received parameters:" }
         p { (format!("{:?}", filters)) }
    })
}
