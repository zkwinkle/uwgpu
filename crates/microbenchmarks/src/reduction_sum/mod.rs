//! Reduction sum microbenchmark

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

/// 1MiB size buffer (of f32)
const BENCHMARK_BUFFER_SIZE: usize = 262_144;
const BENCHMARK_WARMUP_COUNT: usize = 2000;
const BENCHMARK_ITERATIONS: usize = 20000;

/// Microbenchmark for a reduction sum operation. Sums all the elements of an
/// array.
///
/// The workgroup size will dictate the size of workgroups used for the
/// computation.
pub async fn reduction_sum_benchmark(
    workgroup_size: u32,
) -> Result<ReductionSumResults, BenchmarkError> {
    let gpu = GPUContext::new(None).await?;
    let buffers = Buffers::<BENCHMARK_BUFFER_SIZE>::new_with_random_input(
        workgroup_size,
        &gpu,
    );
    let pipeline =
        reduction_sum_pipeline(&gpu, &buffers, workgroup_size).await?;

    let results = Benchmark {
        warmup_count: BENCHMARK_WARMUP_COUNT,
        count: BENCHMARK_ITERATIONS,
        finalize_encoder_callback: None,
    }
    .run(pipeline)
    .await?;

    Ok(ReductionSumResults(results))
}

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// Results from the sum reduction microbenchmark. See
/// [reduction_sum_benchmark].
///
/// Wraps a [BenchmarkResults] with some convenience methods.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[cfg_eval]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct ReductionSumResults(
    #[cfg_attr(feature = "wasm", wasm_bindgen(getter_with_clone))]
    pub  BenchmarkResults,
);

#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl ReductionSumResults {
    /// Get the amount of FLOPS (floating point operations per second)
    ///
    /// Note: The reduction might've carried out some extra sums, but it
    /// would've been with padded out zeroes that don't affect the result,
    /// so I'm not counting those.
    pub fn flops(&self) -> f64 {
        const NUM_FLOPS_PER_ITER: usize = BENCHMARK_BUFFER_SIZE - 1;

        (NUM_FLOPS_PER_ITER as f64 * self.0.count as f64)
            / (self.0.total_time(TimeUnit::Second))
    }
}

/// GPU buffers needed for microbenchmark
struct Buffers<const BUFFER_SIZE: usize> {
    input_buffer: AsyncBuffer,
    /// The final result will be in the [0] element, but the buffer is actually
    /// of size:
    ///
    /// BUFFER_SIZE / workgroup_size (integer division rounded up)
    /// TODO: Change
    ///
    /// The extra space is used to calculate intermediate results of the
    /// reduction.
    ///
    /// The reason it's BUFFER_SIZE / workgroup_size is because in "each round"
    /// of reduction the result array gets reduced by a factor of
    /// workgroup_size
    ///
    /// For example, if workgroup_size is 4, the result gets reduced by a
    /// factor of 4.
    result_buffer: AsyncBuffer,
    barriers_buffer: AsyncBuffer,
    elements_left_buffer: AsyncBuffer,
}

impl<const BUFFER_SIZE: usize> Buffers<BUFFER_SIZE> {
    fn new_with_random_input(workgroup_size: u32, gpu: &GPUContext) -> Self {
        let mut input_data = vec![0_f32; BUFFER_SIZE];

        let mut rng = thread_rng();

        rng.fill(input_data.as_mut_slice());

        Self::new_from_input(&input_data, workgroup_size, &gpu)
    }

