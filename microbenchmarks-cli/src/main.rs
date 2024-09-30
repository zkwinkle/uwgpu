use clap::Parser;
use cli::{Cli, Microbenchmarks};
use microbenchmarks::{
    matmul::matmul_benchmark,
    memory::buffer_sequential::buffer_sequential_benchmark, BenchmarkError,
};

mod cli;
mod print_error;
mod print_results;

use print_error::print_error;
use print_results::PrintableResults;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if cli.microbenchmark.workgroups_empty() {
        eprintln!("No workgroup sizes specified so no benchmarks will be run. Specify workgroup sizes with -w flag.");
        return;
    }

    let result = run_microbenchmark(cli.microbenchmark).await;

    if let Err(err) = result {
        print_error(err);
    }
}

async fn run_microbenchmark(
    microbenchmark: Microbenchmarks,
) -> Result<(), BenchmarkError> {
    match microbenchmark {
        Microbenchmarks::MatMul(params) => {
            for wg in params.workgroup {
                let result = matmul_benchmark(&wg.into()).await?;
                result.print_results(wg);
            }
        }
        Microbenchmarks::BufferCopySequential(params) => {
            for wg in params.workgroup {
                let result = buffer_sequential_benchmark(wg[0]).await?;
                result.print_results(wg);
            }
        }
    }

    Ok(())
}
