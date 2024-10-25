use std::{borrow::Cow, sync::Arc};

use crate::{
    data_store::{
        benchmark_results::{
            DataStoreBenchmarkKind, DataStoreComputationalBenchmark,
            DataStoreComputationalBenchmarkKind,
            DataStoreCreateBenchmarkResult, DataStoreMemoryBenchmark,
            DataStoreMemoryBenchmarkKind,
        },
        non_empty_string::NonEmptyString,
        platform::{
            user_agent_info::{
                DataStoreUserAgent, DataStoreUserAgentDevice,
                DataStoreUserAgentOs, DataStoreUserAgentStringInfo,
            },
            webgpu_adapter_info::DataStoreWebGpuAdapterInfo,
            wgpu_adapter_info::{
                DataStoreWgpuAdapterInfo, DataStoreWgpuBackend,
                DataStoreWgpuDeviceType,
            },
            DataStoreCreatePlatform,
        },
        DataStore,
    },
    error::ServerError,
};
use axum::{
    http::{header::USER_AGENT, HeaderMap},
    Extension, Json,
};
use serde::Deserialize;
use uwgpu::{AdapterInfo, Backend, DeviceType};

use crate::components::benchmark_page::MicrobenchmarkKind;

#[derive(Deserialize, Debug)]
pub struct PostResultsRequest {
    platform_info: PlatformInfo,
    count: u32,
    total_time_spent: f64,
    custom_result: f64,
    workgroup_size: Box<[u32]>,
    benchmark_kind: MicrobenchmarkKind,
}

#[derive(Deserialize, Debug)]
pub struct PlatformInfo {
    wgpu_adapter_info: AdapterInfo,
    webgpu_adapter_info: Option<WebGpuAdapterInfo>,
}

#[derive(Deserialize, Debug)]
pub struct WebGpuAdapterInfo {
    pub architecture: String,
    pub description: String,
    pub device: String,
    pub vendor: String,
}

#[cfg_attr(feature = "debug", axum::debug_handler)]
pub async fn post_results(
    Extension(ua_parser): Extension<Arc<ua_parser::Extractor<'static>>>,
    Extension(data_store): Extension<Arc<dyn DataStore>>,
    headers: HeaderMap,
    Json(results): Json<PostResultsRequest>,
) -> Result<(), ServerError> {
    let user_agent_header: Option<&str> = headers
        .get(USER_AGENT)
        .and_then(|header| header.to_str().ok());

    let user_agent_data: Option<DataStoreUserAgentStringInfo> =
        user_agent_header
            .map(|header| user_agent_to_data_store(&ua_parser, header));

    let webgpu_adapter_info: Option<DataStoreWebGpuAdapterInfo> = results
        .platform_info
        .webgpu_adapter_info
        .map(webgpu_adapter_info_to_data_store);

    let wgpu_adapter_info: DataStoreWgpuAdapterInfo =
        wgpu_adapter_info_to_data_store(
            results.platform_info.wgpu_adapter_info,
        );

    let create = DataStoreCreatePlatform {
        user_agent: user_agent_data,
        webgpu_adapter_info,
        wgpu_adapter_info,
    };

    let platform = data_store.create_or_get_platform(create).await;

    let platform = platform?;

    let create = DataStoreCreateBenchmarkResult {
        platform_id: platform.platform_id,
        count: results.count,
        total_time_spent: results.total_time_spent,
        workgroup_size: workgroup_size_to_tuple(&results.workgroup_size),
        kind: benchmark_kind_to_data_store(
            results.benchmark_kind,
            results.custom_result,
        ),
    };

    let _results = data_store.create_benchmark_results(create).await;

    Ok(())
}

fn workgroup_size_to_tuple(size: &[u32]) -> (u32, u32, u32) {
    (
        size[0],
        size.get(1).copied().unwrap_or(1),
        size.get(2).copied().unwrap_or(1),
    )
}

fn benchmark_kind_to_data_store(
    benchmark: MicrobenchmarkKind,
    custom_result_value: f64,
) -> DataStoreBenchmarkKind {
    use MicrobenchmarkKind::*;

    match benchmark {
        Matmul | Reduction | Convolution | Scan => {
            DataStoreBenchmarkKind::Computational(
                DataStoreComputationalBenchmark {
                    kind: unwrap_computational_kind_to_data_store(benchmark),
                    flops: custom_result_value,
                },
            )
        }
        BufferSequential | BufferShuffled | BufferToTexture
        | TextureToTexture => {
            DataStoreBenchmarkKind::Memory(DataStoreMemoryBenchmark {
                kind: unwrap_memory_kind_to_data_store(benchmark),
                bandwidth: custom_result_value,
            })
        }
    }
}

fn unwrap_computational_kind_to_data_store(
    kind: MicrobenchmarkKind,
) -> DataStoreComputationalBenchmarkKind {
    use MicrobenchmarkKind::*;

    match kind {
        Matmul => DataStoreComputationalBenchmarkKind::Matmul,
        Reduction => DataStoreComputationalBenchmarkKind::Reduction,
 Convolution => DataStoreComputationalBenchmarkKind::Convolution,
 Scan => DataStoreComputationalBenchmarkKind::Scan,
        BufferSequential | BufferShuffled | BufferToTexture |
        TextureToTexture => panic!("unwrap_computational_kind_to_data_store should not be called on memory benchmark {:?}", kind),
    }
}

