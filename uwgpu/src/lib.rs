#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

use std::collections::HashMap;

use wgpu::{
    Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource,
    CommandEncoder, CommandEncoderDescriptor, CompilationInfo,
    ComputePipelineDescriptor, Device, DeviceDescriptor, DeviceLostReason,
    Features, Instance, InstanceDescriptor, Limits, MemoryHints,
    PowerPreference, QuerySet, QueryType, Queue, RequestAdapterOptions,
    RequestDeviceError, ShaderModule, ShaderModuleDescriptor,
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
}

/// Represents a compute pipeline that can be used to execute one benchmark by
/// passing it to [Benchmark::run].
pub struct ComputePipeline<'a> {
    gpu: &'a GPUContext,
    shader_module: ShaderModule,
    bind_group: BindGroup,
    encoder: CommandEncoder,
    query_set: QuerySet,
}

impl<'a> ComputePipeline<'a> {
    /// If the shader compilation fails this function will error. If it doesn't
    /// fail we still recommend checking
    /// [get_shader_compilation_info](Self::get_shader_compilation_info) for any
    /// warnings.
    pub fn new(
        params: PipelineParameters<'a>,
    ) -> Result<Self, CreatePipelineError> {
        let shader_module =
            params.gpu.device.create_shader_module(params.shader);

        // TODO: Inspect messages to find an Error

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

        let encoder = params
            .gpu
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });

        let query_set =
            params
                .gpu
                .device
                .create_query_set(&wgpu::QuerySetDescriptor {
                    label: Some("Timestamp Query Set"),
                    count: 2,
                    ty: QueryType::Timestamp,
                });

        Ok(Self {
            gpu: params.gpu,
            shader_module,
            bind_group,
            encoder,
            query_set,
        })
    }

    /// Get the compilation messages from compiling the shader module
    pub async fn get_shader_compilation_info(&self) -> CompilationInfo {
        self.shader_module.get_compilation_info().await
    }
}

/// This type represents the parameters for running a benchmark.
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
    pub finalize_encoder_callback: Option<&'a dyn Fn(&CommandEncoder)>,
    // Porbably need a workgroups param?
}

/// Results from executing a benchmark with [Benchmark::run].
///
/// All timing quantities are given in nanoseconds
pub struct BenchmarkResults {
    /// Total iterations ran, this should be equal to the `count` field in the
    /// [Benchmark] that was ran, but its also provided here for
    /// convenience.
    pub count: u64,
    /// Total time spent executing the
    pub total_time_spent: u64,

    /// An "overhead benchmark" is carried out by executing a compute pass with
    /// all the same commands except for running the shader. The time spent
    /// in a single execution of this is reported here as "overhead time".
    pub overhead_time_spent: u64,
}
// TODO: Impl functions to get total_time without overhead, time per iteration
// without overhead

impl Benchmark<'_> {
    /// Runs the benchmark using the provided compute pipeline
    pub async fn run<'a>(
        &self,
        _pipeline: ComputePipeline<'a>,
    ) -> BenchmarkResults {
        // 1. Create command buffer and run and get times
        //
        // 2. I also need to have a query resolve buffer and a results buffer in
        //    order to map the timestamp results back to CPU land
        //
        // 3. Can then run another pass that does the same thing but without
        //    running the actual shader to substract the "overhead"

        todo!()
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
    ShaderError,
    // TODO: Include the compilation info messages
    // TODO: We don't return this yet, need to check the compilation info
    // messages for any errors.
}
