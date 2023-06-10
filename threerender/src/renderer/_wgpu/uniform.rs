use bytemuck::{Pod, Zeroable};

use super::scene::Reflection;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct EntityUniformBuffer {
    pub(super) transform: [[f32; 4]; 4],
    pub(super) normal_transform: [[f32; 4]; 4],
    pub(super) color: [f32; 4],
    // First value is used
    pub(super) tex_idx: [i32; 4],
    // First value is used
    pub(super) normal_idx: [i32; 4],
    pub(super) receive_shadow: [u32; 4],
    pub(super) reflection: Reflection,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct ShadowEntityUniformBuffer {
    pub(super) transform: [[f32; 4]; 4],
}
