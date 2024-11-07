//! Scan, also known as prefixed sum, microbenchmarks

use std::collections::HashMap;

use rand::{thread_rng, Rng};
use uwgpu::{
    wgpu::{
        util::BufferInitDescriptor, BindGroup, BufferUsages, ComputePass,
        ShaderModuleDescriptor, ShaderSource,
    },
    wgpu_async::AsyncBuffer,
    Benchmark, BenchmarkComputePipeline, BenchmarkResults, CreatePipelineError,
    GPUContext, PipelineParameters,
};
use uwgpu::{BindGroupParams, TimeUnit};

use crate::BenchmarkError;

/// 1MiB size buffer (of f32)
const BENCHMARK_BUFFER_SIZE: usize = 262_144;
const BENCHMARK_WARMUP_COUNT: usize = 200;
const BENCHMARK_ITERATIONS: usize = 2000;

/// Microbenchmark for a scan operation.
///
/// The workgroup size will dictate the size of workgroups used for the
/// computation.
///
/// ## Panic
///
/// This function panics if the workgroup size is not a power of 2, because it
/// hasn't really been considered how to handle such cases.
pub async fn scan_benchmark(
    workgroup_size: u32,
) -> Result<ScanResults, BenchmarkError> {
    assert!(workgroup_size.is_power_of_two());

    let gpu = GPUContext::new(None).await?;
    let buffers = Buffers::<BENCHMARK_BUFFER_SIZE>::new_with_random_input(
        workgroup_size,
        &gpu,
    );
    let pipeline = scan_pipeline(&gpu, &buffers, workgroup_size).await?;

    let results = Benchmark {
        warmup_count: BENCHMARK_WARMUP_COUNT,
        count: BENCHMARK_ITERATIONS,
        finalize_encoder_callback: None,
        workgroups_dispatch: workgroup_dispatches(
            buffers.strides.len(),
            BENCHMARK_BUFFER_SIZE,
            workgroup_size,
        ),
        dispatch_callback: Some(&dispatch_callback(
            &buffers.create_bind_groups(&pipeline),
        )),
    }
    .run(pipeline)
    .await?;

    Ok(ScanResults(results))
}

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// Results from the scan microbenchmark. See
/// [scan_benchmark].
///
/// Wraps a [BenchmarkResults] with some convenience methods.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[cfg_eval]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct ScanResults(
    #[cfg_attr(feature = "wasm", wasm_bindgen(getter_with_clone))]
    pub  BenchmarkResults,
);

#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl ScanResults {
    /// Get the amount of FLOPS (floating point operations per second)
    ///
    /// Note: The scan might've carried out some extra sums, but it
    /// would've been with padded out zeroes that don't affect the result,
    /// so I'm not counting those.
    pub fn flops(&self) -> f64 {
        const NUM_PASSES: usize =
            (BENCHMARK_BUFFER_SIZE * 2 - 1).ilog2() as usize;
        const NUM_FLOPS_PER_ITER: usize =
            NUM_PASSES * (BENCHMARK_BUFFER_SIZE / 2);

        (NUM_FLOPS_PER_ITER as f64 * self.0.count as f64)
            / (self.0.total_time(TimeUnit::Second))
    }
}

/// GPU buffers needed for microbenchmark
struct Buffers<const BUFFER_SIZE: usize> {
    /// Buffer for scan operation done in-place
    data_buffer: AsyncBuffer,
    /// For each pass that the scan has to do, a stride is given.
    strides: Vec<AsyncBuffer>,
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
        assert_eq!(input_data.len(), BUFFER_SIZE);

        let label = Some("Data Buffer");
        let usage = BufferUsages::STORAGE | BufferUsages::COPY_SRC;

        let rem = input_data.len() % workgroup_size as usize;
        let data_buffer = if rem != 0 {
            // 0 pad
            let mut data_vec = input_data.to_vec();
            data_vec.resize(
                input_data.len() + (workgroup_size as usize - rem),
                0.0,
            );
            gpu.create_buffer_init(&BufferInitDescriptor {
                label,
                usage,
                contents: bytemuck::cast_slice(&data_vec),
            })
        } else {
            gpu.create_buffer_init(&BufferInitDescriptor {
                label,
                usage,
                contents: bytemuck::cast_slice(input_data),
            })
        };

        let passes_needed = (input_data.len() * 2 - 1).ilog2();
        let strides = (1..=passes_needed).map(|i| 2u32.pow(i));

        let stride_buffers = strides
            .map(|stride: u32| {
                gpu.create_buffer_init(&BufferInitDescriptor {
                    label: Some("Stride Uniform"),
                    usage: BufferUsages::UNIFORM,
                    contents: bytemuck::cast_slice(&[stride]),
                })
            })
            .collect();

        Self {
            data_buffer,
            strides: stride_buffers,
        }
    }

    fn create_bind_groups(
        &self,
        pipeline: &BenchmarkComputePipeline,
    ) -> Vec<BindGroup> {
        self.strides
            .iter()
            .map(|stride_buffer| {
                pipeline.create_bind_group(BindGroupParams {
                    label: Some("Stride Uniform Bind Group"),
                    group: 1,
                    entries: HashMap::from([(
                        0,
                        stride_buffer.as_entire_binding(),
                    )]),
                })
            })
            .collect()
    }
}

