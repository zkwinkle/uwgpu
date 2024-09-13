use microbenchmarks::matmul::matmul_benchmark;

#[tokio::main]
async fn main() {
    println!("\nExecuting matmul benchmark...");
    println!("-----------------------------");

    let results = matmul_benchmark().await.unwrap();

    println!("Total time spent: {:.3}s", results.total_time_s());
    println!(
        "Time per iteration: {:.3}ms",
        results.time_per_iteration_ms(),
    );
    println!("FLOPS: {:.3e}", results.flops(),);

    println!("-----------------------------");
}
