#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![feature(cfg_eval)]

use thiserror::Error;
use uwgpu::{CreatePipelineError, GetGPUContextError, MapTimestampResultError};

#[cfg(feature = "wasm")]
pub mod wasm;

pub use uwgpu;
pub mod convolution;
pub mod matmul;
pub mod memcpy;
pub mod reduction_sum;
pub mod scan;

/// An error trying to execute a benchmark
#[derive(Debug, Clone, Error)]
pub enum BenchmarkError {
    /// An error trying to get a handle on the GPU context.
    /// See [GetGPUContextError].
    #[error("error trying to get a handle on the GPU context: {0}")]
    GPUContext(#[from] GetGPUContextError),
    /// An error trying to create the compute pipeline for the microbenchmark.
    /// See [CreatePipelineError].
    #[error(
        "error trying create the compute pipeline for the microbenchmark: {0}"
    )]
    PipelineCreation(#[from] CreatePipelineError),
    /// An error trying to read the timestamp queries from the compute
    /// pipeline. See [MapTimestampResultError].
    #[error(
        "error trying to read the timestamp queries from the compute pipeline: {0}"
    )]
    MapTimestamp(#[from] MapTimestampResultError),
}