/// Pipeline needed for microbenchmark
async fn scan_pipeline<'a, const BUFFER_SIZE: usize>(
    gpu: &'a GPUContext,
    buffers: &'a Buffers<BUFFER_SIZE>,
    workgroup_size: u32,
) -> Result<BenchmarkComputePipeline<'a>, CreatePipelineError> {
    BenchmarkComputePipeline::new(PipelineParameters {
        shader: ShaderModuleDescriptor {
            label: Some("scan shader"),
            source: ShaderSource::Wgsl(include_str!("scan.wgsl").into()),
        },
        entry_point: "main",
        bind_group_0: HashMap::from([(
            0,
            buffers.data_buffer.as_entire_binding(),
        )]),
        gpu,
        workgroup_size: Some((workgroup_size, 1, 1)),
    })
    .await
}

fn workgroup_dispatches(
    num_passes: usize,
    buffer_size: usize,
    workgroup_size: u32,
) -> Vec<(u32, u32, u32)> {
    std::iter::repeat_n(
        (
            1 + (buffer_size.div_ceil(workgroup_size as usize)) as u32,
            1,
            1,
        ),
        num_passes,
    )
    .collect()
}

fn dispatch_callback(
    bind_groups: &[BindGroup],
) -> impl Fn(usize, &mut ComputePass) + use<'_> {
    |i, pass: &mut ComputePass| {
        pass.set_bind_group(1, &bind_groups[i], &[]);
    }
}

#[cfg(test)]
mod tests {
    use uwgpu::{
        wgpu::{BufferDescriptor, BufferUsages, MapMode},
        Benchmark, GPUContext,
    };

    use super::{
        dispatch_callback, scan_pipeline, workgroup_dispatches, Buffers,
    };

    /// Verifies that the shader computes the scan correctly.
    #[tokio::test]
    async fn scan_works() {
        const BUFFER_SIZE: usize = 100;

        // Incremental buffer [1, 2, 3, 4, 5, 6, ... ]
        let input: Vec<f32> = (1..=BUFFER_SIZE).map(|i| i as f32).collect();
        let expected_result: Vec<f32> = input
            .iter()
            .scan(0.0, |acc, n| {
                *acc += *n;
                Some(*acc)
            })
            .collect();
        let workgroup_size = 8;

        let gpu = GPUContext::new(None).await.unwrap();
        let buffers = Buffers::<BUFFER_SIZE>::new_from_input(
            &input,
            workgroup_size,
            &gpu,
        );
        let pipeline =
            scan_pipeline(&gpu, &buffers, workgroup_size).await.unwrap();

        let staging_buffer = gpu.create_buffer(&BufferDescriptor {
            label: Some("Staging Buffer"),
            size: (BUFFER_SIZE * std::mem::size_of::<f32>()) as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let _results = Benchmark {
            warmup_count: 0,
            count: 1,
            workgroups_dispatch: workgroup_dispatches(
                buffers.strides.len(),
                BUFFER_SIZE,
                workgroup_size,
            ),
            dispatch_callback: Some(&dispatch_callback(
                &buffers.create_bind_groups(&pipeline),
            )),
            finalize_encoder_callback: Some(&|encoder| {
                encoder.copy_buffer_to_buffer(
                    &buffers.data_buffer,
                    0,
                    &staging_buffer,
                    0,
                    (BUFFER_SIZE * std::mem::size_of::<f32>()) as u64,
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

    /// Verifies that instantiating the scan buffers with random inputs
    /// creates appropiately sized buffers, essentially by not panicking
    #[tokio::test]
    async fn scan_random_inputs() {
        const BUFFER_SIZE: usize = 50;

        let workgroup_size = 8;
        let gpu = GPUContext::new(None).await.unwrap();
        let buffers =
            Buffers::<BUFFER_SIZE>::new_with_random_input(workgroup_size, &gpu);
        let pipeline =
            scan_pipeline(&gpu, &buffers, workgroup_size).await.unwrap();

        let staging_buffer = gpu.create_buffer(&BufferDescriptor {
            label: Some("Staging Buffer"),
            size: (BUFFER_SIZE * std::mem::size_of::<f32>()) as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let _results = Benchmark {
            warmup_count: 0,
            count: 1,
            workgroups_dispatch: workgroup_dispatches(
                buffers.strides.len(),
                BUFFER_SIZE,
                workgroup_size,
            ),
            dispatch_callback: Some(&dispatch_callback(
                &buffers.create_bind_groups(&pipeline),
            )),
            finalize_encoder_callback: Some(&|encoder| {
                encoder.copy_buffer_to_buffer(
                    &buffers.data_buffer,
                    0,
                    &staging_buffer,
                    0,
                    (BUFFER_SIZE * std::mem::size_of::<f32>()) as u64,
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

        assert!(result_data.iter().all(|f| f.is_normal()));
    }
}
