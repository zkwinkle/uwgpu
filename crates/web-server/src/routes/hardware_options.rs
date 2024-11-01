use std::sync::Arc;

use axum::Extension;
use maud::{html, Markup};

use crate::api_types::Hardware;
use crate::data_store::DataStore;
use crate::error::ServerError;

#[cfg_attr(feature = "debug", axum::debug_handler)]
/// This endpoint returns the hardware options used by the
/// [HistoricalData](crate::components::historical_data::HistoricalData)
pub async fn hardware_options(
    Extension(data_store): Extension<Arc<dyn DataStore>>,
) -> Result<Markup, ServerError> {
    let hardwares: Vec<Hardware> =
        data_store.list_available_hardwares().await?;

    Ok(html! {
        @for hardware in hardwares {
            option value=(serde_json::to_string(&hardware).unwrap())
            { (hardware.webgpu_vendor + " " + &hardware.webgpu_architecture) }
        }
    })
}
