#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DiffuseSettings {
    pub decay_rate: f32,
    pub diffuse_rate: f32,
}
