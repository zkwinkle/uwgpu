#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![feature(cfg_eval)]

// Re-export so that users of this library can use wgpu types.
// Upgrading wgpu major version means a semver breaknig change for this library
// as well. Could use my own type wrappers to avoid that. Idea to dwell on...
pub use wgpu;
pub use wgpu_async;

use std::{mem::size_of, ops::Deref};

use thiserror::Error;
use wgpu::{
    CommandBuffer, CommandEncoder, CommandEncoderDescriptor,
    ComputePassDescriptor, ComputePassTimestampWrites, MapMode, QuerySet,
    QueryType, Queue,
};
use wgpu_async::{AsyncBuffer, AsyncDevice};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
mod wasm_print {
    /// Shadow println! when compiling to WASM
    #[macro_export]
    macro_rules! println {
        ($($t:tt)*) => (web_sys::console::log_1(&format_args!($($t)*).to_string().into()))
    }

    /// Shadow eprintln! when compiling to WASM
    #[macro_export]
    macro_rules! eprintln {
        ($($t:tt)*) => (web_sys::console::error_1(&format_args!($($t)*).to_string().into()))
    }
}

mod adapter_info;
mod gpu;
mod pipeline;

pub use adapter_info::{AdapterInfo, Backend, DeviceType};
pub use gpu::*;
pub use pipeline::*;

