//! Convolution microbenchmarks

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
const KERNEL_MATRIX_DIMS: usize = 3;
const BENCHMARK_WARMUP_COUNT: usize = 300;
const BENCHMARK_ITERATIONS: usize = 3000;

/// Microbenchmark for convolution.
///
/// Applies a random 3x3 kernel to a 1024x1024 matrix of random data.
pub async fn convolution_benchmark(
    workgroup_size: (u32, u32),
) -> Result<ConvolutionResults, BenchmarkError> {
    let gpu = GPUContext::new(None).await?;
    let buffers =
        Buffers::<BENCHMARK_MATRIX_DIMS, KERNEL_MATRIX_DIMS>::new_with_random_inputs(&gpu);
    let pipeline =
        convolution_pipeline(&gpu, &buffers, &workgroup_size).await?;

    let results = Benchmark {
        warmup_count: BENCHMARK_WARMUP_COUNT,
        count: BENCHMARK_ITERATIONS,
        finalize_encoder_callback: None,
        workgroups_dispatch: workgroups_dispatch(
            BENCHMARK_MATRIX_DIMS,
            workgroup_size,
        ),
        dispatch_callback: None,
    }
    .run(pipeline)
    .await?;

    Ok(ConvolutionResults(results))
}

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// Results from the convlution microbenchmark. See
/// [convolution_benchmark].
///
/// Wraps a [BenchmarkResults] with some convenience methods.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[cfg_eval]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct ConvolutionResults(
    #[cfg_attr(feature = "wasm", wasm_bindgen(getter_with_clone))]
    pub  BenchmarkResults,
);

#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl ConvolutionResults {
    /// Get the amount of FLOPS (floating point operations per second)
    pub fn flops(&self) -> f64 {
        // 4 corners * ( 4 muls + 4 sums)
        const FLOPS_CORNERS: usize = 4 * (4 + 4);
        // 4 * elements in borders without corners * ( 6 muls + 6 sums)
        const FLOPS_INNER_BORDERS: usize =
            4 * (BENCHMARK_MATRIX_DIMS - 2) * (6 + 6);
        // elements in inner matrix (no borders) * ( 9 muls + 9 sums)
        const FLOPS_INNER_MATRIX: usize =
            (BENCHMARK_MATRIX_DIMS - 1) * (BENCHMARK_MATRIX_DIMS - 1) * (9 + 9);

        const NUM_FLOPS_PER_ITER: usize =
            FLOPS_CORNERS + FLOPS_INNER_BORDERS + FLOPS_INNER_MATRIX;

        (NUM_FLOPS_PER_ITER as f64 * self.0.count as f64)
            / (self.0.total_time(TimeUnit::Second))
    }
}

/// GPU buffers needed for microbenchmark
///
/// Matrices are assumed to be square matrices with MATRIX_DIMS x MATRIX_DIMS
/// dimensions.
struct Buffers<const MATRIX_DIMS: usize, const KERNEL_DIMS: usize> {
    input_matrix_buffer: AsyncBuffer,
    kernel_buffer: AsyncBuffer,
    result_buffer: AsyncBuffer,
    matrix_size_buffer: AsyncBuffer,
    kernel_size_buffer: AsyncBuffer,
}

