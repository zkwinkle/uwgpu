@group(0) @binding(0) var<storage, read> copy_source: array<u32>;
@group(0) @binding(1) var<storage, read_write> copy_destination: array<u32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index: u32 = global_id.x;
    copy_destination[index] = copy_source[index];
}

