#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SizeSettings {
    pub width: u32,
    pub height: u32,
}
