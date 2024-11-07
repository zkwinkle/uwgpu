@group(0) @binding(0) var<storage, read_write> data: array<f32>;

@group(1) @binding(0) var<uniform> stride: u32;

// Strategy: Sklansky
// See https://user-images.githubusercontent.com/68340554/224912079-b1580955-b702-45f9-887a-7c1003825bf9.png

// So threads are grouped into blocks, each block of threads gets assigned an
// index `i` of `data`.
// Each thread must assign `data[i+block_i+1] += data[i]`.
// Where:
//
// - `i` = (global_id / (stride/2)) * stride + (stride/2)
// - `block_i` = global_id % (stride/2)

// To avoid bounds checks in the shader, the `data` array should be a multiple
// of the workgroup size, 0-padded if necessary.

@compute @workgroup_size($workgroup$)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let thread_id = global_id.x;

    let stride_half = stride / 2;
    let offset = stride_half-1;

    let i = (thread_id / (stride_half)) * stride + offset;
    let block_i = thread_id % (stride_half);

    data[i + block_i + 1] += data[i];
}
