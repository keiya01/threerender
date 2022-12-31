use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct EntityUniformBuffer {
    pub(super) transform: [[f32; 4]; 4],
    pub(super) color: [f32; 4],
}
