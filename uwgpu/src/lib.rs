#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

// Re-export so that users of this library can use wgpu types.
// Upgrading wgpu major version means a semver breaknig change for this library
// as well. Could use my own type wrappers to avoid that. Idea to dwell on...
pub use wgpu;
pub use wgpu_async;

use std::mem::size_of;

use wgpu::{
    CommandBuffer, CommandEncoder, CommandEncoderDescriptor,
    ComputePassDescriptor, ComputePassTimestampWrites, MapMode, QuerySet,
    QueryType,
};
use wgpu_async::{AsyncBuffer, AsyncDevice};

mod gpu;
mod pipeline;

pub use gpu::*;
pub use pipeline::*;

#[cfg(target_arch = "wasm32")]
mod wasm_utils;

/// This type represents the parameters for running a benchmark.
///
/// The benchmark will run 2 compute passes in the following order:
///
/// 1. If `warmup_count` is >0, it will run a compute pass executing the shader
///    `warmup_count` times.
///
/// 2. After that it'll run the actual benchmark compute pass which will be
///    timed. The time taken will be returned in the [BenchmarkResults]
///    `total_time_spent` field.
#[derive(Clone)]
pub struct Benchmark<'a> {
    /// The number of warm-up iterations to run before starting the actual
    /// benchmarking process.
    /// This will be executed for each shader invocation specified through the
    /// [PipelineParameters] `workgroups` field.
    pub warmup_count: usize,

    /// The number of iterations of the benchmark to execute.
    /// This will be executed for each shader invocation specified through the
    /// [PipelineParameters] `workgroups` field.
    pub count: usize,

    /// Optional callback to encode any last commands in the command buffer
    /// before execution.
    ///
    /// A common usage of this callback is copying the output from the shader
    /// to a buffer that has the [MAP_READ](wgpu::BufferUsages::MAP_READ)
    /// usage flag set.
    ///
    /// The callback will be called after benchmark compute pass. Right before
    /// calling `encoder.finish()`.
    ///
    /// Will NOT be run after the warmup pass, only after the actual benchmark
    /// passes.
    pub finalize_encoder_callback: Option<&'a dyn Fn(&mut CommandEncoder)>,
}

/// Results from executing a benchmark with [Benchmark::run].
///
/// All timing quantities are given in nanoseconds
#[derive(Debug)]
pub struct BenchmarkResults {
    /// Total iterations ran, this should be equal to the `count` field in the
    /// [Benchmark] that was ran, but its also provided here for
    /// convenience.
    pub count: usize,

    /// Total time spent executing the benchmark.
    pub total_time_spent: f64,
}

impl BenchmarkResults {
    /// Get the total time spent in the time unit given.
    pub fn total_time(&self, unit: TimeUnit) -> f64 {
        nano_to_unit(self.total_time_spent, unit)
    }

    /// Get the time spent per iteration in the time unit given
    pub fn time_per_iteration(&self, unit: TimeUnit) -> f64 {
        nano_to_unit(self.total_time_spent, unit) / (self.count as f64)
    }
}