impl<const MATRIX_DIMS: usize, const KERNEL_DIMS: usize>
    Buffers<MATRIX_DIMS, KERNEL_DIMS>
{
    const MATRIX_SIZE: usize = MATRIX_DIMS * MATRIX_DIMS;
    const KERNEL_SIZE: usize = KERNEL_DIMS * KERNEL_DIMS;

    fn new_with_random_inputs(gpu: &GPUContext) -> Self {
        let mut input_matrix = vec![0_f32; Self::MATRIX_SIZE];
        let mut kernel_data = vec![0_f32; Self::KERNEL_SIZE];

        let mut rng = thread_rng();

        rng.fill(input_matrix.as_mut_slice());
        rng.fill(kernel_data.as_mut_slice());

        Self::new_from_inputs(&input_matrix, &kernel_data, &gpu)
    }

    fn new_from_inputs(
        input_matrix_data: &[f32],
        kernel_data: &[f32],
        gpu: &GPUContext,
    ) -> Self {
        assert_eq!(input_matrix_data.len(), Self::MATRIX_SIZE);
        assert_eq!(kernel_data.len(), Self::KERNEL_SIZE);

        let matrix_a_buffer = gpu.create_buffer_init(&BufferInitDescriptor {
            label: Some("Matrix A Buffer"),
            contents: bytemuck::cast_slice(input_matrix_data),
            usage: BufferUsages::STORAGE,
        });

        let kernel_buffer = gpu.create_buffer_init(&BufferInitDescriptor {
            label: Some("Kernel Buffer"),
            contents: bytemuck::cast_slice(kernel_data),
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

        let kernel_size_buffer =
            gpu.create_buffer_init(&BufferInitDescriptor {
                label: Some("Kernel Size Buffer"),
                contents: bytemuck::cast_slice(&[KERNEL_DIMS]),
                usage: BufferUsages::UNIFORM,
            });

        Self {
            input_matrix_buffer: matrix_a_buffer,
            kernel_buffer,
            result_buffer,
            matrix_size_buffer,
            kernel_size_buffer,
        }
    }
}

/// Pipeline needed for microbenchmark
///
/// Matrices are assumed to be square matrices with MATRIX_DIMS x MATRIX_DIMS
/// dimensions.
async fn convolution_pipeline<
    'a,
    const MATRIX_DIMS: usize,
    const KERNEL_DIMS: usize,
>(
    gpu: &'a GPUContext,
    buffers: &'a Buffers<MATRIX_DIMS, KERNEL_DIMS>,
    workgroup_size: &(u32, u32),
) -> Result<BenchmarkComputePipeline<'a>, CreatePipelineError> {
    BenchmarkComputePipeline::new(PipelineParameters {
        shader: ShaderModuleDescriptor {
            label: Some("convolution shader"),
            source: ShaderSource::Wgsl(include_str!("convolution.wgsl").into()),
        },
        entry_point: "main",
        bind_group_0: HashMap::from([
            (0, buffers.input_matrix_buffer.as_entire_binding()),
            (1, buffers.kernel_buffer.as_entire_binding()),
            (2, buffers.result_buffer.as_entire_binding()),
            (3, buffers.matrix_size_buffer.as_entire_binding()),
            (4, buffers.kernel_size_buffer.as_entire_binding()),
        ]),
        gpu,
        workgroup_size: Some((workgroup_size.0, workgroup_size.1, 1)),
    })
    .await
}

fn workgroups_dispatch(
    matrix_dims: usize,
    workgroup_size: (u32, u32),
) -> Vec<(u32, u32, u32)> {
    vec![(
        1 + (matrix_dims / (workgroup_size.0 as usize)) as u32,
        1 + (matrix_dims / (workgroup_size.1 as usize)) as u32,
        1,
    )]
}

#[cfg(test)]
mod tests {
    use std::iter;

    use uwgpu::{
        wgpu::{BufferDescriptor, BufferUsages, MapMode},
        Benchmark, GPUContext,
    };

    use super::*;

    /// Verifies that the convolutoin shader computes the operation correctly.
    #[tokio::test]
    async fn convolution_works() {
        const MATRIX_DIMS: usize = 5;
        const MATRIX_SIZE: usize = MATRIX_DIMS * MATRIX_DIMS;
        const KERNEL_DIMS: usize = 3;

        // Incremental gradient matrix [1, 2, 3, 4, 5 ; 6, 7, ... ;...]
        let input_matrix: Vec<f32> =
            (1..=MATRIX_SIZE).map(|i| i as f32).collect();

        // One of the sobel operators
        let kernel: [f32; KERNEL_DIMS * KERNEL_DIMS] =
            [1.0, 0.0, -1.0, 2.0, 0.0, -2.0, 1.0, 0.0, -1.0];

        // Results in a matrix where only values in the edge columns are
        // different, everything in the center columns is 6 for edge rows or 8.
        let expected_result: Vec<f32> = (0..MATRIX_DIMS)
            .flat_map(|row| {
                let left_edge = if row != 0 {
                    input_matrix[(row - 1) * MATRIX_DIMS + 1] * kernel[0]
                } else {
                    0.0
                } + if row != MATRIX_DIMS - 1 {
                    input_matrix[(row + 1) * MATRIX_DIMS + 1] * kernel[6]
                } else {
                    0.0
                } + input_matrix[row * MATRIX_DIMS + 1]
                    * kernel[3];

                let right_edge = if row != 0 {
                    input_matrix[row * MATRIX_DIMS - 2] * kernel[2]
                } else {
                    0.0
                } + if row != MATRIX_DIMS - 1 {
                    input_matrix[(row + 2) * MATRIX_DIMS - 2] * kernel[8]
                } else {
                    0.0
                } + input_matrix[(row + 1) * MATRIX_DIMS - 2]
                    * kernel[5];

                let middle_values =
                    if row == 0 || row == MATRIX_DIMS - 1 { 6.0 } else { 8.0 };

                iter::once(left_edge)
                    .chain(iter::repeat_n(middle_values, MATRIX_DIMS - 2))
                    .chain(iter::once(right_edge))
            })
            .collect();

        let workgroup_size = (8, 8);
        let gpu = GPUContext::new(None).await.unwrap();
        let buffers = Buffers::<MATRIX_DIMS, KERNEL_DIMS>::new_from_inputs(
            &input_matrix,
            &kernel,
            &gpu,
        );
        let pipeline = convolution_pipeline(&gpu, &buffers, &workgroup_size)
            .await
            .unwrap();

        let staging_buffer = gpu.create_buffer(&BufferDescriptor {
            label: Some("Staging Buffer"),
            size: (MATRIX_SIZE * std::mem::size_of::<f32>()) as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let _results = Benchmark {
            warmup_count: 0,
            count: 1,
            workgroups_dispatch: workgroups_dispatch(
                MATRIX_DIMS,
                workgroup_size,
            ),
            dispatch_callback: None,
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
    async fn convolution_random_inputs() {
        const MATRIX_DIMS: usize = 5;
        const KERNEL_DIMS: usize = 3;
        const MATRIX_SIZE: usize = MATRIX_DIMS * MATRIX_DIMS;

        let workgroup_size = (8, 8);
        let gpu = GPUContext::new(None).await.unwrap();
        let buffers =
            Buffers::<MATRIX_DIMS, KERNEL_DIMS>::new_with_random_inputs(&gpu);
        let pipeline = convolution_pipeline(&gpu, &buffers, &workgroup_size)
            .await
            .unwrap();

        let staging_buffer = gpu.create_buffer(&BufferDescriptor {
            label: Some("Staging Buffer"),
            size: (MATRIX_SIZE * std::mem::size_of::<f32>()) as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let _results = Benchmark {
            warmup_count: 0,
            count: 1,
            workgroups_dispatch: workgroups_dispatch(
                MATRIX_DIMS,
                workgroup_size,
            ),
            dispatch_callback: None,
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
