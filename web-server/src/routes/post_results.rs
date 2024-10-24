use uwgpu::AdapterInfo;

use crate::components::benchmark_page::MicrobenchmarkPage;

pub struct PostResultsRequest {
    platform_info: PlatformInfo,
    count: u32,
    total_time_spent: f64,
    workgroup_size: Box<[u32]>,
    benchmark_page: MicrobenchmarkPage,
}

pub struct PlatformInfo {
    wgpu_adapter_info: AdapterInfo,
    webgpu_adapter_info: WebGpuAdapterInfo,
}

pub struct WebGpuAdapterInfo {
    pub architecture: String,
    pub description: String,
    pub device: String,
    pub vendor: String,
}
