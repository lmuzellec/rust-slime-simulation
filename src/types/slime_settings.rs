use super::species_settings::SpeciesSettings;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SlimeSettings {
    pub num_agents: u32,
    pub trail_weight: f32,
    pub memory_offset_1: u32,
    pub memory_offset_2: u32,

    pub species_settings: [SpeciesSettings; 4],
}
