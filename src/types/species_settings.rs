#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpeciesSettings {
    pub move_speed: f32,
    pub turn_speed: f32,

    pub sensor_angle_spacing: f32,
    pub sensor_offset_dst: f32,
    pub sensor_size: u32,
    pub memory_offset_1: u32,
    pub memory_offset_2: u32,
    pub memory_offset_3: u32,
}

impl Default for SpeciesSettings {
    fn default() -> Self {
        Self {
            move_speed: 50.0,
            turn_speed: 10.0,
            sensor_angle_spacing: 30.0,
            sensor_offset_dst: 5.0,
            sensor_size: 1,
            memory_offset_1: 0,
            memory_offset_2: 0,
            memory_offset_3: 0,
        }
    }
}
