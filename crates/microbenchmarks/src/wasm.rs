//! This module implements any necessary conversions between types to WASM
//! variants and offers the WASM-compatible versions of the microbenchmarks

use wasm_bindgen::prelude::*;

use crate::convolution::{convolution_benchmark, ConvolutionResults};
use crate::matmul::{matmul_benchmark, MatmulResults};
use crate::memcpy::buffer_to_buffer::{
    buffer_to_buffer_benchmark, BufferToBufferResults,
};
use crate::memcpy::buffer_to_texture::{
    buffer_to_texture_benchmark, BufferToTextureResults,
};
use crate::memcpy::texture_to_texture::{
    texture_to_texture_benchmark, TextureToTextureResults,
};
use crate::reduction_sum::{reduction_sum_benchmark, ReductionSumResults};
use crate::scan::{scan_benchmark, ScanResults};

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

/// WASM compatible version of [scan_benchmark]
#[wasm_bindgen]
pub async fn wasm_scan_benchmark(
    workgroup_size: u32,
) -> Result<ScanResults, JsError> {
    Ok(scan_benchmark(workgroup_size).await?)
}

#[wasm_bindgen]
/// WASM compatible version of [buffer_to_buffer_benchmark]
pub async fn wasm_buffer_to_buffer_benchmark(
    workgroup_size: u32,
) -> Result<BufferToBufferResults, JsError> {
    Ok(buffer_to_buffer_benchmark(workgroup_size).await?)
}

#[wasm_bindgen]
/// WASM compatible version of [buffer_to_texture_benchmark]
pub async fn wasm_buffer_to_texture_benchmark(
    workgroup_size_x: u32,
    workgroup_size_y: u32,
) -> Result<BufferToTextureResults, JsError> {
    Ok(
        buffer_to_texture_benchmark((workgroup_size_x, workgroup_size_y))
            .await?,
    )
}

#[wasm_bindgen]
/// WASM compatible version of [texture_to_texture_benchmark]
pub async fn wasm_texture_to_texture_benchmark(
    workgroup_size_x: u32,
    workgroup_size_y: u32,
) -> Result<TextureToTextureResults, JsError> {
    Ok(
        texture_to_texture_benchmark((workgroup_size_x, workgroup_size_y))
            .await?,
    )
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
