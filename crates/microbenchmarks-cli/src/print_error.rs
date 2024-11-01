use microbenchmarks::{
    uwgpu::{
        wgpu::CompilationMessageType, CreatePipelineError, GetGPUContextError,
    },
    BenchmarkError,
};

pub fn print_error(error: BenchmarkError) {
    match error {
        BenchmarkError::GPUContext(gpu_error) => match gpu_error {
            GetGPUContextError::NoAdapter => {
                println!("Couldn't get handle to GPU adapter")
            }
            GetGPUContextError::RequestDevice(device_error) => println!(
                "Error requesting a device from the GPU adapter: {}",
                device_error
            ),
            GetGPUContextError::DoesNotSupportTimestamps => println!(
                "GPU adapter does not have support for timestamp queries"
            ),
            GetGPUContextError::DoesNotSupportRequestedFeatures(_) => println!(
                "GPU adapter does not support one of the required features"
            ),
        },
        BenchmarkError::PipelineCreation(pipeline_error) => {
            match pipeline_error {
                CreatePipelineError::ShaderCompilationError(
                    compilation_msgs,
                ) => {
                    println!("Error compiling shader:\n");
                    for msg in compilation_msgs {
                        print!(
                            "{}",
                            compilation_message_type_to_string(
                                msg.message_type
                            )
                        );
                        if let Some(location) = msg.location {
                            print!(" (line {})", location.line_number)
                        }
                        println!(": {}", msg.message);
                    }
                }
            }
        }
        BenchmarkError::MapTimestamp(_) => {
            println!("Couldn't read the timestamp query results")
        }
    }
}

fn compilation_message_type_to_string(
    msg_type: CompilationMessageType,
) -> String {
    match msg_type {
        CompilationMessageType::Error => "Error",
        CompilationMessageType::Warning => "Warning",
        CompilationMessageType::Info => "Info",
    }
    .to_string()
}