/// According to https://www.w3.org/TR/webgpu/#timestamp, timestamp queries can
/// return negative time deltas in rare circumstances. To mitigate this effect,
/// we run many short compute passes to get many timestamp measurements and
/// ignore any invalid ones. This constant is the max amount of shader
/// invocations we'll do per compute pass.
const MAX_COUNT_BETWEEN_QUERIES: usize = 50;

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
#[cfg_eval]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct BenchmarkResults {
    /// Total iterations that counted towards the result, this can differ from
    /// the `count` field in the [Benchmark] that was ran due to the fact that
    /// some of the timestamps that were measured in the benchmark returned
    /// invalid values (negative deltas).
    ///
    /// Reference: https://www.w3.org/TR/webgpu/#timestamp
    pub count: usize,

    /// Total time spent executing the benchmark.
    pub total_time_spent: f64,

    /// Information about the adapter used in the benchmark.
    #[cfg_attr(feature = "wasm", wasm_bindgen(getter_with_clone))]
    pub adapter_info: AdapterInfo,
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
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
        let timestamp_query =
            TimestampQuery::new(&pipeline.gpu.device, self.count);

        let warmup_command_buf = self.warmup_pass(&pipeline);
        let benchmark_command_buf =
            self.benchmark_passes(&pipeline, &timestamp_query);
        let resolve_timestamp_pass =
            self.timestamp_pass(&pipeline, &timestamp_query);

        // Chromium will panick if we don't deref to the regular Queue
        // for some reason.
        let queue: &Queue = pipeline.gpu.queue.deref();
        queue.submit([
            warmup_command_buf,
            benchmark_command_buf,
            resolve_timestamp_pass,
        ]);

        let ts_data = timestamp_query.get_timestamp_result().await?;
        let ts_period = pipeline.gpu.queue.get_timestamp_period() as f64;

        // println!("ts_data: {:?}", ts_data);

        // Accumulator inside fold is (total_time, real_count)
        let (total_time_spent, real_count) = ts_data
            .chunks(2)
            .enumerate()
            .fold((0, 0), |(total_time, real_count), (i, times)| {
                let start_time = times[0];
                let end_time = times[1];
                let (time, is_invalid) = end_time.overflowing_sub(start_time);

                if is_invalid {
                    // println!(
                    //     "overflow! start: {}, end: {}",
                    //     start_time, end_time
                    // );
                    (total_time, real_count)
                } else {
                    // println!(
                    //     "GOOD!  time: {}\t total_time: {}",
                    //     time,
                    //     total_time + time
                    // );
                    // println!(" start: {}, end: {}", start_time, end_time);
                    let is_last_result = ((i + 1) * 2) == ts_data.len();
                    let additional_count = if is_last_result {
                        let residue = self.count % MAX_COUNT_BETWEEN_QUERIES;
                        if residue == 0 {
                            MAX_COUNT_BETWEEN_QUERIES
                        } else {
                            residue
                        }
                    } else {
                        MAX_COUNT_BETWEEN_QUERIES
                    };
                    (total_time + time, real_count + additional_count)
                }
            });

        // println!(
        //     "Total time: {}\t Real count: {}\nTotal time as f64:
        // {}\nts_period: {}",     total_time_spent, real_count,
        // total_time_spent as f64, ts_period );
        let total_time_spent = (total_time_spent as f64) * ts_period;

        Ok(BenchmarkResults {
            count: real_count,
            total_time_spent,
            adapter_info: pipeline.gpu.adapter_info.clone().into(),
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
            for (i, dispatch) in pipeline.workgroups_dispatch.iter().enumerate()
            {
                warmup_pass
                    .dispatch_workgroups(dispatch.0, dispatch.1, dispatch.2);

                if let Some(callback) = pipeline.dispatch_callback {
                    callback(i, &mut warmup_pass);
                }
            }
        }

        drop(warmup_pass); // has to be dropped before finishing commands

        encoder.finish()
    }

    /// Benchmark compute passes + end callback commands
    fn benchmark_passes(
        &self,
        pipeline: &BenchmarkComputePipeline,
        timestamp_query: &TimestampQuery,
    ) -> CommandBuffer {
        let passes_num = self.count / MAX_COUNT_BETWEEN_QUERIES;

        let mut encoder = pipeline
            .gpu
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });

        for pass in 0..=passes_num {
            let (query_set, begin_index, end_index) =
                timestamp_query.get_query_set(pass);

            let amount = {
                if pass < passes_num {
                    MAX_COUNT_BETWEEN_QUERIES
                } else {
                    self.count % MAX_COUNT_BETWEEN_QUERIES
                }
            };

            // Can happen if the count == MAX_COUNT_BETWEEN_QUERIES
            if amount == 0 {
                break;
            }

            let mut bench_pass =
                encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: None,
                    timestamp_writes: Some(ComputePassTimestampWrites {
                        query_set: &query_set,
                        beginning_of_pass_write_index: Some(begin_index),
                        end_of_pass_write_index: Some(end_index),
                    }),
                });

            bench_pass.set_pipeline(&pipeline.pipeline);
            bench_pass.set_bind_group(0, &pipeline.bind_group, &[]);

            // println!("Adding pass i={} with {} runs", pass, amount);

            for _ in 0..amount {
                for (i, dispatch) in
                    pipeline.workgroups_dispatch.iter().enumerate()
                {
                    bench_pass.dispatch_workgroups(
                        dispatch.0, dispatch.1, dispatch.2,
                    );

                    if let Some(callback) = pipeline.dispatch_callback {
                        callback(i, &mut bench_pass);
                    }
                }
            }

            // Already gets dropped but leaving here as a reminder in case I
            // change stuff around
            // drop(bench_pass) // must be dropped before encoding more commands
        }

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
            query_sets,
            query_buf,
            query_staging_buf,
        } = timestamp_query;

        let mut encoder = pipeline
            .gpu
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });

        let num_runs = (self.count + MAX_COUNT_BETWEEN_QUERIES - 1)
            / MAX_COUNT_BETWEEN_QUERIES;
        for (i, query_set) in query_sets.iter().enumerate() {
            let offset = (i * (wgpu::QUERY_SET_MAX_QUERIES as usize)) as u64;
            let amount = {
                if i < (query_sets.len() - 1) {
                    wgpu::QUERY_SET_MAX_QUERIES as usize
                } else {
                    (num_runs * 2) % (wgpu::QUERY_SET_MAX_QUERIES as usize)
                }
            };

            // println!(
            //     "Resolving query set i={} with offset {} and amount {}",
            //     i, offset, amount
            // );

            encoder.resolve_query_set(
                &query_set,
                0..(amount as u32),
                &query_buf,
                offset,
            );
            encoder.copy_buffer_to_buffer(
                &query_buf,
                offset,
                &query_staging_buf,
                offset,
                (amount * size_of::<u64>()) as u64,
            );
        }

        encoder.finish()
    }
}

