//! Microbenchmark for copying buffer to buffer copy throughput in a sequential
//! manner

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use uwgpu::{
        wgpu::{
            util::BufferInitDescriptor, BufferDescriptor, BufferUsages,
            MapMode, ShaderModuleDescriptor, ShaderSource,
        },
        Benchmark, BenchmarkComputePipeline, GPUContext, PipelineParameters,
    };

    #[tokio::test]
    async fn verify_buffer_copy_works() {
        let gpu = GPUContext::new(None).await.unwrap();

        // 1MiB size buffer of random data
        //
        // 1 MiB = 2^20 = 1_048_576
        //
        // 1MiB / 4bytes = 262_144
        let source_buffer_data: Box<[u32; 262_144]> = (0..262_144)
            .map(|_| rand::random())
            .collect::<Vec<u32>>()
            .try_into()
            .unwrap();

        // Destination initialized to 0s
        let destination_buffer_data: Box<[u32; 262_144]> =
            Box::new([0_u32; 262_144]);

        let expected_result = source_buffer_data.clone();

        let source_buffer = gpu.create_buffer_init(&BufferInitDescriptor {
            label: Some("Source Buffer"),
            contents: bytemuck::cast_slice(&*source_buffer_data),
            usage: BufferUsages::STORAGE,
        });

        let destination_buffer =
            gpu.create_buffer_init(&BufferInitDescriptor {
                label: Some("Desination Buffer"),
                contents: bytemuck::cast_slice(&*destination_buffer_data),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            });

        let pipeline = BenchmarkComputePipeline::new(PipelineParameters {
            shader: ShaderModuleDescriptor {
                label: Some("sequential buffer copy shader"),
                source: ShaderSource::Wgsl(
                    include_str!("buffer_sequential.wgsl").into(),
                ),
            },
            entry_point: "main",
            bind_group_0: HashMap::from([
                (0, source_buffer.as_entire_binding()),
                (1, destination_buffer.as_entire_binding()),
            ]),
            gpu: &gpu,

            // WG size = 64
            // 1MiB u32 = 262_144 elements
            // 262_144 / 64 = 4096
            workgroups: (4096, 1, 1),
        })
        .await
        .unwrap();

        let staging_buffer = gpu.create_buffer(&BufferDescriptor {
            label: Some("Staging Buffer"),
            size: 2_u64.pow(20),
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let results = Benchmark {
            warmup_count: 0,
            count: 1,
            finalize_encoder_callback: Some(&|encoder| {
                println!("This is getting called");
                encoder.copy_buffer_to_buffer(
                    &destination_buffer,
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

        // TODO: Decente simple memcpy BW microbenchmark.
        // For the real benchmark I probs want to vary the data type, maybe try
        // different buffer sizes.
        todo!("Once this microbenchmark is no longer just a test and we don't really need to print info here I'll remove this todo!");
    }
}
