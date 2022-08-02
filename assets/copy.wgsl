struct SizeSettings {
    width: u32,
    height: u32,
};

@group(0) @binding(0) var<uniform> size_settings: SizeSettings;
@group(0) @binding(1) var texture_view_read: texture_storage_2d<rgba16float, read>;
@group(0) @binding(2) var texture_view_write: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(8, 8)
fn copy_main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    if (invocation_id.x < 0u || 
        invocation_id.x >= size_settings.width || 
        invocation_id.y < 0u || 
        invocation_id.y >= size_settings.height) {
        return;
    }

    let coords = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

    let texture_state = textureLoad(texture_view_read, coords);

    textureStore(texture_view_write, coords, texture_state);
}