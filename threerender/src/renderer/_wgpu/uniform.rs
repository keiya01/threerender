use bytemuck::{Pod, Zeroable};

use super::scene::Reflection;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct EntityUniformBuffer {
    pub(super) transform: [[f32; 4]; 4],
    pub(super) color: [f32; 4],
    pub(super) reflection: Reflection,
    pub(super) tex_idx: i32,
    pub(super) padding: [f32; 3],
}
