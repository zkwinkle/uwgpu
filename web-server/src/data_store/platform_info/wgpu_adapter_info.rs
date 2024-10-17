/// Wrapper for storing [AdapterInfo](uwgpu::wgpu::AdapterInfo) info.
pub struct DataStoreWgpuAdapterInfo {
    /// Adapter name
    ///
    /// As of October 2024, after testing this on a web target, it appears this
    /// is always an empty string ("") in the browser.
    ///
    /// This should be stored in a separate table to avoid doubly storing the
    /// same value (multiple people's computers might return the same name).
    pub name: String,
    /// Backend-specific vendor ID of the adapter.
    ///
    /// As of October 2024, after testing this on a web target, it appears this
    /// is always 0 in the browser.
    pub vendor: u32,
    /// Backend-specific device ID of the adapter
    ///
    /// As of October 2024, after testing this on a web target, it appears this
    /// is always 0 in the browser.
    pub device: u32,
    /// Type of device
    ///
    /// As of October 2024, after testing this on a web target, it appears this
    /// is always 0 in the browser.
    pub device_type: DataStoreWgpuDeviceType,
    /// Driver name
    ///
    /// as of october 2024, after testing this on a web target, it appears this
    /// is always an empty string ("") in the browser.
    ///
    /// This should be stored in a separate table to avoid doubly storing the
    /// same value (multiple people's computers might return the same name).
    pub driver: String,
    /// Driver info
    ///
    /// As of October 2024, after testing this on a web target, it appears this
    /// is always an empty string ("") in the browser.
    ///
    /// This should be stored in a separate table to avoid doubly storing the
    /// same value (multiple people's computers might return the same name).
    pub driver_info: String,
    /// Backend used for device
    pub backend: DataStoreWgpuBackend,
}

/// Datastore wrapper for [Backend](uwgpu::wgpu::Backend) enum.
pub enum DataStoreWgpuBackend {
    /// Vulkan API (Windows, Linux, Android, MacOS via
    /// `vulkan-portability`/MoltenVK)
    Vulkan,
    /// Metal API (Apple platforms)
    Metal,
    /// Direct3D-12 (Windows)
    Dx12,
    /// OpenGL 3.3+ (Windows), OpenGL ES 3.0+ (Linux, Android, MacOS via
    /// Angle), and WebGL2
    Gl,
    /// WebGPU in the browser
    BrowserWebGpu,
}

/// Datastore wrapper for [DeviceType](uwgpu::wgpu::DeviceType) enum.
pub enum DataStoreWgpuDeviceType {
    /// Other or Unknown.
    Unknown,
    /// Integrated GPU with shared CPU/GPU memory.
    IntegratedGpu,
    /// Discrete GPU with separate CPU/GPU memory.
    DiscreteGpu,
    /// Virtual / Hosted.
    VirtualGpu,
    /// Cpu / Software Rendering.
    Cpu,
}
