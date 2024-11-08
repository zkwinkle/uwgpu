//! Microbenchmark for texture to texture copy throughput

use std::collections::HashMap;

use rand::{thread_rng, Rng};
use uwgpu::{
    wgpu::{
        BindingResource, Extent3d, ShaderModuleDescriptor, ShaderSource,
        Texture, TextureDescriptor, TextureDimension, TextureFormat,
        TextureUsages, TextureViewDescriptor,
    },
    Benchmark, BenchmarkComputePipeline, BenchmarkResults, CreatePipelineError,
    GPUContext, PipelineParameters, TimeUnit,
};

use crate::BenchmarkError;

const BENCHMARK_TEXTURE_DIMS: usize = 1024;
const BENCHMARK_MEMORY_SIZE: usize =
    BENCHMARK_TEXTURE_DIMS * BENCHMARK_TEXTURE_DIMS;
const BENCHMARK_WARMUP_COUNT: usize = 1000;
const BENCHMARK_ITERATIONS: usize = 10000;

/// Microbenchmark for measuring the Texture -> Texture memory copy BW within
/// the GPU
pub async fn texture_to_texture_benchmark(
    workgroup_size: (u32, u32),
) -> Result<TextureToTextureResults, BenchmarkError> {
    let gpu = GPUContext::new(None).await?;
    let textures =
        Bindings::<BENCHMARK_TEXTURE_DIMS>::new_with_random_inputs(&gpu);
    let pipeline =
        texture_to_texture_pipeline(&gpu, &textures, workgroup_size).await?;
    let results = Benchmark {
        warmup_count: BENCHMARK_WARMUP_COUNT,
        count: BENCHMARK_ITERATIONS,
        finalize_encoder_callback: None,
        workgroups_dispatch: workgroups_dispatch(
            BENCHMARK_TEXTURE_DIMS,
            workgroup_size,
        ),
        dispatch_callback: None,
    }
    .run(pipeline)
    .await?;

    Ok(TextureToTextureResults(results))
}

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// Results from the memcpy texture->texture benchmark. See
/// [texture_to_texture_benchmark].
///
/// Wraps a [BenchmarkResults] with some convenience methods.
#[cfg_eval]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct TextureToTextureResults(
    #[cfg_attr(feature = "wasm", wasm_bindgen(getter_with_clone))]
    pub  BenchmarkResults,
);

#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl TextureToTextureResults {
    /// Get the Bandwidth of memory copy in bytes per second
    pub fn bandwidth(&self) -> f64 {
        ((BENCHMARK_MEMORY_SIZE * std::mem::size_of::<u32>() * self.0.count)
            as f64)
            / self.0.total_time(TimeUnit::Second)
    }
}

/// GPU binding needed for microbenchmark
struct Bindings<const TEXTURE_DIMS: usize> {
    source_texture: Texture,
    destination_texture: Texture,
}

impl<const TEXTURE_DIMS: usize> Bindings<TEXTURE_DIMS> {
    const MEM_SIZE: usize = TEXTURE_DIMS * TEXTURE_DIMS;

    fn new_with_random_inputs(gpu: &GPUContext) -> Self {
        let mut source_texture_data = vec![0u32; Self::MEM_SIZE];

        let mut rng = thread_rng();

        rng.fill(source_texture_data.as_mut_slice());

        Self::new_from_source_data(source_texture_data.as_slice(), gpu)
    }

    fn new_from_source_data(source_data: &[u32], gpu: &GPUContext) -> Self {
        assert_eq!(source_data.len(), Self::MEM_SIZE);

        let source_texture = gpu.create_texture_with_data(
            &TextureDescriptor {
                label: Some("Source Texture"),
                size: Extent3d {
                    width: TEXTURE_DIMS as u32,
                    height: TEXTURE_DIMS as u32,
                    depth_or_array_layers: 1,
                },
                format: TextureFormat::Rgba8Uint,
                usage: TextureUsages::TEXTURE_BINDING,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                view_formats: &[],
            },
            Default::default(),
            bytemuck::cast_slice(source_data),
        );

        let destination_texture = gpu.create_texture(&TextureDescriptor {
            label: Some("Destination Texture"),
            size: Extent3d {
                width: TEXTURE_DIMS as u32,
                height: TEXTURE_DIMS as u32,
                depth_or_array_layers: 1,
            },
            format: TextureFormat::Rgba8Uint,
            usage: TextureUsages::COPY_SRC
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::STORAGE_BINDING,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            view_formats: &[],
        });

        Self {
            source_texture,
            destination_texture,
        }
    }
}

