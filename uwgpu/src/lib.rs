#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

// Re-export so that users of this library can use wgpu types.
// Upgrading wgpu major version means a semver breaknig change for this library
// as well. Could use my own type wrappers to avoid that. Idea to dwell on...
pub use wgpu;
pub use wgpu_async;

use std::{collections::HashMap, mem::size_of, sync::Arc};

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt, TextureDataOrder},
    Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource,
    BufferDescriptor, CommandBuffer, CommandEncoder, CommandEncoderDescriptor,
    CompilationInfo, CompilationMessage, CompilationMessageType,
    ComputePassDescriptor, ComputePassTimestampWrites,
    ComputePipelineDescriptor, DeviceDescriptor, DeviceLostReason, Features,
    Instance, InstanceDescriptor, Limits, MapMode, MemoryHints,
    PowerPreference, QuerySet, QueryType, RequestAdapterOptions,
    RequestDeviceError, ShaderModule, ShaderModuleDescriptor, Texture,
    TextureDescriptor,
};
use wgpu_async::{AsyncBuffer, AsyncDevice, AsyncQueue};

#[cfg(target_arch = "wasm32")]
mod wasm_utils;

/// Represents a handle on a GPU device
pub struct GPUContext {
    device: AsyncDevice,
    queue: AsyncQueue,
}

impl GPUContext {
    /// Instantiate a new [GPUContext]
    pub async fn new(
        required_features: Option<Features>,
    ) -> Result<Self, GetGPUContextError> {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        });

        let Some(adapter) = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
        else {
            return Err(GetGPUContextError::NoAdapter);
        };

        let features = adapter.features();

        if !(features.intersects(Features::TIMESTAMP_QUERY)) {
            return Err(GetGPUContextError::DoesNotSupportTimestamps);
        }

        if let Some(required_feaures) = required_features {
            if !(features.contains(required_feaures)) {
                return Err(
                    GetGPUContextError::DoesNotSupportRequestedFeatures,
                );
            }
        }

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    required_features: Features::TIMESTAMP_QUERY
                        | required_features.unwrap_or(Features::empty()),
                    required_limits: Limits::default(),
                    memory_hints: MemoryHints::Performance,
                },
                Default::default(),
            )
            .await
            .map_err(|err| GetGPUContextError::RequestDevice(err))?;

        let (device, queue) =
            wgpu_async::wrap(Arc::new(device), Arc::new(queue));

        Ok(GPUContext { device, queue })
    }

    /// Set the [device lost
    /// callback](https://developer.mozilla.org/en-US/docs/Web/API/GPUDevice/lost)
    ///
    /// If the callback gets triggered, then this [`BenchmarkPipeline`] will no
    /// longer be valid, request a new one with [`BenchmarkPipeline::new()`]
    pub fn set_device_lost_callback(
        &self,
        callback: impl Fn(DeviceLostReason, String) + Send + 'static,
    ) {
        self.device.set_device_lost_callback(callback)
    }

    /// Creates a [Buffer], this is just a wrapper for
    /// [AsyncDevice::create_buffer]
    pub fn create_buffer(&self, desc: &BufferDescriptor) -> AsyncBuffer {
        self.device.create_buffer(desc)
    }

    /// Creates a [Buffer], this is just a wrapper for
    /// [AsyncDevice::create_buffer_init]
    pub fn create_buffer_init(
        &self,
        desc: &BufferInitDescriptor,
    ) -> AsyncBuffer {
        self.device.create_buffer_init(desc)
    }

    /// Creates a [Texture], this is just a wrapper for [Device::create_texture]
    pub fn create_texture(&self, desc: &TextureDescriptor) -> Texture {
        self.device.create_texture(desc)
    }

    /// Creates a [Texture], this is just a wrapper for
    /// [DeviceExt::create_texture_with_data]
    pub fn create_texture_with_data(
        &self,
        desc: &TextureDescriptor,
        order: TextureDataOrder,
        data: &[u8],
    ) -> Texture {
        self.device
            .create_texture_with_data(&self.queue, desc, order, data)
    }
}

