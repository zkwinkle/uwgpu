@group(0) @binding(0) var<storage, read> inputMatrix: array<f32>;
@group(0) @binding(1) var<storage, read> kernel: array<f32>;
@group(0) @binding(2) var<storage, read_write> result: array<f32>;
@group(0) @binding(3) var<uniform> matrixSize: u32;
@group(0) @binding(4) var<uniform> kernelSize: u32;

@compute @workgroup_size($workgroup$)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x: u32 = global_id.x;
    let y: u32 = global_id.y;

    if x >= matrixSize || y >= matrixSize {
        return;
    }

    var sum: f32 = 0.0;
    for (var ky = 0u; ky < kernelSize; ky++) {
        for (var kx = 0u; kx < kernelSize; kx++) {
						// kernel should be applied mirrored
            let i_x = x - (kx - kernelSize / 2);
            let i_y = y - (ky - kernelSize / 2);
						// Assume square matrix
            if i_x >= 0 && i_x < matrixSize && i_y >= 0 && i_y < matrixSize {
                let ki = ky * kernelSize + kx;
                let i = i_y * matrixSize + i_x;

                sum += inputMatrix[i] * kernel[ki];
            }
        }
    }
    result[y * matrixSize + x] = sum;
}
