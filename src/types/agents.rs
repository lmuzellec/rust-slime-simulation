#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Agent {
    pub position: [f32; 2],
    pub angle: f32,
    pub species_index: u32,
}

impl Default for Agent {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0],
            angle: 0.0,
            species_index: 0,
        }
    }
}
