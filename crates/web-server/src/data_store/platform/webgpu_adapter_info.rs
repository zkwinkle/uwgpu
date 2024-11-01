use crate::data_store::non_empty_string::NonEmptyString;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DataStoreWebGpuAdapterInfo {
    pub architecture: Option<NonEmptyString>,
    pub description: Option<NonEmptyString>,
    pub device: Option<NonEmptyString>,
    pub vendor: Option<NonEmptyString>,
}