/// Utility struct for passing around the query sets and buffers needed to add
/// timestamp queries to the [Benchmark] compute passes.
struct TimestampQuery {
    /// The timestamp query sets
    query_sets: Vec<QuerySet>,
    /// Buffer where the timestamp queries gets resolved.
    /// Flags: COPY_SRC | QUERY_RESOLVE,
    query_buf: wgpu::Buffer,
    /// Mappable buffer to read the queries results.
    /// Flags: COPY_SRC | QUERY_RESOLVE,
    query_staging_buf: AsyncBuffer,
}

impl TimestampQuery {
    fn new(device: &AsyncDevice, count: usize) -> Self {
        let num_runs =
            (count + MAX_COUNT_BETWEEN_QUERIES - 1) / MAX_COUNT_BETWEEN_QUERIES;

        let number_of_full_query_sets =
            (num_runs * 2) / (wgpu::QUERY_SET_MAX_QUERIES as usize);

        let remainder =
            ((num_runs * 2) % (wgpu::QUERY_SET_MAX_QUERIES as usize)) as u32;

        let total_count = number_of_full_query_sets + remainder as usize;

        // println!(
        //         "Creating {} full query sets with maybe a remainder set with
        // {} elements", number_of_full_query_sets, remainder,     );

        let mut query_sets: Vec<QuerySet> = (0..number_of_full_query_sets)
            .map(|_| {
                device.create_query_set(&wgpu::QuerySetDescriptor {
                    label: Some("Timestamp Query Set"),
                    count: wgpu::QUERY_SET_MAX_QUERIES,
                    ty: QueryType::Timestamp,
                })
            })
            .collect();

        if remainder > 0 {
            query_sets.push(device.create_query_set(
                &wgpu::QuerySetDescriptor {
                    label: Some("Timestamp Query Set"),
                    count: remainder,
                    ty: QueryType::Timestamp,
                },
            ))
        }

        let query_buf = (**device).create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (total_count * size_of::<u64>()) as u64,
            usage: wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::QUERY_RESOLVE,
            mapped_at_creation: false,
        });
        let query_staging_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (total_count * size_of::<u64>()) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            query_sets,
            query_buf,
            query_staging_buf,
        }
    }

    /// Returns the appropiate query set for the given compute pass, then the
    /// 2 indexes returned are the ones that should be passed to
    /// `beginning_of_pass_write_index` and `end_of_pass_write_index`
    /// of the [ComputePassTimestampWrites]
    fn get_query_set(&self, pass: usize) -> (&QuerySet, u32, u32) {
        let query_set_i = (pass * 2) / (wgpu::QUERY_SET_MAX_QUERIES as usize);
        let start_index =
            ((pass * 2) % (wgpu::QUERY_SET_MAX_QUERIES as usize)) as u32;
        let end_index = start_index + 1;

        (&self.query_sets[query_set_i], start_index, end_index)
    }

    async fn get_timestamp_result<'a>(
        &'a self,
    ) -> Result<Box<[u64]>, MapTimestampResultError> {
        let timestamp_query_slice = self.query_staging_buf.slice(..);

        timestamp_query_slice
            .map_async(MapMode::Read)
            .await
            .map_err(|_| MapTimestampResultError)?;

        let ts_data: Box<[u64]> = {
            let ts_data_raw: &[u8] = &*timestamp_query_slice.get_mapped_range();
            bytemuck::cast_slice(&ts_data_raw)
                .to_vec()
                .into_boxed_slice()
        };

        self.query_staging_buf.unmap();

        Ok(ts_data)
    }
}

/// There was an error mapping the results of the timestamp query buffer, which
/// is needed in order to get the benchmark's timing information.
#[derive(Error, Debug, Clone)]
#[error(
    "error mapping the results of the timestamp query in order to read them"
)]
pub struct MapTimestampResultError;

/// Used for [BenchmarkResults] methods to indicate which unit to get the
/// results in.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
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
