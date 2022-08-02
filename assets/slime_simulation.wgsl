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

struct TimeBuffer {
    time: f32,
    delta_time: f32,
};

@group(0) @binding(0) var<uniform> size_settings: SizeSettings;
@group(0) @binding(1) var<uniform> slime_settings: SlimeSettings;
@group(0) @binding(2) var<uniform> time: TimeBuffer;
@group(0) @binding(3) var<storage, read_write> agents: Agents;
@group(0) @binding(4) var texture_read: texture_storage_2d<rgba16float, read>;
@group(0) @binding(5) var texture_write: texture_storage_2d<rgba16float, write>;

fn hash(value: u32) -> u32 {
    var state = value;
    state = state ^ 2747636419u;
    state = state * 2654435769u;
    state = state ^ state >> 16u;
    state = state * 2654435769u;
    state = state ^ state >> 16u;
    state = state * 2654435769u;
    return state;
}

fn scale_to_range(state: u32) -> f32 {
    return f32(state) / 4294967295.0;
}

fn sense(agent: Agent, species_setting: SpeciesSetting, sensor_angle_offset: f32) -> f32 {
    let sensor_angle = agent.angle + sensor_angle_offset;
    let sensor_dir = vec2<f32>(cos(sensor_angle), sin(sensor_angle));

    let sensor_pos = agent.position + sensor_dir * species_setting.sensor_offset_dst;

    let sensor_pos_x = i32(sensor_pos.x);
    let sensor_pos_y = i32(sensor_pos.y);

    var sum: f32;
    var offset_x: i32;
    var offset_y: i32;

    for (offset_x = i32(species_setting.sensor_size) * -1; offset_x <= i32(species_setting.sensor_size); offset_x = offset_x + 1) {
        for (offset_y = i32(species_setting.sensor_size) * -1; offset_y <= i32(species_setting.sensor_size); offset_y = offset_y + 1) {
            let sample_x = min(i32(size_settings.width) - 1, max(0, sensor_pos_x + offset_x));
            let sample_y = min(i32(size_settings.height) - 1, max(0, sensor_pos_y + offset_y));

            let current_map = textureLoad(texture_read, vec2<i32>(sample_x, sample_y));
            // TODO use a species mask
            let mask = vec4<f32>(1.0, 1.0, 1.0, 1.0) * 2.0 - 1.0;
            sum = sum + dot(mask, current_map);
        }
    }

    return sum;
}

@compute @workgroup_size(64, 1, 1)
fn slime_main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {

    let total_agents = arrayLength(&agents.agents);
    let agent_index = invocation_id.x;

    if (agent_index >= total_agents) {
        return;
    }

    var agent: Agent = agents.agents[agent_index];
    var species_setting: SpeciesSetting = slime_settings.species_settings[agent.species_index];

    let random = hash(
        u32(agent.position.y) * size_settings .width
            + u32(agent.position.x)
            + hash(u32(f32(agent_index) + time.time * 100000.0))
    );

    let sensor_angle = species_setting.sensor_angle_spacing * (3.1415 / 180.0);
    let weight_forward = sense(agent, species_setting, 0.0);
    let weight_left    = sense(agent, species_setting, sensor_angle);
    let weight_right   = sense(agent, species_setting, -sensor_angle);

    let random_steer_strength =  scale_to_range(random);
    let turn_speed = species_setting.turn_speed * 3.1415 * 2.0;

    if (weight_forward >= weight_left && weight_forward >= weight_right) {
        agents.agents[agent_index].angle = agent.angle + 0.0;
    } else if (weight_forward < weight_left && weight_forward < weight_right) {
        agents.agents[agent_index].angle = agent.angle + (random_steer_strength - 0.5) * 2.0 * turn_speed * time.delta_time;
    } else if (weight_right > weight_left) {
        agents.agents[agent_index].angle = agent.angle - random_steer_strength * turn_speed * time.delta_time;
    } else if (weight_left > weight_right) {
        agents.agents[agent_index].angle = agent.angle + random_steer_strength * turn_speed * time.delta_time;
    }

    let direction = vec2<f32>(cos(agent.angle), sin(agent.angle));
    var new_pos: vec2<f32> = agent.position + direction * species_setting.move_speed * time.delta_time;

    if (new_pos.x < 0.0 || new_pos.x > f32(size_settings.width) || new_pos.y < 0.0 || new_pos.y > f32(size_settings.height)) {
        let new_rand = hash(random);
        let random_angle = scale_to_range(new_rand) * 3.1415 * 2.0;

        new_pos.x = min(f32(size_settings.width) - 1.0, max(0.0, new_pos.x));
        new_pos.y = min(f32(size_settings.height) - 1.0, max(0.0, new_pos.y));
        agents.agents[agent_index].angle = random_angle;
    } else {
        let current_pos = vec2<i32>(i32(new_pos.x), i32(new_pos.y));
        // TODO use old_map with species mask
        // let current_map = textureLoad(texture, current_pos);

        textureStore(texture_write, current_pos, vec4<f32>(1.0, 1.0, 1.0, 1.0) * slime_settings.trail_weight);
    }

    agents.agents[agent_index].position = new_pos;
}
