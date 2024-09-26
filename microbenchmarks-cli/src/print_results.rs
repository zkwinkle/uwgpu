use microbenchmarks::{
    matmul::MatmulResults, memory::buffer_sequential::BufferSequentialResults,
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

    fn print_info(&self) {
        println!("Total time spent: {:.3}s", self.total_time_s());
        println!("Time per iteration: {:.3}ms", self.time_per_iteration_ms(),);
        println!("FLOPS: {:.3e}", self.flops(),);
    }
}

impl PrintableResults<1> for BufferSequentialResults {
    fn microbenchmark_label(&self) -> String {
        "Sequential Copy Between Buffers".to_string()
    }

    fn print_info(&self) {
        println!("Total time spent: {:.3}s", self.total_time_s());
        println!("Time per iteration: {:.4}ms", self.time_per_iteration_ms(),);
        println!("Bandwidth (GB/s): {:.3}", self.gb_per_s());
    }
}