impl Benchmark<'_> {
    /// Runs the benchmark using the provided compute pipeline
    ///
    /// See [MapTimestampResultError] for the failure mode of this operation.
    pub async fn run<'a>(
        &self,
        pipeline: BenchmarkComputePipeline<'a>,
    ) -> Result<BenchmarkResults, MapTimestampResultError> {
        let timestamp_query = TimestampQuery::new(&pipeline.gpu.device);

        let warmup_command_buf = self.warmup_pass(&pipeline);
        let benchmark_command_buf =
            self.benchmark_pass(&pipeline, &timestamp_query);
        let resolve_timestamp_pass =
            self.timestamp_pass(&pipeline, &timestamp_query);

        pipeline.gpu.queue.submit([
            warmup_command_buf,
            benchmark_command_buf,
            resolve_timestamp_pass,
        ]);

        let ts_data = timestamp_query.get_timestamp_result().await?;
        let ts_period = pipeline.gpu.queue.get_timestamp_period();

        Ok(BenchmarkResults {
            count: self.count,
            total_time_spent: u64::wrapping_sub(ts_data[1], ts_data[0]) as f64
                * (ts_period as f64),
        })
    }

    /// Warmup compute pass
    fn warmup_pass(
        &self,
        pipeline: &BenchmarkComputePipeline,
    ) -> CommandBuffer {
        let mut encoder = pipeline
            .gpu
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });

        let mut warmup_pass =
            encoder.begin_compute_pass(&ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });

        warmup_pass.set_pipeline(&pipeline.pipeline);
        warmup_pass.set_bind_group(0, &pipeline.bind_group, &[]);

        for _ in 0..self.warmup_count {
            warmup_pass.dispatch_workgroups(
                pipeline.workgroups_dispatch.0,
                pipeline.workgroups_dispatch.1,
                pipeline.workgroups_dispatch.2,
            )
        }

        drop(warmup_pass); // has to be dropped before finishing commands

        encoder.finish()
    }

    /// Benchmark compute pass
    fn benchmark_pass(
        &self,
        pipeline: &BenchmarkComputePipeline,
        timestamp_query: &TimestampQuery,
    ) -> CommandBuffer {
        let query_set = &timestamp_query.query_set;

        let mut encoder = pipeline
            .gpu
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });

        let mut bench_pass =
            encoder.begin_compute_pass(&ComputePassDescriptor {
                label: None,
                timestamp_writes: Some(ComputePassTimestampWrites {
                    query_set: &query_set,
                    beginning_of_pass_write_index: Some(0),
                    end_of_pass_write_index: Some(1),
                }),
            });

        bench_pass.set_pipeline(&pipeline.pipeline);
        bench_pass.set_bind_group(0, &pipeline.bind_group, &[]);

        for _ in 0..self.count {
            bench_pass.dispatch_workgroups(
                pipeline.workgroups_dispatch.0,
                pipeline.workgroups_dispatch.1,
                pipeline.workgroups_dispatch.2,
            )
        }

        drop(bench_pass); // has to be dropped before encoding more commands

        if let Some(callback) = self.finalize_encoder_callback {
            callback(&mut encoder)
        }

        encoder.finish()
    }

    /// Pass for resolving the timestamp query compute pass
    fn timestamp_pass(
        &self,
        pipeline: &BenchmarkComputePipeline,
        timestamp_query: &TimestampQuery,
    ) -> CommandBuffer {
        let TimestampQuery {
            query_set,
            query_buf,
            query_staging_buf,
        } = timestamp_query;

        let mut encoder = pipeline
            .gpu
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });

        encoder.resolve_query_set(
            &query_set,
            0..(TimestampQuery::COUNT as u32),
            &query_buf,
            0,
        );
        encoder.copy_buffer_to_buffer(
            &query_buf,
            0,
            &query_staging_buf,
            0,
            (TimestampQuery::COUNT * size_of::<u64>()) as u64,
        );

        encoder.finish()
    }
}

/// Utility struct for passing around the query set and buffers needed to add
/// timestamp queries to the [Benchmark] compute passes.
struct TimestampQuery {
    /// The timestamp query set
    query_set: QuerySet,
    /// Buffer where the timestamp query gets resolved.
    /// Flags: COPY_SRC | QUERY_RESOLVE,
    query_buf: wgpu::Buffer,
    /// Mappable buffer to read the query results.
    /// Flags: COPY_SRC | QUERY_RESOLVE,
    query_staging_buf: AsyncBuffer,
}

impl TimestampQuery {
    const COUNT: usize = 2;

    fn new(device: &AsyncDevice) -> Self {
        let query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("Timestamp Query Set"),
            count: Self::COUNT as u32,
            ty: QueryType::Timestamp,
        });

        let query_buf = (**device).create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (Self::COUNT * size_of::<u64>()) as u64,
            usage: wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::QUERY_RESOLVE,
            mapped_at_creation: false,
        });
        let query_staging_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (Self::COUNT * size_of::<u64>()) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            query_set,
            query_buf,
            query_staging_buf,
        }
    }

    async fn get_timestamp_result<'a>(
        &'a self,
    ) -> Result<[u64; Self::COUNT], MapTimestampResultError> {
        let timestamp_query_slice = self.query_staging_buf.slice(..);

        timestamp_query_slice
            .map_async(MapMode::Read)
            .await
            .map_err(|_| MapTimestampResultError)?;

        let ts_data: [u64; Self::COUNT] = {
            let ts_data_raw: &[u8] = &*timestamp_query_slice.get_mapped_range();
            bytemuck::cast_slice(&ts_data_raw)
                .to_vec()
                .try_into()
                .unwrap()
        };

        self.query_staging_buf.unmap();

        Ok(ts_data)
    }
}

/// There was an error mapping the results of the timestamp query buffer, which
/// is needed in order to get the benchmark's timing information.
#[derive(Debug, Clone)]
pub struct MapTimestampResultError;

#[derive(Clone, Copy, Debug)]
/// Used for [BenchmarkResults] methods to indicate which unit to get the
/// results in.
pub enum TimeUnit {
    /// 1s
    Second,

    /// 1ms
    Milli,

    /// 1µs
    Micro,

    /// 1ns
    Nano,
}

/// Converts a nanoseconds unit to the given [TimeUnit]
fn nano_to_unit(nanoseconds: f64, unit: TimeUnit) -> f64 {
    match unit {
        TimeUnit::Second => nanoseconds / 1_000_000_000.0,
        TimeUnit::Milli => nanoseconds / 1_000_000.0,
        TimeUnit::Micro => nanoseconds / 1000.0,
        TimeUnit::Nano => nanoseconds,
    }
}
