//! Microbenchmark for buffer to buffer copy throughput

use std::collections::HashMap;

use rand::{thread_rng, Rng};
use uwgpu::{
    wgpu::{
        util::BufferInitDescriptor, BufferDescriptor, BufferUsages,
        ShaderModuleDescriptor, ShaderSource,
    },
    wgpu_async::AsyncBuffer,
    Benchmark, BenchmarkComputePipeline, BenchmarkResults, CreatePipelineError,
    GPUContext, PipelineParameters, TimeUnit,
};

use crate::BenchmarkError;

/// 1MiB size buffer
///
/// 1 MiB = 2^20 = 1_048_576
///
/// 1MiB / 4bytes = 262_144
const BENCHMARK_BUFFER_SIZE: usize = 262_144;
const BENCHMARK_WARMUP_COUNT: usize = 500;
const BENCHMARK_ITERATIONS: usize = 100000;

/// Microbenchmark for measuring the Buffer -> Buffer memory copy BW within the
/// GPU
///
/// The workgroup size will dictate the size of workgroups used for the
/// computation.
pub async fn buffer_to_buffer_benchmark(
    workgroup_size: u32,
) -> Result<BufferToBufferResults, BenchmarkError> {
    let gpu = GPUContext::new(None).await?;
    let buffers = Buffers::new_with_random_inputs(&gpu);
    let pipeline =
        buffer_to_buffer_pipeline(&gpu, &buffers, workgroup_size).await?;

    let results = Benchmark {
        warmup_count: BENCHMARK_WARMUP_COUNT,
        count: BENCHMARK_ITERATIONS,
        finalize_encoder_callback: None,
        workgroups_dispatch: workgroups_dispatch(
            BENCHMARK_BUFFER_SIZE,
            workgroup_size,
        ),
        dispatch_callback: None,
    }
    .run(pipeline)
    .await?;

    Ok(BufferToBufferResults(results))
}

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// Results from the memcpy buffer->buffer benchmark. See
/// [buffer_to_buffer_benchmark].
///
/// Wraps a [BenchmarkResults] with some convenience methods.
#[cfg_eval]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct BufferToBufferResults(
    #[cfg_attr(feature = "wasm", wasm_bindgen(getter_with_clone))]
    pub  BenchmarkResults,
);

#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl BufferToBufferResults {
    /// Get the total amount of time in seconds spent executing the
    /// microbenchmark
    pub fn total_time_s(&self) -> f64 { self.0.total_time(TimeUnit::Second) }

    /// Get the amount of time per iteration in ms
    pub fn time_per_iteration_ms(&self) -> f64 {
        self.0.time_per_iteration(TimeUnit::Milli)
    }

    /// Get the Bandwidth of memory copy in bytes per second
    pub fn bandwidth(&self) -> f64 {
        ((BENCHMARK_BUFFER_SIZE * std::mem::size_of::<u32>() * self.0.count)
            as f64)
            / self.total_time_s()
    }
}

/// GPU buffers needed for microbenchmark
struct Buffers {
    source_buffer: AsyncBuffer,
    destination_buffer: AsyncBuffer,
}

impl Buffers {
    fn new_with_random_inputs(gpu: &GPUContext) -> Self {
        let mut source_buffer_data: Box<[u32; BENCHMARK_BUFFER_SIZE]> =
            Box::new([0_u32; 262_144]);

        let mut rng = thread_rng();

        rng.fill(source_buffer_data.as_mut_slice());

        Self::new_from_source_data(source_buffer_data.as_slice(), gpu)
    }

    fn new_from_source_data(source_data: &[u32], gpu: &GPUContext) -> Self {
        let source_buffer = gpu.create_buffer_init(&BufferInitDescriptor {
            label: Some("Source Buffer"),
            contents: bytemuck::cast_slice(source_data),
            usage: BufferUsages::STORAGE,
        });

        let destination_buffer = gpu.create_buffer(&BufferDescriptor {
            label: Some("Destination Buffer"),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            size: BENCHMARK_BUFFER_SIZE as u64 * 8,
            mapped_at_creation: false,
        });

        Self {
            source_buffer,
            destination_buffer,
        }
    }
}

/// Pipeline needed for microbenchmark.
async fn buffer_to_buffer_pipeline<'a>(
    gpu: &'a GPUContext,
    buffers: &'a Buffers,
    workgroup_size: u32,
) -> Result<BenchmarkComputePipeline<'a>, CreatePipelineError> {
    BenchmarkComputePipeline::new(PipelineParameters {
        shader: ShaderModuleDescriptor {
            label: Some("buffer to buffer copy shader"),
            source: ShaderSource::Wgsl(
                include_str!("buffer_to_buffer.wgsl").into(),
            ),
        },
        entry_point: "main",
        bind_group_0: HashMap::from([
            (0, buffers.source_buffer.as_entire_binding()),
            (1, buffers.destination_buffer.as_entire_binding()),
        ]),
        gpu: &gpu,

        workgroup_size: Some((workgroup_size, 1, 1)),
    })
    .await
}

fn workgroups_dispatch(
    buffer_size: usize,
    workgroup_size: u32,
) -> Vec<(u32, u32, u32)> {
    vec![(1 + (buffer_size / (workgroup_size as usize)) as u32, 1, 1)]
}

#[cfg(test)]
mod tests {

    use uwgpu::{
        wgpu::{BufferDescriptor, BufferUsages, MapMode},
        Benchmark, GPUContext,
    };

    use super::*;

    #[tokio::test]
    async fn verify_buffer_copy_works() {
        let gpu = GPUContext::new(None).await.unwrap();

        let mut source_buffer_data: Box<[u32; BENCHMARK_BUFFER_SIZE]> =
            Box::new([0_u32; 262_144]);
        let mut rng = thread_rng();
        rng.fill(source_buffer_data.as_mut_slice());

        let expected_result = source_buffer_data.clone();

        let buffers =
            Buffers::new_from_source_data(source_buffer_data.as_slice(), &gpu);

        let workgroup_size = 64;
        let pipeline =
            buffer_to_buffer_pipeline(&gpu, &buffers, workgroup_size)
                .await
                .unwrap();

        let staging_buffer = gpu.create_buffer(&BufferDescriptor {
            label: Some("Staging Buffer"),
            size: (BENCHMARK_BUFFER_SIZE * std::mem::size_of::<u32>()) as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let _results = Benchmark {
            warmup_count: 0,
            count: 1,
            finalize_encoder_callback: Some(&|encoder| {
                encoder.copy_buffer_to_buffer(
                    &buffers.destination_buffer,
                    0,
                    &staging_buffer,
                    0,
                    2u64.pow(20),
                )
            }),
            workgroups_dispatch: workgroups_dispatch(
                BENCHMARK_BUFFER_SIZE,
                workgroup_size,
            ),
            dispatch_callback: None,
        }
        .run(pipeline)
        .await
        .unwrap();

        let staging_slice = staging_buffer.slice(..);
        staging_slice.map_async(MapMode::Read).await.unwrap();

        let result_data: Vec<u32> = {
            let result_data_raw: &[u8] = &*staging_slice.get_mapped_range();
            bytemuck::cast_slice(&result_data_raw).to_vec()
        };

        staging_buffer.unmap();

        assert_eq!(&result_data, &*expected_result);
    }
}
