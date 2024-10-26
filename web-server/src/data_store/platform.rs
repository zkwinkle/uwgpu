use crate::data_store::platform::wgpu_adapter_info::DataStoreWgpuAdapterInfo;
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use user_agent_info::DataStoreUserAgentStringInfo;
use webgpu_adapter_info::DataStoreWebGpuAdapterInfo;

pub mod user_agent_info;
pub mod webgpu_adapter_info;
pub mod wgpu_adapter_info;

mod interface;
pub use interface::DataStorePlatformInterface;

/// Datastore version of the platform info that we care to store for each
/// execution run.
#[derive(Debug)]
pub struct DataStorePlatform {
    /// The ID of this platform record.
    pub platform_id: Uuid,

    /// The user agent information extracted from the user agent header.
    ///
    /// It's optional because the user agent header is not mandatory.
    pub user_agent: Option<DataStoreUserAgentStringInfo>,

    /// The adapter info record corresponding to this platform, provided by
    /// [wgpu](uwgpu::wgpu). The adapter is seen as part of the platform.
    pub wgpu_adapter_info: DataStoreWgpuAdapterInfo,

    /// The adapter info record corresponding to this platform but provided by
    /// the WebGPU javascript APIs. This info will only be available from web
    /// targets, which is also where it's most useful since the wgpu adapter
    /// info is lacking in those platforms.
    pub webgpu_adapter_info: Option<DataStoreWebGpuAdapterInfo>,
}

/// Fields used when listing and querying available hardware
///
/// Not using the [NonEmptyString] for easy decoding from DB, i can assume the
/// strings aren't empty tho.
#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct Hardware {
    pub webgpu_vendor: String,
    pub webgpu_architecture: String,
}

/// Supported general "platforms" for filtering results
///
/// These all have different ways of being queried for, so that's why we decide
/// to just state them in this enum instead of doing some heuristic query of the
/// DB to find the available variants. (Like we do for [Hardware] variants for
/// example.)
#[derive(Serialize, Deserialize)]
pub enum Platform {
    Chromium,
    Firefox,
    OtherBrowser,
    NativeVulkan,
    NativeMetal,
    NativeDx12,
}

/// This structure describes a platform to create in the datastore.
pub struct DataStoreCreatePlatform {
    /// Available when user agent header is present.
    pub user_agent: Option<DataStoreUserAgentStringInfo>,
    /// Available when benchmark is executed in a web target.
    pub webgpu_adapter_info: Option<DataStoreWebGpuAdapterInfo>,
    /// Always available.
    pub wgpu_adapter_info: DataStoreWgpuAdapterInfo,
}
