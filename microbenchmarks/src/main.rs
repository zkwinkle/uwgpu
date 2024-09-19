use microbenchmarks::{
    matmul::matmul_benchmark,
    memory::buffer_sequential::buffer_sequential_benchmark,
};

#[tokio::main]
async fn main() {
    println!("\nExecuting matmul benchmark...");
    println!("-----------------------------");

    let results = matmul_benchmark(&(8, 8)).await.unwrap();

    println!("Total time spent: {:.3}s", results.total_time_s());
    println!(
        "Time per iteration: {:.3}ms",
        results.time_per_iteration_ms(),
    );
    println!("FLOPS: {:.3e}", results.flops(),);

    println!("-----------------------------");

    println!("\nExecuting sequential copies benchmark...");
    println!("-----------------------------");

    let results = buffer_sequential_benchmark(64).await.unwrap();

    println!("Total time spent: {:.3}s", results.total_time_s());
    println!(
        "Time per iteration: {:.4}ms",
        results.time_per_iteration_ms(),
    );
    println!("Bandwidth (GB/s): {:.3}", results.gb_per_s());

    println!("-----------------------------");
}
