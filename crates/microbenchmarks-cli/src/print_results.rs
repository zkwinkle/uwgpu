use microbenchmarks::{
    convolution::ConvolutionResults,
    matmul::MatmulResults,
    memcpy::{
        buffer_to_buffer::BufferToBufferResults,
        buffer_to_texture::BufferToTextureResults,
        texture_to_texture::TextureToTextureResults,
    },
    reduction_sum::ReductionSumResults,
    scan::ScanResults,
    uwgpu::{BenchmarkResults, TimeUnit},
};

/// Trait for implementing on the different benchmark results to simplify
/// their printing.
pub trait PrintableResults<const DIMS: usize> {
    fn microbenchmark_label(&self) -> String;
    fn print_info(&self);

    fn print_results(&self, workgroups: [u32; DIMS]) {
        print!(
            "\n{} microbenchmark [{}",
            self.microbenchmark_label(),
            workgroups[0]
        );
        for dim in &workgroups[1..] {
            print!("x{}", dim);
        }
        println!("]");
        println!("-----------------------------");
        self.print_info();
        println!("-----------------------------");
    }
}

impl PrintableResults<2> for MatmulResults {
    fn microbenchmark_label(&self) -> String {
        "Matrix Multiplication".to_string()
    }

    fn print_info(&self) { results_with_flops(&self.0, self.flops()) }
}

impl PrintableResults<2> for ConvolutionResults {
    fn microbenchmark_label(&self) -> String { "Convolution".to_string() }

    fn print_info(&self) { results_with_flops(&self.0, self.flops()) }
}

impl PrintableResults<1> for ReductionSumResults {
    fn microbenchmark_label(&self) -> String { "Reduction Sum".to_string() }

    fn print_info(&self) { results_with_flops(&self.0, self.flops()) }
}

impl PrintableResults<1> for ScanResults {
    fn microbenchmark_label(&self) -> String { "Reduction Sum".to_string() }

    fn print_info(&self) { results_with_flops(&self.0, self.flops()) }
}

impl PrintableResults<1> for BufferToBufferResults {
    fn microbenchmark_label(&self) -> String {
        "Copy Between Buffers".to_string()
    }

    fn print_info(&self) { results_with_bandwidth(&self.0, self.bandwidth()) }
}

impl PrintableResults<2> for BufferToTextureResults {
    fn microbenchmark_label(&self) -> String {
        "Copy From Buffer To Texture".to_string()
    }

    fn print_info(&self) { results_with_bandwidth(&self.0, self.bandwidth()) }
}

impl PrintableResults<2> for TextureToTextureResults {
    fn microbenchmark_label(&self) -> String {
        "Copy Between Textures".to_string()
    }

    fn print_info(&self) { results_with_bandwidth(&self.0, self.bandwidth()) }
}

fn results_with_flops(results: &BenchmarkResults, flops: f64) {
    println!(
        "Total time spent: {:.3}s",
        results.total_time(TimeUnit::Second)
    );
    println!(
        "Time per iteration: {:.3}ms",
        results.time_per_iteration(TimeUnit::Milli),
    );
    println!("GFLOPS: {:.3}", flops / 1_000_000_000.0);
}

fn results_with_bandwidth(results: &BenchmarkResults, bandwidth: f64) {
    println!(
        "Total time spent: {:.3}s",
        results.total_time(TimeUnit::Second)
    );
    println!(
        "Time per iteration: {:.4}ms",
        results.time_per_iteration(TimeUnit::Milli),
    );
    println!("Bandwidth (GB/s): {:.3}", bandwidth / 1_000_000_000.0);
}
