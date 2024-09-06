use uwgpu::{BenchmarkComputePipeline, GPUContext};

pub struct MatmulResults;

pub async fn matmul_pipeline(_gpu: &GPUContext) -> BenchmarkComputePipeline {
    todo!()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use uwgpu::{
        wgpu::{
            util::BufferInitDescriptor, BufferDescriptor, BufferUsages,
            ShaderModuleDescriptor, ShaderSource,
        },
        Benchmark, BenchmarkComputePipeline, GPUContext, PipelineParameters,
    };

    #[tokio::test]
    async fn verify_matmul_works() {
        let gpu = GPUContext::new(None).await.unwrap();

        let matrix_a_data: Vec<f32> = (0u16..512).map(f32::from).collect();
        let matrix_b_data: Vec<f32> =
            (0u16..512).rev().map(f32::from).collect();
        let result_size = 512;
        let matrix_size = 512;

        let matrix_a_buffer = gpu.create_buffer_init(&BufferInitDescriptor {
            label: Some("Matrix A Buffer"),
            contents: bytemuck::cast_slice(&matrix_a_data),
            usage: BufferUsages::STORAGE,
        });

        let matrix_b_buffer = gpu.create_buffer_init(&BufferInitDescriptor {
            label: Some("Matrix B Buffer"),
            contents: bytemuck::cast_slice(&matrix_b_data),
            usage: BufferUsages::STORAGE,
        });

        let result_buffer = gpu.create_buffer(&BufferDescriptor {
            label: Some("Result Buffer"),
            size: (result_size * std::mem::size_of::<f32>()) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let matrix_size_buffer =
            gpu.create_buffer_init(&BufferInitDescriptor {
                label: Some("Matrix Size Buffer"),
                contents: bytemuck::cast_slice(&[matrix_size]),
                usage: BufferUsages::UNIFORM,
            });

        let pipeline = BenchmarkComputePipeline::new(PipelineParameters {
            shader: ShaderModuleDescriptor {
                label: Some("matmul shader"),
                source: ShaderSource::Wgsl(include_str!("matmul.wgsl").into()),
            },
            entry_point: "main",
            bind_group_0: HashMap::from([
                (0, matrix_a_buffer.as_entire_binding()),
                (1, matrix_b_buffer.as_entire_binding()),
                (2, result_buffer.as_entire_binding()),
                (3, matrix_size_buffer.as_entire_binding()),
            ]),
            gpu: &gpu,
            workgroups: (8, 8, 1),
        })
        .await
        .unwrap();

        let staging_buffer = gpu.create_buffer(&BufferDescriptor {
            label: Some("Staging Buffer"),
            size: (result_size * std::mem::size_of::<f32>()) as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let results = Benchmark {
            warmup_count: 0,
            count: 1,
            finalize_encoder_callback: Some(&|encoder| {
                for _ in 0..1 {
                    encoder.copy_buffer_to_buffer(
                        &result_buffer,
                        0,
                        &staging_buffer,
                        0,
                        (512 * std::mem::size_of::<f32>()) as u64,
                    )
                }
            }),
        }
        .run(pipeline)
        .await
        .unwrap();

        println!(
            "Total time spent: {}ms",
            results.total_time_spent / 1000000.0
        );

        // TODO: run benchmark, map staging buffer, verify results
        panic!();
    }
}
