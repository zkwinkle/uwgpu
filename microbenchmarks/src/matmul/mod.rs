//! Matrix multiplication microbenchmarks

use std::collections::HashMap;

use rand::{thread_rng, Rng};
use uwgpu::TimeUnit;
use uwgpu::{
    wgpu::{
        util::BufferInitDescriptor, BufferDescriptor, BufferUsages,
        ShaderModuleDescriptor, ShaderSource,
    },
    wgpu_async::AsyncBuffer,
    Benchmark, BenchmarkComputePipeline, BenchmarkResults, CreatePipelineError,
    GPUContext, PipelineParameters,
};

use crate::BenchmarkError;

const BENCHMARK_MATRIX_DIMS: usize = 1024;
const BENCHMARK_WARMUP_COUNT: usize = 5;
const BENCHMARK_ITERATIONS: usize = 10;

/// Microbenchmark for matrix mulitplication
///
/// Multiplies 2 randomly initialized 1024x1024 matrices repeatedly.
///
/// The workgroup size will dictate the size of workgroups used for the
/// computation.
pub async fn matmul_benchmark(
    workgroup_size: (u32, u32),
) -> Result<MatmulResults, BenchmarkError> {
    let gpu = GPUContext::new(None).await?;
    let buffers =
        Buffers::<BENCHMARK_MATRIX_DIMS>::new_with_random_inputs(&gpu);
    let pipeline = matmul_pipeline(&gpu, &buffers, &workgroup_size).await?;

    let results = Benchmark {
        warmup_count: BENCHMARK_WARMUP_COUNT,
        count: BENCHMARK_ITERATIONS,
        finalize_encoder_callback: None,
    }
    .run(pipeline)
    .await?;

    Ok(MatmulResults(results))
}

/// Results from the matrix multiplication microbenchmark. See
/// [matmul_benchmark].
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
pub struct MatmulResults(BenchmarkResults);

impl MatmulResults {
    /// Get the total amount of time in seconds spent executing the
    /// microbenchmark
    pub fn total_time_s(&self) -> f64 { self.0.total_time(TimeUnit::Second) }

    /// Get the amount of time per iteration in ms
    pub fn time_per_iteration_ms(&self) -> f64 {
        self.0.time_per_iteration(TimeUnit::Milli)
    }

    /// Get the amount of FLOPS (floating point operations per second)
    pub fn flops(&self) -> f64 {
        /// Reference for the amount of FLOPs in a matrix multiplication:
        /// https://math.stackexchange.com/questions/3512976/proof-of-of-flops-in-matrix-multiplication
        const NUM_FLOPS_PER_ITER: usize =
            2 * (BENCHMARK_MATRIX_DIMS.pow(3)) - BENCHMARK_MATRIX_DIMS.pow(2);

        ((NUM_FLOPS_PER_ITER * self.0.count) as f64) / (self.total_time_s())
    }
}

/// GPU buffers needed for microbenchmark
///
/// Matrices are assumed to be square matrices with MATRIX_DIMS x MATRIX_DIMS
/// dimensions.
struct Buffers<const MATRIX_DIMS: usize> {
    matrix_a_buffer: AsyncBuffer,
    matrix_b_buffer: AsyncBuffer,
    result_buffer: AsyncBuffer,
    matrix_size_buffer: AsyncBuffer,
}

impl<const MATRIX_DIMS: usize> Buffers<MATRIX_DIMS> {
    const MATRIX_SIZE: usize = MATRIX_DIMS * MATRIX_DIMS;

    fn new_with_random_inputs(gpu: &GPUContext) -> Self {
        let mut matrix_a_data = vec![0_f32; Self::MATRIX_SIZE];
        let mut matrix_b_data = vec![0_f32; Self::MATRIX_SIZE];

        let mut rng = thread_rng();

        rng.fill(matrix_a_data.as_mut_slice());
        rng.fill(matrix_b_data.as_mut_slice());

        Self::new_from_inputs(&matrix_a_data, &matrix_b_data, &gpu)
    }

