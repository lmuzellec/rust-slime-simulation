struct SizeSettings {
    width: u32,
    height: u32,
};

struct DiffuseSettings {
    decay_rate: f32,
    diffuse_rate: f32,
}

struct TimeBuffer {
    time: f32,
    delta_time: f32,
};

@group(0) @binding(0) var<uniform> size_settings: SizeSettings;
@group(0) @binding(1) var<uniform> diffuse_settings: DiffuseSettings;
@group(0) @binding(2) var<uniform> time: TimeBuffer;
@group(0) @binding(3) var trail_texture: texture_storage_2d<rgba16float, read>;
@group(0) @binding(4) var diffuse_texture: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(8, 8)
fn diffuse_main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    if (invocation_id.x < 0u || 
        invocation_id.x >= size_settings.width || 
        invocation_id.y < 0u || 
        invocation_id.y >= size_settings.height) {
        return;
    }
    
    let coords = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

    var sum: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    let original_col = textureLoad(trail_texture, coords);
    for (var offset_x: i32 = -1; offset_x <= 1; offset_x = offset_x + 1) {
        for (var offset_y: i32 = -1; offset_y <= 1; offset_y = offset_y + 1) {
            let sample_x = min(size_settings.width - 1u, max(0u, invocation_id.x + u32(offset_x)));
            let sample_y = min(size_settings.height - 1u, max(0u, invocation_id.y + u32(offset_y)));

            let offset_coords = vec2<i32>(i32(sample_x), i32(sample_y));
            let texture_state = textureLoad(trail_texture, offset_coords);
            sum = sum + texture_state;
        }
    }

    let blurred_col = sum / 9.0;
    let diffuse_weight = clamp(diffuse_settings.diffuse_rate * time.delta_time, 0.0, 1.0);
    let blurred_col = original_col * (1.0 - diffuse_weight) + blurred_col * diffuse_weight;

    let output = max(vec4<f32>(0.0, 0.0, 0.0, 0.0), blurred_col - diffuse_settings.decay_rate * time.delta_time);

    textureStore(diffuse_texture, coords, output);
}