/// This type can be used to create a [ComputePipeline] by calling
/// [ComputePipeline::new()].
#[derive(Clone)]
pub struct PipelineParameters<'a> {
    /// Compute shader to execute
    pub shader: ShaderModuleDescriptor<'a>,

    /// Entry point of the compute shader.
    /// Must be the name of a shader function annotated with `@compute` and no
    /// return value.
    pub entry_point: &'a str,

    /// This bind group must specify all the bindings used in the shader.
    /// The key used in the HashMap is the `n` index value of the corresponding
    /// `@binding(n)` attribute in the shader.
    ///
    /// This BindGroup will be assigned to `@group(0)` in the shader,
    /// the shader should only use that group.
    ///
    /// Note: All the executions of the benchmark will reuse this same bind
    /// group, so for example if the shader uses the same buffer for input
    /// and output (by overriding it), it will keep overriding the same
    /// buffer over and over, effectively using last iteration's output as its
    /// next iteration's input.
    pub bind_group_0: HashMap<u32, BindingResource<'a>>,

    /// GPU context that is to be used for creating this pipeline.
    pub gpu: &'a GPUContext,

    /// The amount of workgroups to dispatch, the tuple represents the `(x, y,
    /// z)` dimensions of the grid of workgroups.
    pub workgroups: (u32, u32, u32),
}

/// Represents a compute pipeline that can be used to execute one benchmark by
/// passing it to [Benchmark::run].
pub struct BenchmarkComputePipeline<'a> {
    gpu: &'a GPUContext,
    shader_module: ShaderModule,
    bind_group: BindGroup,
    pipeline: wgpu::ComputePipeline,
    workgroups: (u32, u32, u32),
}

impl<'a> BenchmarkComputePipeline<'a> {
    /// If the shader compilation fails this function will error. If it doesn't
    /// fail we still recommend checking
    /// [get_shader_compilation_info](Self::get_shader_compilation_info) for any
    /// warnings.
    pub async fn new(
        params: PipelineParameters<'a>,
    ) -> Result<Self, CreatePipelineError> {
        let shader_module =
            params.gpu.device.create_shader_module(params.shader);

        let compilation_info = shader_module.get_compilation_info().await;

        if compilation_info
            .messages
            .iter()
            .any(|msg| msg.message_type == CompilationMessageType::Error)
        {
            return Err(CreatePipelineError::ShaderCompilationError(
                compilation_info.messages,
            ));
        }

        let pipeline = params.gpu.device.create_compute_pipeline(
            &ComputePipelineDescriptor {
                label: None,
                layout: None,
                module: &shader_module,
                entry_point: params.entry_point,
                compilation_options: Default::default(),
                cache: None,
            },
        );

        let bind_group =
            params.gpu.device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &pipeline.get_bind_group_layout(0),
                entries: &params
                    .bind_group_0
                    .into_iter()
                    .map(|(id, resource)| BindGroupEntry {
                        binding: id,
                        resource,
                    })
                    .collect::<Vec<BindGroupEntry>>(),
            });

        Ok(Self {
            gpu: params.gpu,
            shader_module,
            bind_group,
            pipeline,
            workgroups: params.workgroups,
        })
    }

    /// Get the compilation messages from compiling the shader module
    pub async fn get_shader_compilation_info(&self) -> CompilationInfo {
        self.shader_module.get_compilation_info().await
    }
}

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
    pub warmup_count: usize,

    /// The number of iterations of the benchmark to execute.
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
                pipeline.workgroups.0,
                pipeline.workgroups.1,
                pipeline.workgroups.2,
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
                pipeline.workgroups.0,
                pipeline.workgroups.1,
                pipeline.workgroups.2,
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

/// An error when trying to get a GPU context with [GPUContext::new]
#[derive(Debug, Clone)]
pub enum GetGPUContextError {
    /// Failed to get an adapter, a possible reason could be because no backend
    /// was available.
    ///
    /// For example, if this is running in a browser that doesn't support
    /// WebGPU.
    NoAdapter,

    /// Failed to request the device, see [`RequestDeviceError`]
    RequestDevice(RequestDeviceError),

    /// The adapter doesn't support timestamp queries.
    ///
    /// This feature is needed to time the microbenchmarks accurately,
    /// therefore if the feature is not available we treat it as an error.
    DoesNotSupportTimestamps,

    /// The adapter doesn't support one of the features requested in the
    /// parameter to [GPUContext::new].
    DoesNotSupportRequestedFeatures,
}

/// Error creating a [ComputePipeline]
#[derive(Debug)]
pub enum CreatePipelineError {
    /// Error compiling the shader
    ShaderCompilationError(Vec<CompilationMessage>),
    // TODO: Include the compilation info messages
    // TODO: We don't return this yet, need to check the compilation info
    // messages for any errors.
}

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
