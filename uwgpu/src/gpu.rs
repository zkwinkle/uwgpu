//! Manipulating the GPU, see [GPUContext]

use std::sync::Arc;

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt, TextureDataOrder},
    Backends, BufferDescriptor, DeviceDescriptor, DeviceLostReason, Features,
    Instance, InstanceDescriptor, Limits, MemoryHints, PowerPreference,
    RequestAdapterOptions, RequestDeviceError, Texture, TextureDescriptor,
};
use wgpu_async::{AsyncBuffer, AsyncDevice, AsyncQueue};

/// Represents a handle on a GPU device
pub struct GPUContext {
    pub(crate) device: AsyncDevice,
    pub(crate) queue: AsyncQueue,
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
