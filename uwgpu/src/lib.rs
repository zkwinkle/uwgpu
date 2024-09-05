#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

use std::collections::HashMap;

use wgpu::{
    Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource,
    CommandBuffer, CommandEncoderDescriptor, CompilationInfo,
    CompilationMessageType, ComputePass, ComputePassDescriptor,
    ComputePassTimestampWrites, ComputePipelineDescriptor, Device,
    DeviceDescriptor, DeviceLostReason, Features, Instance, InstanceDescriptor,
    Limits, MapMode, MemoryHints, PowerPreference, QuerySet, QueryType, Queue,
    RequestAdapterOptions, RequestDeviceError, ShaderModule,
    ShaderModuleDescriptor,
};

#[cfg(target_arch = "wasm32")]
mod wasm_utils;

/// Represents a handle on a GPU device
pub struct GPUContext {
    device: Device,
    queue: Queue,
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
pub struct ComputePipeline<'a> {
    gpu: &'a GPUContext,
    shader_module: ShaderModule,
    bind_group: BindGroup,
    pipeline: wgpu::ComputePipeline,
    workgroups: (u32, u32, u32),
}

impl<'a> ComputePipeline<'a> {
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
            return Err(CreatePipelineError::ShaderCompilationError);
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
/// The benchmark will run 3 compute passes in the following order:
///
/// 1. If `warmup_count` is >0, it will run a compute pass executing the shader
///    `warmup_count` times.
///
/// 2. After that it'll run the actual benchmark compute pass which will be
///    timed. The time taken will be returned in the [BenchmarkResults]
///    `total_time_spent` field.
///
/// 3. Lastly it'll run an extra compute pass to calculate "overhead" time. To
///    do this, it'll execute a pass that is the same as the previous one if
///    `count` were equal to `0`. In order words it just doesn't execute the
///    shader. The time taken will be returned in the [BenchmarkResults]
///    `overhead_time_spent` field.
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
    /// This will get called after all the iterations of benchmark execution
    /// have been.
    ///
    /// A common usage of this callback is copying the output from the shader
    /// to a buffer that has the [MAP_READ](wgpu::BufferUsages::MAP_READ)
    /// usage flag set.
    ///
    /// The callback will be called at the end of the 3 compute passes
    /// described in the [Benchmark] docs.
    pub finalize_encoder_callback: Option<&'a dyn Fn(&mut ComputePass)>,
}

/// Results from executing a benchmark with [Benchmark::run].
///
/// All timing quantities are given in nanoseconds
pub struct BenchmarkResults {
    /// Total iterations ran, this should be equal to the `count` field in the
    /// [Benchmark] that was ran, but its also provided here for
    /// convenience.
    pub count: usize,

    /// Total time spent executing the
    pub total_time_spent: f64,

    /// An "overhead benchmark" is carried out by executing a compute pass with
    /// all the same commands except for running the shader. The time spent
    /// in a single execution of this is reported here as "overhead time".
    pub overhead_time_spent: f64,
}
// TODO: Impl functions to get total_time without overhead, time per iteration
// without overhead

impl Benchmark<'_> {
    /// Runs the benchmark using the provided compute pipeline
    pub async fn run<'a>(
        &self,
        pipeline: ComputePipeline<'a>,
    ) -> BenchmarkResults {
        // Timestamp query set and buffers
        // ----------------------------------
        let query_set =
            pipeline
                .gpu
                .device
                .create_query_set(&wgpu::QuerySetDescriptor {
                    label: Some("Timestamp Query Set"),
                    count: 4,
                    ty: QueryType::Timestamp,
                });

        let query_buf =
            pipeline.gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: 32, // count * 2
                usage: wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::QUERY_RESOLVE,
                mapped_at_creation: false,
            });
        let query_staging_buf =
            pipeline.gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                timestamp_writes: None,
            });

        bench_pass.set_pipeline(&pipeline.pipeline);
        bench_pass.set_bind_group(0, &pipeline.bind_group, &[]);

        for _ in 0..self.warmup_count {
            bench_pass.dispatch_workgroups(
                pipeline.workgroups.0,
                pipeline.workgroups.1,
                pipeline.workgroups.2,
            )
        }

        if let Some(callback) = self.finalize_encoder_callback {
            callback(&mut bench_pass)
        }

        encoder.finish()
    }

    /// Benchmark compute pass
    fn benchmark_pass(
        &self,
        pipeline: &ComputePipeline,
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

        if let Some(callback) = self.finalize_encoder_callback {
            callback(&mut bench_pass)
        }
        encoder.resolve_query_set(&query_set, 0..2, &query_buf, 0);
        encoder.copy_buffer_to_buffer(&query_buf, 0, &query_staging_buf, 0, 16);

        encoder.finish()
    }

    /// Overhead compute pass
    fn overhead_pass(
        &self,
        pipeline: &ComputePipeline,
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

        let mut overhead_pass =
            encoder.begin_compute_pass(&ComputePassDescriptor {
                label: None,
                timestamp_writes: Some(ComputePassTimestampWrites {
                    query_set: &query_set,
                    beginning_of_pass_write_index: Some(2),
                    end_of_pass_write_index: Some(3),
                }),
            });

        overhead_pass.set_pipeline(&pipeline.pipeline);
        overhead_pass.set_bind_group(0, &pipeline.bind_group, &[]);

        if let Some(callback) = self.finalize_encoder_callback {
            callback(&mut overhead_pass)
        }
        encoder.resolve_query_set(&query_set, 2..4, &query_buf, 2);
        encoder.copy_buffer_to_buffer(&query_buf, 2, &query_staging_buf, 2, 16);

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
    query_staging_buf: wgpu::Buffer,
}

impl TimestampQuery {
    fn new(device: &Device) -> Self {
        let query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("Timestamp Query Set"),
            count: 4,
            ty: QueryType::Timestamp,
        });

        let query_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 32, // count * 2
            usage: wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::QUERY_RESOLVE,
            mapped_at_creation: false,
        });
        let query_staging_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 32, // same as query_buf
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            query_set,
            query_buf,
            query_staging_buf,
        }
    }
}

/// An error when trying to get a GPU context with [GPUContext::new]
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
pub enum CreatePipelineError {
    /// Error compiling the shader
    ShaderCompilationError,
    // TODO: Include the compilation info messages
    // TODO: We don't return this yet, need to check the compilation info
    // messages for any errors.
}
