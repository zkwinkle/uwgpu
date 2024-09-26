#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

use uwgpu::{CreatePipelineError, GetGPUContextError, MapTimestampResultError};

pub use uwgpu;
pub mod matmul;
pub mod memory;

/// An error trying to execute a benchmark
#[derive(Debug)]
pub enum BenchmarkError {
    /// An error trying to get a handle on the GPU context.
    /// See [GetGPUContextError].
    GPUContext(GetGPUContextError),
    /// An error trying to create the compute pipeline for the microbenchmark.
    /// See [CreatePipelineError].
    PipelineCreation(CreatePipelineError),
    /// An error trying to read the timestamp queries from the compute
    /// pipeline. See [MapTimestampResultError].
    MapTimestamp(MapTimestampResultError),
}
