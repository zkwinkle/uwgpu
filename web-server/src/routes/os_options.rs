use std::sync::Arc;

use axum::Extension;
use maud::{html, Markup};

use crate::data_store::DataStore;
use crate::error::ServerError;

#[cfg_attr(feature = "debug", axum::debug_handler)]
/// This endpoint returns the operating system options used by the
/// [HistoricalData](crate::components::historical_data::HistoricalData)
pub async fn os_options(
    Extension(data_store): Extension<Arc<dyn DataStore>>,
) -> Result<Markup, ServerError> {
    let operating_systems: Vec<String> =
        data_store.list_available_operating_systems().await?;

    Ok(html! {
        @for os in operating_systems {
            option value=(os) { (os) }
        }
    })
}