    fn new_from_inputs(
        matrix_a_data: &[f32],
        matrix_b_data: &[f32],
        gpu: &GPUContext,
    ) -> Self {
        let matrix_a_buffer = gpu.create_buffer_init(&BufferInitDescriptor {
            label: Some("Matrix A Buffer"),
            contents: bytemuck::cast_slice(matrix_a_data),
            usage: BufferUsages::STORAGE,
        });

        let matrix_b_buffer = gpu.create_buffer_init(&BufferInitDescriptor {
            label: Some("Matrix B Buffer"),
            contents: bytemuck::cast_slice(matrix_b_data),
            usage: BufferUsages::STORAGE,
        });

        let result_buffer = gpu.create_buffer(&BufferDescriptor {
            label: Some("Result Buffer"),
            size: (Self::MATRIX_SIZE * std::mem::size_of::<f32>()) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let matrix_size_buffer =
            gpu.create_buffer_init(&BufferInitDescriptor {
                label: Some("Matrix Size Buffer"),
                contents: bytemuck::cast_slice(&[MATRIX_DIMS]),
                usage: BufferUsages::UNIFORM,
            });

        Self {
            matrix_a_buffer,
            matrix_b_buffer,
            result_buffer,
            matrix_size_buffer,
        }
    }
}

/// Pipeline needed for microbenchmark
///
/// Matrices are assumed to be square matrices with MATRIX_DIMS x MATRIX_DIMS
/// dimensions.
async fn matmul_pipeline<'a, const MATRIX_DIMS: usize>(
    gpu: &'a GPUContext,
    buffers: &'a Buffers<MATRIX_DIMS>,
    workgroup_size: &(u32, u32),
) -> Result<BenchmarkComputePipeline<'a>, CreatePipelineError> {
    BenchmarkComputePipeline::new(PipelineParameters {
        shader: ShaderModuleDescriptor {
            label: Some("matmul shader"),
            source: ShaderSource::Wgsl(include_str!("matmul.wgsl").into()),
        },
        entry_point: "main",
        bind_group_0: HashMap::from([
            (0, buffers.matrix_a_buffer.as_entire_binding()),
            (1, buffers.matrix_b_buffer.as_entire_binding()),
            (2, buffers.result_buffer.as_entire_binding()),
            (3, buffers.matrix_size_buffer.as_entire_binding()),
        ]),
        gpu,
        workgroups_dispatch: (
            1 + (MATRIX_DIMS / (workgroup_size.0 as usize)) as u32,
            1 + (MATRIX_DIMS / (workgroup_size.1 as usize)) as u32,
            1,
        ),
        workgroup_size: Some((workgroup_size.0, workgroup_size.1, 1)),
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

    /// Verifies that the matmul shader computes the matrix multiplication
    /// correctly.
    #[tokio::test]
    async fn matmul_works() {
        const MATRIX_DIMS: usize = 100;
        const MATRIX_SIZE: usize = MATRIX_DIMS * MATRIX_DIMS;

        // Incremental matrix [0, 1, 2, 3, 4 ; 5, 6, ... ;...]
        let matrix_a: Vec<f32> = (0..MATRIX_SIZE).map(|i| i as f32).collect();

        // Matrix of all 1s
        let matrix_b: Vec<f32> = (0..MATRIX_SIZE).map(|_| 1.0).collect();

        // Results in a matrix where all values in a row are the sum of
        // that row in matrix a
        let expected_result: Vec<f32> = (0..MATRIX_SIZE)
            .map(|i| {
                let row = i / MATRIX_DIMS;
                (0..MATRIX_DIMS)
                    .map(|column| (row * MATRIX_DIMS + column) as f32)
                    .sum()
            })
            .collect();

        let gpu = GPUContext::new(None).await.unwrap();
        let buffers =
            Buffers::<MATRIX_DIMS>::new_from_inputs(&matrix_a, &matrix_b, &gpu);
        let pipeline = matmul_pipeline(&gpu, &buffers, &(8, 8)).await.unwrap();

        let staging_buffer = gpu.create_buffer(&BufferDescriptor {
            label: Some("Staging Buffer"),
            size: (MATRIX_SIZE * std::mem::size_of::<f32>()) as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let _results = Benchmark {
            warmup_count: 0,
            count: 1,
            finalize_encoder_callback: Some(&|encoder| {
                encoder.copy_buffer_to_buffer(
                    &buffers.result_buffer,
                    0,
                    &staging_buffer,
                    0,
                    (MATRIX_SIZE * std::mem::size_of::<f32>()) as u64,
                )
            }),
        }
        .run(pipeline)
        .await
        .unwrap();

        let staging_slice = staging_buffer.slice(..);
        staging_slice.map_async(MapMode::Read).await.unwrap();

        let result_data: Vec<f32> = {
            let result_data_raw: &[u8] = &*staging_slice.get_mapped_range();
            bytemuck::cast_slice(&result_data_raw).to_vec()
        };

        staging_buffer.unmap();

        assert_eq!(result_data, expected_result);
    }

    /// Verifies that instantiating the matmul buffers with random inputs
    /// creates appropiately sized buffers, essentially by not panicking
    #[tokio::test]
    async fn matmul_random_inputs() {
        const MATRIX_DIMS: usize = 5;
        const MATRIX_SIZE: usize = MATRIX_DIMS * MATRIX_DIMS;

        let gpu = GPUContext::new(None).await.unwrap();
        let buffers = Buffers::<MATRIX_DIMS>::new_with_random_inputs(&gpu);
        let pipeline = matmul_pipeline(&gpu, &buffers, &(8, 8)).await.unwrap();

        let staging_buffer = gpu.create_buffer(&BufferDescriptor {
            label: Some("Staging Buffer"),
            size: (MATRIX_SIZE * std::mem::size_of::<f32>()) as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let _results = Benchmark {
            warmup_count: 0,
            count: 1,
            finalize_encoder_callback: Some(&|encoder| {
                encoder.copy_buffer_to_buffer(
                    &buffers.result_buffer,
                    0,
                    &staging_buffer,
                    0,
                    (MATRIX_SIZE * std::mem::size_of::<f32>()) as u64,
                )
            }),
        }
        .run(pipeline)
        .await
        .unwrap();

        let staging_slice = staging_buffer.slice(..);
        staging_slice.map_async(MapMode::Read).await.unwrap();

        let result_data: Vec<f32> = {
            let result_data_raw: &[u8] = &*staging_slice.get_mapped_range();
            bytemuck::cast_slice(&result_data_raw).to_vec()
        };

        staging_buffer.unmap();

        assert!(result_data.iter().all(|&x| x != 0.0));
    }
}