fn unwrap_memory_kind_to_data_store(
    kind: MicrobenchmarkKind,
) -> DataStoreMemoryBenchmarkKind {
    use MicrobenchmarkKind::*;

    match kind {
        Matmul | Reduction | Convolution | Scan => panic!("unwrap_memory_kind_to_data_store should not be called on computational benchmark {:?}", kind),
        BufferSequential => DataStoreMemoryBenchmarkKind::BufferSequential,
        BufferShuffled => DataStoreMemoryBenchmarkKind::BufferShuffled,
        BufferToTexture => DataStoreMemoryBenchmarkKind::BufferToTexture,
        TextureToTexture => DataStoreMemoryBenchmarkKind::TextureToTexture,
    }
}

fn webgpu_adapter_info_to_data_store(
    info: WebGpuAdapterInfo,
) -> DataStoreWebGpuAdapterInfo {
    DataStoreWebGpuAdapterInfo {
        architecture: NonEmptyString::new(info.architecture),
        description: NonEmptyString::new(info.description),
        device: NonEmptyString::new(info.device),
        vendor: NonEmptyString::new(info.vendor),
    }
}

fn wgpu_adapter_info_to_data_store(
    info: AdapterInfo,
) -> DataStoreWgpuAdapterInfo {
    DataStoreWgpuAdapterInfo {
        name: NonEmptyString::new(info.name),
        vendor: info.vendor,
        device: info.device,
        device_type: wgpu_device_type_to_data_store(info.device_type),
        driver: NonEmptyString::new(info.driver),
        driver_info: NonEmptyString::new(info.driver_info),
        backend: wgpu_backend_to_data_store(info.backend),
    }
}

fn wgpu_device_type_to_data_store(
    device_type: DeviceType,
) -> DataStoreWgpuDeviceType {
    match device_type {
        DeviceType::Other => DataStoreWgpuDeviceType::Unknown,
        DeviceType::IntegratedGpu => DataStoreWgpuDeviceType::IntegratedGpu,
        DeviceType::DiscreteGpu => DataStoreWgpuDeviceType::DiscreteGpu,
        DeviceType::VirtualGpu => DataStoreWgpuDeviceType::VirtualGpu,
        DeviceType::Cpu => DataStoreWgpuDeviceType::Cpu,
    }
}

fn wgpu_backend_to_data_store(backend: Backend) -> DataStoreWgpuBackend {
    match backend {
        Backend::Empty => unreachable!("We should never get this in real code since it's a dummy backend for tests."),
        Backend::Vulkan => DataStoreWgpuBackend::Vulkan,
        Backend::Metal => DataStoreWgpuBackend::Metal,
        Backend::Dx12 => DataStoreWgpuBackend::Dx12,
        Backend::Gl => DataStoreWgpuBackend::Gl,
        Backend::BrowserWebGpu => DataStoreWgpuBackend::BrowserWebGpu,
    }
}

fn user_agent_to_data_store(
    parser: &ua_parser::Extractor,
    header: &str,
) -> DataStoreUserAgentStringInfo {
    let ua = parser.extract(header);

    let user_agent = ua.0.map(|user_agent| DataStoreUserAgent {
        family: unwrap_non_empty_cow(user_agent.family),
        major: user_agent.major.and_then(maybe_non_empty_str),
        minor: user_agent.minor.and_then(maybe_non_empty_str),
        patch: user_agent.patch.and_then(maybe_non_empty_str),
        patch_minor: user_agent.patch_minor.and_then(maybe_non_empty_str),
    });

    let operating_system = ua.1.map(|os| DataStoreUserAgentOs {
        operating_system: unwrap_non_empty_cow(os.os),
        major: os.major.and_then(maybe_non_empty_cow),
        minor: os.minor.and_then(maybe_non_empty_cow),
        patch: os.patch.and_then(maybe_non_empty_cow),
        patch_minor: os.patch_minor.and_then(maybe_non_empty_cow),
    });

    let device = ua.2.map(|device| DataStoreUserAgentDevice {
        device: unwrap_non_empty_cow(device.device),
        brand: device.brand.and_then(maybe_non_empty_cow),
        model: device.model.and_then(maybe_non_empty_cow),
    });

    DataStoreUserAgentStringInfo {
        user_agent,
        operating_system,
        device,
    }
}

fn unwrap_non_empty_cow(string: Cow<'_, str>) -> NonEmptyString {
    NonEmptyString::new(string.into_owned()).unwrap()
}

fn maybe_non_empty_str(string: &str) -> Option<NonEmptyString> {
    NonEmptyString::new(string.to_owned())
}

fn maybe_non_empty_cow(string: Cow<'_, str>) -> Option<NonEmptyString> {
    NonEmptyString::new(string.into_owned())
}
