@group(0) @binding(0) var<storage, read> matrixA: array<f32>;
@group(0) @binding(1) var<storage, read> matrixB: array<f32>;
@group(0) @binding(2) var<storage, read_write> result: array<f32>;
@group(0) @binding(3) var<uniform> matrixSize: u32;

@compute @workgroup_size($workgroup$)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let row: u32 = global_id.x;
    let col: u32 = global_id.y;
    let N = matrixSize;


    // Assume square matrix
    if row < N && col < N {
        var sum: f32 = 0.0;

        // Perform the dot product for row of A and column of B
        for (var k: u32 = 0; k < N; k = k + 1) {
            let a: f32 = matrixA[(row * N) + k];
            let b: f32 = matrixB[(k * N) + col];
            sum = sum + (a * b);
        }

        // Store the result in the result matrix
        result[(row * N) + col] = sum;
    }
}

