//! This module implements any necessary conversions between types to WASM
//! variants and offers the WASM-compatible versions of the microbenchmarks

use wasm_bindgen::prelude::*;

use crate::convolution::{convolution_benchmark, ConvolutionResults};
use crate::matmul::{matmul_benchmark, MatmulResults};
use crate::memory::buffer_sequential::{
    buffer_sequential_benchmark, BufferSequentialResults,
};
use crate::reduction_sum::{reduction_sum_benchmark, ReductionSumResults};

#[wasm_bindgen(start)]
/// Entrypoint to instantiate the WASM module.
pub async fn setup() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}

/// WASM compatible version of [matmul_benchmark]
#[wasm_bindgen]
pub async fn wasm_matmul_benchmark(
    workgroup_size_x: u32,
    workgroup_size_y: u32,
) -> Result<MatmulResults, JsError> {
    Ok(matmul_benchmark((workgroup_size_x, workgroup_size_y)).await?)
}

/// WASM compatible version of [convolution_benchmark]
#[wasm_bindgen]
pub async fn wasm_convolution_benchmark(
    workgroup_size_x: u32,
    workgroup_size_y: u32,
) -> Result<ConvolutionResults, JsError> {
    Ok(convolution_benchmark((workgroup_size_x, workgroup_size_y)).await?)
}

/// WASM compatible version of [reduction_sum_benchmark]
#[wasm_bindgen]
pub async fn wasm_reduction_sum_benchmark(
    workgroup_size: u32,
) -> Result<ReductionSumResults, JsError> {
    Ok(reduction_sum_benchmark(workgroup_size).await?)
}

#[wasm_bindgen]
/// WASM compatible version of [buffer_sequential_benchmark]
pub async fn wasm_buffer_sequential_benchmark(
    workgroup_size: u32,
) -> Result<BufferSequentialResults, JsError> {
    Ok(buffer_sequential_benchmark(workgroup_size).await?)
}

/// Shadow println! when compiling to WASM
#[macro_export]
macro_rules! println {
        ($($t:tt)*) => (web_sys::console::log_1(&format_args!($($t)*).to_string().into()))
    }

/// Shadow eprintln! when compiling to WASM
#[macro_export]
macro_rules! eprintln {
        ($($t:tt)*) => (web_sys::console::error_1(&format_args!($($t)*).to_string().into()))
    }
