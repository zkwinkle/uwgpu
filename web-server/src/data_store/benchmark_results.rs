use sqlx::types::Uuid;

mod computational;
mod memory;

pub use computational::*;
pub use memory::*;

mod interface;
pub use interface::DataStoreBenchmarkResultsInterface;

/// Datastore version of a benchmark's execution results.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DataStoreBenchmarkResults {
    /// The id of the platform  record corresponding to these results.
    /// See [DataStorePlatform](super::platform::DataStorePlatform)
    pub platform_id: Uuid,

    /// Total iterations that counted towards the result
    pub count: u32,
    /// Total time spent executing the benchmark.
    pub total_time_spent: f64,

    /// The size of the workgroups used.
    pub workgroup_size: (u32, u32, u32),

    /// Specific data and metrics stored depending on the type of benchmark.
    pub kind: DataStoreBenchmarkKind,
}

/// The specific type of benchmark this is and the specific data associated
///
/// Should be implemented as a XOR CHECK on nullable foreign key ids such that
/// 1 and only 1 is always available and links to the different kinds of
/// specific info tables.
#[derive(Debug, Clone)]
pub enum DataStoreBenchmarkKind {
    /// Computational benchmarks that need to store computational-style info
    /// like FLOPS
    Computational(DataStoreComputationalBenchmark),
    /// Memory copy related benchmarks that need to store specific info like
    /// bandwidth.
    Memory(DataStoreMemoryBenchmark),
}

pub struct DataStoreCreateBenchmarkResult {
    /// The ID of the platform this benchmark was run on
    pub platform_id: Uuid,
    /// Total iterations that counted towards the result
    pub count: u32,
    /// Total time spent executing the benchmark.
    pub total_time_spent: f64,
    /// The size of the workgroups used.
    pub workgroup_size: (u32, u32, u32),
    /// Specific data and metrics stored depending on the type of benchmark.
    pub kind: DataStoreBenchmarkKind,
}
