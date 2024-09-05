#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

use std::collections::HashMap;

use wgpu::{
    Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource,
    CommandEncoder, CommandEncoderDescriptor, CompilationInfo,
    ComputePipelineDescriptor, Device, DeviceLostReason, Features, Instance,
    InstanceDescriptor, Limits, MemoryHints, PowerPreference, Queue,
    RequestAdapterOptions, RequestDeviceError, ShaderModule,
    ShaderModuleDescriptor,
};

#[cfg(target_arch = "wasm32")]
mod wasm_utils;

/// Parameters when instantiating a [`BenchmarkPipeline`].
///
/// At a minimum, you must specify the compute shader to execute and its inputs
#[derive(Clone)]
pub struct BenchmarkDescriptor<'a> {
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
    pub bind_group_0: HashMap<u32, BindingResource<'a>>,
    /* /// Generate compute shader inputs
     * generate_inputs: &'a dyn Fn() -> &'a [u8], */

    // get_output: Fn(bindings) -> R

    // assert_output: R
}

/// Represents a simple compute pipeline that can execute 1 shader in various
/// ways, useful for benchmarking and testing compute shaders
pub struct BenchmarkPipeline {
    device: Device,
    queue: Queue,
    shader_module: ShaderModule,
    bind_group: BindGroup,
    encoder: CommandEncoder,
}

impl BenchmarkPipeline {
    /// Create a new [`BenchmarkPipeline`].
    ///
    /// The `shader` parameter must be the path to the file with the compute
    /// shader
    ///
    /// If the shader compilation fails this function will error. If it doesn't
    /// fail we still recommend checking `get_shader_compilation_info`
    pub async fn new<'a>(
        params: BenchmarkDescriptor<'a>,
    ) -> Result<Self, CreatePipelineError> {
        let (device, queue) = get_device()
            .await
            .map_err(|err| CreatePipelineError::GetDevice(err))?;

        let shader_module = device.create_shader_module(params.shader);

        // TODO: Inspect messages to find an Error

        let pipeline =
            device.create_compute_pipeline(&ComputePipelineDescriptor {
                label: None,
                layout: None,
                module: &shader_module,
                entry_point: params.entry_point,
                compilation_options: Default::default(),
                cache: None,
            });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
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

        let encoder = device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });

        Ok(Self {
            device,
            queue,
            shader_module,
            bind_group,
            encoder,
        })
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

    /// Get the compilation messages from compiling the shader module
    pub async fn get_shader_compilation_info(&self) -> CompilationInfo {
        self.shader_module.get_compilation_info().await
    }
}

/// Error creating a [`BenchmarkPipeline`]
pub enum CreatePipelineError {
    /// Error getting the GPU Device, see [`GetDeviceError`]
    GetDevice(GetDeviceError),
}

async fn get_device() -> Result<(Device, Queue), GetDeviceError> {
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
        return Err(GetDeviceError::NoAdapter);
    };

    let features = adapter.features();

    if !(features.intersects(Features::TIMESTAMP_QUERY)) {
        return Err(GetDeviceError::DoesNotSupportTimestamps);
    }

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: Features::empty(),
                required_limits: Limits::default(),
                memory_hints: MemoryHints::Performance,
            },
            Default::default(),
        )
        .await
        .map_err(|err| GetDeviceError::RequestDevice(err))?;

    // TODO: set_device_lost_callback() on device, or leave up to the user?

    Ok((device, queue))
}

/// An error when trying to get the GPU device
pub enum GetDeviceError {
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
}
