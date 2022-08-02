#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TimeBuffer {
    pub time: f32,
    pub delta_time: f32,
}
