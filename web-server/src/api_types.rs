use serde::{Deserialize, Serialize};

mod microbenchmark_kind;

pub use microbenchmark_kind::MicrobenchmarkKind;

/// Statistics that can be obtained for a given microbenchmark.
#[derive(Debug)]
pub struct BenchmarkResultsStatistics {
    /// The workgroup size that this was executed with.
    pub workgroup_size: (u32, u32, u32),
    /// The amount of results that were retrieved for this workgroup size and
    /// the applied filters.
    pub result_count: usize,
    /// The time per iteration.
    pub average_time_per_iter: f64,
    /// A custom result like FLOPS or bandwidth.
    pub average_custom_result: f64,
}

/// Filters that can be applied when obtaining statistical data of benchmark
/// results.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct BenchmarkResultsFilters {
    pub hardware: Option<Hardware>,
    pub operating_system: Option<String>,
    pub platform: Option<Platform>,
    pub microbenchmark: MicrobenchmarkKind,
}

/// Fields used when listing and querying available hardware
///
/// Not using the [NonEmptyString] for easy decoding from DB, i can assume the
/// strings aren't empty tho.
#[derive(Clone, Debug, Serialize, Deserialize)]
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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Platform {
    Chromium,
    Firefox,
    OtherBrowser,
    NativeVulkan,
    NativeMetal,
    NativeDx12,
}