    fn new_from_input(
        input_data: &[f32],
        workgroup_size: u32,
        gpu: &GPUContext,
    ) -> Self {
        let input_buffer = gpu.create_buffer_init(&BufferInitDescriptor {
            label: Some("Input Buffer"),
            contents: bytemuck::cast_slice(input_data),
            usage: BufferUsages::STORAGE,
        });

        let result_buffer = gpu.create_buffer(&BufferDescriptor {
            label: Some("Result Buffer"),
            // See `result_buffer` field docs for size explanation.
            size: ((BUFFER_SIZE.div_ceil(workgroup_size as usize))
                * std::mem::size_of::<f32>()) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let mut elements_per_pass: Vec<u32> = vec![];
        let mut elements = BUFFER_SIZE as u32;
        while elements > 1 {
            elements = elements.div_ceil(workgroup_size);
            elements_per_pass.push(elements);
        }
        let elements_per_pass = elements_per_pass;

        println!("elements per pass: {:?}", elements_per_pass);

        // See `result_buffer` field docs for size explanation.
        let barriers_buffer_size: u32 =
            elements_per_pass[1..].iter().sum::<u32>().max(1);
        let barriers_buffer = gpu.create_buffer_init(&BufferInitDescriptor {
            label: Some("Barriers Buffer"),
            contents: bytemuck::cast_slice(&vec![
                0f32;
                barriers_buffer_size as usize
            ]),
            usage: BufferUsages::STORAGE,
        });

        let elements_left_buffer =
            gpu.create_buffer_init(&BufferInitDescriptor {
                label: Some("Elements Left Buffer"),
                contents: bytemuck::cast_slice(&elements_per_pass),
                usage: BufferUsages::STORAGE,
            });

        Self {
            input_buffer,
            result_buffer,
            barriers_buffer,
            elements_left_buffer,
        }
    }
}

/// Pipeline needed for microbenchmark
async fn reduction_sum_pipeline<'a, const BUFFER_SIZE: usize>(
    gpu: &'a GPUContext,
    buffers: &'a Buffers<BUFFER_SIZE>,
    workgroup_size: u32,
) -> Result<BenchmarkComputePipeline<'a>, CreatePipelineError> {
    BenchmarkComputePipeline::new(PipelineParameters {
        shader: ShaderModuleDescriptor {
            label: Some("reduction sum shader"),
            source: ShaderSource::Wgsl(
                include_str!("reduction_sum.wgsl").into(),
            ),
        },
        entry_point: "main",
        bind_group_0: HashMap::from([
            (0, buffers.input_buffer.as_entire_binding()),
            (1, buffers.result_buffer.as_entire_binding()),
            (2, buffers.barriers_buffer.as_entire_binding()),
            (3, buffers.elements_left_buffer.as_entire_binding()),
        ]),
        gpu,
        workgroups_dispatch: (
            1 + (BUFFER_SIZE.div_ceil(workgroup_size as usize)) as u32,
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

    /// Verifies that the shader computes the reduction sum correctly.
    #[tokio::test]
    async fn reduction_works() {
        const BUFFER_SIZE: usize = 1000;

        // Incremental buffer [0, 1, 2, 3, 4, 5, 6, ... ]
        let input: Vec<f32> = (1..=BUFFER_SIZE).map(|i| i as f32).collect();
        let expected_result: f32 =
            (BUFFER_SIZE * (BUFFER_SIZE + 1)) as f32 / 2.0;
        // arbitrary
        let workgroup_size = 8;

        let gpu = GPUContext::new(None).await.unwrap();
        let buffers = Buffers::<BUFFER_SIZE>::new_from_input(
            &input,
            workgroup_size,
            &gpu,
        );
        let pipeline = reduction_sum_pipeline(&gpu, &buffers, workgroup_size)
            .await
            .unwrap();

        let staging_buffer = gpu.create_buffer(&BufferDescriptor {
            label: Some("Staging Buffer"),
            size: std::mem::size_of::<f32>() as u64,
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
                    std::mem::size_of::<f32>() as u64,
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

        assert_eq!(result_data.len(), 1);
        assert_eq!(result_data[0], expected_result);
    }

    /// Verifies that instantiating the reduction buffers with random inputs
    /// creates appropiately sized buffers, essentially by not panicking
    #[tokio::test]
    async fn reduction_random_inputs() {
        const BUFFER_SIZE: usize = 5;

        // arbitrary
        let workgroup_size = 8;

        let gpu = GPUContext::new(None).await.unwrap();
        let buffers =
            Buffers::<BUFFER_SIZE>::new_with_random_input(workgroup_size, &gpu);
        let pipeline = reduction_sum_pipeline(&gpu, &buffers, workgroup_size)
            .await
            .unwrap();

        let staging_buffer = gpu.create_buffer(&BufferDescriptor {
            label: Some("Staging Buffer"),
            size: std::mem::size_of::<f32>() as u64,
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
                    std::mem::size_of::<f32>() as u64,
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

        assert_eq!(result_data.len(), 1);
        assert!(result_data[0].is_finite());
        assert_ne!(result_data[0], 0.0);
    }
}
