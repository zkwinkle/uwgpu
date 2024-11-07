use serde::{Deserialize, Serialize};
use MicrobenchmarkKind::*;

/// A specific benchmark supported by the website.
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum MicrobenchmarkKind {
    Matmul,
    Reduction,
    Convolution,
    Scan,
    BufferSequential,
    BufferShuffled,
    BufferToBuffer,
    BufferToTexture,
    TextureToTexture,
}

impl MicrobenchmarkKind {
    /// Crafts the call to the JS `run_microbenchmark` function defined on the
    /// [Layout](crate::routes::extractors::Layout) based on the specific
    /// microbenchmark.
    ///
    /// This function assumes the existence of 2 variables:
    ///
    /// - `results_div`: the element results will be appended to.
    /// - `disable_checkbox`: Checkbox element that if checked will disable
    ///   POSTing results.
    pub fn run_microbenchmark_fn(&self) -> String {
        format!(
            r#"run_microbenchmark({microbenchmark_json},
                                      "{wasm_benchmark_fn_str}",
                                      {workgroups_array},
                                      "{custom_result_fn_str}",
                                      (result) => {create_custom_result},
                                      results_div,
                                      disable_checkbox)"#,
            microbenchmark_json = serde_json::to_string(&self).unwrap(),
            wasm_benchmark_fn_str = self.wasm_benchmark_function(),
            workgroups_array = self.benchmark_workgroups(),
            custom_result_fn_str = self.custom_result_function(),
            create_custom_result = self.custom_result(),
        )
    }

    fn wasm_benchmark_function(&self) -> &'static str {
        match self {
            Matmul => "wasm_matmul_benchmark",
            Reduction => "wasm_reduction_sum_benchmark",
            Convolution => "wasm_convolution_benchmark",
            Scan => "wasm_scan_benchmark",
            BufferSequential => todo!(),
            BufferToBuffer => "wasm_buffer_sequential_benchmark",
            BufferShuffled => todo!(),
            BufferToTexture => todo!(),
            TextureToTexture => todo!(),
        }
    }

    fn benchmark_workgroups(&self) -> &'static str {
        match self {
            // Accessing same-row should be faster than accessing different rows
            // which is why we use column-dominant workgroups
            Matmul | Convolution => {
                "[[4, 8], [2, 16], [1, 32], [8, 8], [4, 16], [2, 32],
             [1, 64], [8, 16], [4, 32], [2, 64], [1, 128], [16, 16], [8, 32],
             [4, 64], [2, 128], [1, 256]]"
            }
            Reduction | Scan => "[8, 16, 32, 64, 128, 256]",
            BufferSequential => todo!(),
            BufferShuffled => todo!(),
            BufferToBuffer => "[32, 64, 128, 256]",
            BufferToTexture => todo!(),
            TextureToTexture => todo!(),
        }
    }

    /// JS instructions for getting an extra line with a custom result such as
    /// FLOPs or Bandwidth
    ///
    /// Assume there is a `result` variable in the script with the benchmark
    /// results.
    ///
    /// The code should be an expression to create a string for the line with
    /// the custom result.
    fn custom_result(&self) -> &'static str {
        match self {
            Matmul | Reduction | Convolution | Scan => {
                r#"
                "GFLOPS: " + (result.flops() / 1_000_000_000).toFixed(3)
            "#
            }
            BufferSequential | BufferShuffled | BufferToBuffer
            | BufferToTexture | TextureToTexture => {
                r#"
                "Bandwidth (GB/s): " + (result.bandwidth() / 1_000_000_000).toFixed(3)
            "#
            }
        }
    }

    fn custom_result_function(&self) -> &'static str {
        match self {
            Matmul | Reduction | Convolution | Scan => "flops",
            BufferSequential | BufferShuffled | BufferToBuffer
            | BufferToTexture | TextureToTexture => "bandwidth",
        }
    }

    pub const fn path(&self) -> &'static str {
        match self {
            Matmul => "/matmul",
            Reduction => "/reduction",
            Convolution => "/convolution",
            Scan => "/scan",
            BufferSequential => "/buffer_sequential",
            BufferShuffled => "/buffer_shuffled",
            BufferToBuffer => "/buffer_to_buffer",
            BufferToTexture => "buffer_to_texture",
            TextureToTexture => "texture_to_texture",
        }
    }

    pub const fn title(&self) -> &'static str {
        match self {
            Matmul => "Matrix Multiplication",
            Reduction => "Reduction",
            Convolution => "Convolution",
            Scan => "Scan",
            BufferSequential => "Sequential Buffer Memory Access",
            BufferShuffled => "Shuffled Buffer Memory Accesses",
            BufferToBuffer => "Memory Copy From Buffer To Buffer",
            BufferToTexture => "Memory Copy From Buffer To Texture",
            TextureToTexture => "Memory Copy From Texture To Texture",
        }
    }
}
