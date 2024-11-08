@group(0) @binding(0) var<storage, read> copy_source: array<u32>;
@group(0) @binding(1) var copy_destination: texture_storage_2d<rgba8uint, write>;

@compute @workgroup_size($workgroup$)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;

    let dims = textureDimensions(copy_destination);

    if x < dims.x && y < dims.y {
        let idx = y * dims.x + x;

        let pixel = vec4<u32>(
            copy_source[idx] & 0xFFu,
            (copy_source[idx] >> 8) & 0xFFu,
            (copy_source[idx] >> 16) & 0xFFu,
            (copy_source[idx] >> 24) & 0xFFu,
        );

        textureStore(copy_destination, vec2<u32>(x, y), pixel);
    }
}
