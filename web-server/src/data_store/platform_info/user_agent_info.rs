/// Data extracted from the user agent header string using the
/// [uap-core specification](https://github.com/ua-parser/uap-core/blob/master/docs/specification.md) through the [ua_parser] crate.
pub struct DataStoreUserAgentStringInfo {
    pub operating_system: Option<DataStoreUserAgentOs>,
    pub device: Option<DataStoreUserAgentDevice>,
    pub user_agent: Option<DataStoreUserAgent>,
}

/// Operating system data extracted from user agent string.
///
/// Based on the [uap-core
/// specification](https://github.com/ua-parser/uap-core/blob/master/docs/specification.md).
pub struct DataStoreUserAgentOs {
    pub operating_system: String,
    pub major: Option<String>,
    pub minor: Option<String>,
    pub patch: Option<String>,
    pub patch_minor: Option<String>,
}

/// Device data extracted from user agent string.
///
/// Based on the [uap-core
/// specification](https://github.com/ua-parser/uap-core/blob/master/docs/specification.md).
pub struct DataStoreUserAgentDevice {
    pub device: String,
    pub brand: Option<String>,
    pub model: Option<String>,
}

/// User agent data extracted from user agent string.
///
/// Based on the [uap-core
/// specification](https://github.com/ua-parser/uap-core/blob/master/docs/specification.md).
pub struct DataStoreUserAgent {
    pub family: String,
    pub major: Option<String>,
    pub minor: Option<String>,
    pub patch: Option<String>,
    pub patch_minor: Option<String>,
}
