struct SizeSettings {
    width: u32,
    height: u32,
};

struct Agent {
    position: vec2<f32>,
    angle: f32,
    species_index: u32,
};

struct Agents {
    agents: array<Agent>,
};

struct SpeciesSetting {
    move_speed: f32,
    turn_speed: f32,

    sensor_angle_spacing: f32,
    sensor_offset_dst: f32,
    @size(16) sensor_size: u32,
};

struct SlimeSettings {
    num_agents: u32,
    @size(12) trail_weight: f32,

    species_settings: array<SpeciesSetting, 4>,// offset(16) align(16) size(32 * 4) stride(32)
    // offset must be multiple of 16
    // stride must be multiple of 16
    // https://www.w3.org/TR/WGSL/#address-space-layout-constraints
};

@group(0) @binding(0) var<uniform> size_settings: SizeSettings;
@group(0) @binding(1) var<uniform> slime_settings: SlimeSettings;
@group(0) @binding(2) var<storage, read_write> agents: Agents;
@group(0) @binding(3) var texture_read: texture_storage_2d<rgba16float, read>;
@group(0) @binding(4) var texture_write: texture_storage_2d<rgba16float, write>;

fn draw_sense(agent: Agent, species_setting: SpeciesSetting, sensor_angle_offset: f32, color: vec4<f32>) {
    let sensor_angle = agent.angle + sensor_angle_offset;
    let sensor_dir = vec2<f32>(cos(sensor_angle), sin(sensor_angle));

    let sensor_pos = agent.position + sensor_dir * species_setting.sensor_offset_dst;

    let sensor_pos_x = i32(sensor_pos.x);
    let sensor_pos_y = i32(sensor_pos.y);

    var offset_x: i32;
    var offset_y: i32;

    for (offset_x = i32(species_setting.sensor_size) * -1; offset_x <= i32(species_setting.sensor_size); offset_x = offset_x + 1) {
        for (offset_y = i32(species_setting.sensor_size) * -1; offset_y <= i32(species_setting.sensor_size); offset_y = offset_y + 1) {
            let sample_x = min(i32(size_settings.width) - 1, max(0, sensor_pos_x + offset_x));
            let sample_y = min(i32(size_settings.height) - 1, max(0, sensor_pos_y + offset_y));

            textureStore(texture_write, vec2<i32>(sample_x, sample_y), color);
        }
    }
}

@compute @workgroup_size(64, 1, 1)
fn draw_sensor_main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {

    let total_agents = arrayLength(&agents.agents);
    let agent_index = invocation_id.x;

    if (agent_index >= total_agents) {
        return;
    }

    var agent: Agent = agents.agents[agent_index];
    var species_setting: SpeciesSetting = slime_settings.species_settings[agent.species_index];

    let sensor_angle = species_setting.sensor_angle_spacing * (3.1415 / 180.0);
    draw_sense(agent, species_setting, sensor_angle, vec4<f32>(1.0, 0.0, 0.0, 1.0));
    draw_sense(agent, species_setting, 0.0, vec4<f32>(0.0, 1.0, 0.0, 1.0));
    draw_sense(agent, species_setting, -sensor_angle, vec4<f32>(0.0, 0.0, 1.0, 1.0));
}
