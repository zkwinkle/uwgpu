use clap::Parser;
use cli::{Cli, Microbenchmarks};
use microbenchmarks::{
    convolution::convolution_benchmark,
    matmul::matmul_benchmark,
    memcpy::{
        buffer_to_buffer::buffer_to_buffer_benchmark,
        buffer_to_texture::buffer_to_texture_benchmark,
        texture_to_texture::texture_to_texture_benchmark,
    },
    reduction_sum::reduction_sum_benchmark,
    scan::scan_benchmark,
    BenchmarkError,
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
                let result = matmul_benchmark(wg.into()).await?;
                result.print_results(wg);
            }
        }
        Microbenchmarks::Convolution(params) => {
            for wg in params.workgroup {
                let result = convolution_benchmark(wg.into()).await?;
                result.print_results(wg);
            }
        }
        Microbenchmarks::Scan(params) => {
            for wg in params.workgroup {
                let result = scan_benchmark(wg[0]).await?;
                result.print_results(wg);
            }
        }
        Microbenchmarks::Reduction(params) => {
            for wg in params.workgroup {
                let result = reduction_sum_benchmark(wg[0]).await?;
                result.print_results(wg);
            }
        }
        Microbenchmarks::BufferToBuffer(params) => {
            for wg in params.workgroup {
                let result = buffer_to_buffer_benchmark(wg[0]).await?;
                result.print_results(wg);
            }
        }
        Microbenchmarks::BufferToTexture(params) => {
            for wg in params.workgroup {
                let result = buffer_to_texture_benchmark(wg.into()).await?;
                result.print_results(wg);
            }
        }
        Microbenchmarks::TextureToTexture(params) => {
            for wg in params.workgroup {
                let result = texture_to_texture_benchmark(wg.into()).await?;
                result.print_results(wg);
            }
        }
    }

    Ok(())
}
