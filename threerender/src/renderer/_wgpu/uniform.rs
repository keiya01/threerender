use bytemuck::{Pod, Zeroable};

use super::scene::Reflection;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct EntityUniformBuffer {
    pub(super) transform: [[f32; 4]; 4],
    pub(super) color: [f32; 4],
    pub(super) reflection: Reflection,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct TextureInfoUniformBuffer {
    pub(super) idx: u32,
}