/// Pipeline needed for microbenchmark.
async fn texture_to_texture_pipeline<'a, const TEXTURE_DIMS: usize>(
    gpu: &'a GPUContext,
    textures: &'a Bindings<TEXTURE_DIMS>,
    workgroup_size: (u32, u32),
) -> Result<BenchmarkComputePipeline<'a>, CreatePipelineError> {
    BenchmarkComputePipeline::new(PipelineParameters {
        shader: ShaderModuleDescriptor {
            label: Some("texture to texture copy shader"),
            source: ShaderSource::Wgsl(
                include_str!("texture_to_texture.wgsl").into(),
            ),
        },
        entry_point: "main",
        bind_group_0: HashMap::from([
            (
                0,
                BindingResource::TextureView(
                    &textures
                        .source_texture
                        .create_view(&TextureViewDescriptor::default()),
                ),
            ),
            (
                1,
                BindingResource::TextureView(
                    &textures
                        .destination_texture
                        .create_view(&TextureViewDescriptor::default()),
                ),
            ),
        ]),
        gpu: &gpu,

        workgroup_size: Some((workgroup_size.0, workgroup_size.1, 1)),
    })
    .await
}

fn workgroups_dispatch(
    texture_dims: usize,
    workgroup_size: (u32, u32),
) -> Vec<(u32, u32, u32)> {
    vec![(
        1 + (texture_dims / (workgroup_size.0 as usize)) as u32,
        1 + (texture_dims / (workgroup_size.1 as usize)) as u32,
        1,
    )]
}

#[cfg(test)]
mod tests {

    use uwgpu::{
        wgpu::{
            BufferDescriptor, BufferUsages, ImageCopyBuffer, ImageDataLayout,
            MapMode,
        },
        Benchmark, GPUContext,
    };

    use super::*;

    #[tokio::test]
    async fn verify_buffer_to_texture_copy_works() {
        let gpu = GPUContext::new(None).await.unwrap();

        const TEXTURE_DIMS: usize = 100;
        const MEMORY_SIZE: usize = TEXTURE_DIMS * TEXTURE_DIMS;

        let source_buffer_data: Vec<u32> =
            (0..MEMORY_SIZE).map(|i| i as u32).collect();

        let expected_result = source_buffer_data.clone();

        let buffers = Bindings::<TEXTURE_DIMS>::new_from_source_data(
            &source_buffer_data,
            &gpu,
        );

        let workgroup_size = (8, 8);
        let pipeline =
            texture_to_texture_pipeline(&gpu, &buffers, workgroup_size)
                .await
                .unwrap();

        let row_len = TEXTURE_DIMS.next_multiple_of(256);
        let staging_buffer_size = row_len * TEXTURE_DIMS;
        let staging_buffer = gpu.create_buffer(&BufferDescriptor {
            label: Some("Staging Buffer"),
            size: (staging_buffer_size * std::mem::size_of::<u32>()) as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let _results = Benchmark {
            warmup_count: 0,
            count: 1,
            finalize_encoder_callback: Some(&|encoder| {
                encoder.copy_texture_to_buffer(
                    buffers.destination_texture.as_image_copy(),
                    ImageCopyBuffer {
                        buffer: &staging_buffer,
                        layout: ImageDataLayout {
                            bytes_per_row: Some(
                                (row_len * std::mem::size_of::<u32>()) as u32,
                            ),
                            ..Default::default()
                        },
                    },
                    Extent3d {
                        width: TEXTURE_DIMS as u32,
                        height: TEXTURE_DIMS as u32,
                        depth_or_array_layers: 1,
                    },
                )
            }),
            workgroups_dispatch: workgroups_dispatch(
                TEXTURE_DIMS,
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

        // if texture dims is not a multiple of 256 the buffer will be
        // padded with 0s.

        let result_data = if TEXTURE_DIMS == row_len {
            result_data
        } else {
            result_data
                .iter()
                .enumerate()
                .filter_map(|(i, val)| {
                    if (i % row_len) >= TEXTURE_DIMS {
                        None
                    } else {
                        Some(*val)
                    }
                })
                .collect()
        };

        assert_eq!(&result_data, &expected_result);
    }
}
