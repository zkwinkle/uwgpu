@group(0) @binding(0) var copy_source: texture_2d<u32>;
@group(0) @binding(1) var copy_destination: texture_storage_2d<rgba8uint, write>;

@compute @workgroup_size($workgroup$)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;

    let dims = textureDimensions(copy_destination);

    if x < dims.x && y < dims.y {

        let pos = vec2<u32>(x, y);

        let texel = textureLoad(copy_source, pos, 0);

        textureStore(copy_destination, pos, texel);
    }
}
