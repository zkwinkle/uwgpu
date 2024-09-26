use std::error::Error;

use clap::{command, Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about = "CLI tool for executing µwgpu microbenchmarks", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub microbenchmark: Microbenchmarks,
}

#[derive(Subcommand)]
#[command(name = "Microbenchmark")]
pub enum Microbenchmarks {
    /// Run the matmul microbenchmark
    MatMul(MicrobenchmarkParams<2>),
}

/// Common parameters shared by microbenchmarks
#[derive(Args)]
pub struct MicrobenchmarkParams<const DIMS: usize> {
    #[arg(short, long, value_parser = parse_array::<2, u32>)]
    pub workgroup: Vec<[u32; DIMS]>,
}

#[derive(Args)]
pub struct WorkgroupSize<const DIMS: usize> {}

/// Parse an array of values
fn parse_array<const SIZE: usize, T>(
    s: &str,
) -> Result<[T; SIZE], Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
{
    let array: Result<Vec<T>, _> =
        s.split(',').map(|array| array.parse::<T>()).collect();

    let array = array?;

    let array: [T; SIZE] = array.try_into().map_err(|v: Vec<_>| {
        format!(
            "invalid array: expected {} elements but found {}",
            SIZE,
            v.len()
        )
    })?;

    Ok(array)
}
