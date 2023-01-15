use bytemuck::{Pod, Zeroable};
use glam::Vec3;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pos: [f32; 4],
    normal: [f32; 3],
}

impl Vertex {
    pub fn from_vec3(pos: Vec3, normal: Vec3) -> Vertex {
        Vertex {
            pos: [pos.x, pos.y, pos.z, 1.],
            normal: [normal.x, normal.y, normal.z],
        }
    }
}

pub fn vertex(pos: [f32; 4], normal: [f32; 3]) -> Vertex {
    Vertex { pos, normal }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct TextureVertex {
    pos: [f32; 4],
    normal: [f32; 3],
    tex: [f32; 2],
}

pub fn texture(tex: [f32; 2]) -> [f32; 2] {
    tex
}

pub fn texture_vertex(vertex: Vertex, tex: [f32; 2]) -> TextureVertex {
    TextureVertex {
        pos: vertex.pos,
        normal: vertex.normal,
        tex,
    }
}
