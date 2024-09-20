//! Microbenchmark for copying buffer to buffer copy throughput in a sequential
//! manner

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
/// GPU in a sequential manner.
///
/// The workgroup size will dictate the size of workgroups used for the
/// computation.
pub async fn buffer_sequential_benchmark(
    workgroup_size: u32,
) -> Result<BufferSequentialResults, BenchmarkError> {
    let gpu = GPUContext::new(None)
        .await
        .map_err(|e| BenchmarkError::GPUContext(e))?;
    let buffers = Buffers::new_with_random_inputs(&gpu);
    let pipeline = buffer_sequential_pipeline(&gpu, &buffers, workgroup_size)
        .await
        .map_err(|e| BenchmarkError::PipelineCreation(e))?;

    let results = Benchmark {
        warmup_count: BENCHMARK_WARMUP_COUNT,
        count: BENCHMARK_ITERATIONS,
        finalize_encoder_callback: None,
    }
    .run(pipeline)
    .await
    .map_err(|e| BenchmarkError::MapTimestamp(e))?;

    Ok(BufferSequentialResults(results))
}

/// Results from the matrix multiplication microbenchmark. See
/// [buffer_sequential_benchmark].
pub struct BufferSequentialResults(BenchmarkResults);

impl BufferSequentialResults {
    /// Get the total amount of time in seconds spent executing the
    /// microbenchmark
    pub fn total_time_s(&self) -> f64 { self.0.total_time(TimeUnit::Second) }

    /// Get the amount of time per iteration in ms
    pub fn time_per_iteration_ms(&self) -> f64 {
        self.0.time_per_iteration(TimeUnit::Milli)
    }

    /// Get the Bandwidth of memory copy in MB/s
    pub fn gb_per_s(&self) -> f64 {
        ((BENCHMARK_BUFFER_SIZE * std::mem::size_of::<u32>() * self.0.count)
            as f64)
            / (self.total_time_s() * 1_000_000_000.0)
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
async fn buffer_sequential_pipeline<'a>(
    gpu: &'a GPUContext,
    buffers: &'a Buffers,
    workgroup_size: u32,
) -> Result<BenchmarkComputePipeline<'a>, CreatePipelineError> {
    BenchmarkComputePipeline::new(PipelineParameters {
        shader: ShaderModuleDescriptor {
            label: Some("sequential buffer copy shader"),
            source: ShaderSource::Wgsl(
                include_str!("buffer_sequential.wgsl").into(),
            ),
        },
        entry_point: "main",
        bind_group_0: HashMap::from([
            (0, buffers.source_buffer.as_entire_binding()),
            (1, buffers.destination_buffer.as_entire_binding()),
        ]),
        gpu: &gpu,

        // WG size = 64
        // 1MiB u32 = 262_144 elements
        // 262_144 / 64 = 4096
        workgroups_dispatch: (
            1 + (BENCHMARK_BUFFER_SIZE / (workgroup_size as usize)) as u32,
            1,
            1,
        ),
        workgroup_size: Some((workgroup_size, 1, 1)),
    })
    .await
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

        let pipeline = buffer_sequential_pipeline(&gpu, &buffers, 64)
            .await
            .unwrap();

        let staging_buffer = gpu.create_buffer(&BufferDescriptor {
            label: Some("Staging Buffer"),
            size: (BENCHMARK_BUFFER_SIZE * std::mem::size_of::<u32>()) as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let results = Benchmark {
            warmup_count: 0,
            count: 1,
            finalize_encoder_callback: Some(&|encoder| {
                println!("This is getting called");
                encoder.copy_buffer_to_buffer(
                    &buffers.destination_buffer,
                    0,
                    &staging_buffer,
                    0,
                    2u64.pow(20),
                )
            }),
        }
        .run(pipeline)
        .await
        .unwrap();

        println!(
            "Total time spent: {}ms",
            results.total_time_spent / 1000000.0
        );

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