//! This module contains a copy of the [wgpu::AdapterInfo] but compatible
//! with wasm when the feature is enabled.

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// Information about an adapter. Clone of [wgpu::AdapterInfo] but with
/// wasm_bindgen on wasm feature.
#[cfg_eval]
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct AdapterInfo {
    /// Adapter name
    #[cfg_attr(feature = "wasm", wasm_bindgen(getter_with_clone))]
    pub name: String,
    /// [`Backend`]-specific vendor ID of the adapter
    pub vendor: u32,
    /// [`Backend`]-specific device ID of the adapter
    ///
    ///
    /// This generally is a 16-bit PCI device ID in the least significant bytes
    /// of this field. However, more significant bytes may be non-zero if
    /// the backend uses a different representation.
    ///
    /// * For [`Backend::Vulkan`], the [`VkPhysicalDeviceProperties::deviceID`]
    ///   is used, which is a superset of PCI IDs.
    ///
    /// [`VkPhysicalDeviceProperties::deviceID`]: https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkPhysicalDeviceProperties.html
    pub device: u32,
    /// Type of device
    pub device_type: DeviceType,
    /// Driver name
    #[cfg_attr(feature = "wasm", wasm_bindgen(getter_with_clone))]
    pub driver: String,
    /// Driver info
    #[cfg_attr(feature = "wasm", wasm_bindgen(getter_with_clone))]
    pub driver_info: String,
    /// Backend used for device
    pub backend: Backend,
}

impl From<wgpu::AdapterInfo> for AdapterInfo {
    fn from(value: wgpu::AdapterInfo) -> Self {
        Self {
            name: value.name,
            vendor: value.vendor,
            device: value.device,
            device_type: value.device_type.into(),
            driver: value.driver,
            driver_info: value.driver_info,
            backend: value.backend.into(),
        }
    }
}

/// Backends supported by wgpu.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub enum Backend {
    /// Dummy backend, used for testing.
    Empty = 0,
    /// Vulkan API (Windows, Linux, Android, MacOS via
    /// `vulkan-portability`/MoltenVK)
    Vulkan = 1,
    /// Metal API (Apple platforms)
    Metal = 2,
    /// Direct3D-12 (Windows)
    Dx12 = 3,
    /// OpenGL 3.3+ (Windows), OpenGL ES 3.0+ (Linux, Android, MacOS via
    /// Angle), and WebGL2
    Gl = 4,
    /// WebGPU in the browser
    BrowserWebGpu = 5,
}

impl From<wgpu::Backend> for Backend {
    fn from(value: wgpu::Backend) -> Self {
        match value {
            wgpu::Backend::Empty => Self::Empty,
            wgpu::Backend::Vulkan => Self::Vulkan,
            wgpu::Backend::Metal => Self::Metal,
            wgpu::Backend::Dx12 => Self::Dx12,
            wgpu::Backend::Gl => Self::Gl,
            wgpu::Backend::BrowserWebGpu => Self::BrowserWebGpu,
        }
    }
}

/// Supported physical device types.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub enum DeviceType {
    /// Other or Unknown.
    Other,
    /// Integrated GPU with shared CPU/GPU memory.
    IntegratedGpu,
    /// Discrete GPU with separate CPU/GPU memory.
    DiscreteGpu,
    /// Virtual / Hosted.
    VirtualGpu,
    /// Cpu / Software Rendering.
    Cpu,
}

impl From<wgpu::DeviceType> for DeviceType {
    fn from(value: wgpu::DeviceType) -> Self {
        match value {
            wgpu::DeviceType::Other => Self::Other,
            wgpu::DeviceType::IntegratedGpu => Self::IntegratedGpu,
            wgpu::DeviceType::DiscreteGpu => Self::DiscreteGpu,
            wgpu::DeviceType::VirtualGpu => Self::VirtualGpu,
            wgpu::DeviceType::Cpu => Self::Cpu,
        }
    }
}
