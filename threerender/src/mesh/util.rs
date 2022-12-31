use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pos: [f32; 4],
    normal: [f32; 3],
}

pub(super) fn vertex(pos: [f32; 4], normal: [f32; 3]) -> Vertex {
    Vertex { pos, normal }
}
