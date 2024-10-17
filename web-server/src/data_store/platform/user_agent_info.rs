use crate::data_store::non_empty_string::NonEmptyString;

/// Data extracted from the user agent header string using the
/// [uap-core specification](https://github.com/ua-parser/uap-core/blob/master/docs/specification.md) through the [ua_parser] crate.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DataStoreUserAgentStringInfo {
    pub operating_system: Option<DataStoreUserAgentOs>,
    pub device: Option<DataStoreUserAgentDevice>,
    pub user_agent: Option<DataStoreUserAgent>,
}

/// Operating system data extracted from user agent string.
///
/// Based on the [uap-core
/// specification](https://github.com/ua-parser/uap-core/blob/master/docs/specification.md).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DataStoreUserAgentOs {
    pub operating_system: NonEmptyString,
    pub major: Option<NonEmptyString>,
    pub minor: Option<NonEmptyString>,
    pub patch: Option<NonEmptyString>,
    pub patch_minor: Option<NonEmptyString>,
}

/// Device data extracted from user agent string.
///
/// Based on the [uap-core
/// specification](https://github.com/ua-parser/uap-core/blob/master/docs/specification.md).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DataStoreUserAgentDevice {
    pub device: NonEmptyString,
    pub brand: Option<NonEmptyString>,
    pub model: Option<NonEmptyString>,
}

/// User agent data extracted from user agent string.
///
/// Based on the [uap-core
/// specification](https://github.com/ua-parser/uap-core/blob/master/docs/specification.md).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DataStoreUserAgent {
    pub family: NonEmptyString,
    pub major: Option<NonEmptyString>,
    pub minor: Option<NonEmptyString>,
    pub patch: Option<NonEmptyString>,
    pub patch_minor: Option<NonEmptyString>,
}
