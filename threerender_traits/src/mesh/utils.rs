use bytemuck::{Pod, Zeroable};
use threerender_math::Vec3;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub pos: [f32; 4],
    pub normal: [f32; 3],
    pub tex: [f32; 2],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
}

impl Vertex {
    pub fn from_vec3(pos: Vec3, normal: Vec3) -> Vertex {
        Vertex {
            pos: [pos.x, pos.y, pos.z, 1.],
            normal: [normal.x, normal.y, normal.z],
            tex: [0., 0.],
            tangent: [0., 0., 0.],
            bitangent: [0., 0., 0.],
        }
    }
}

pub fn vertex(pos: [f32; 4], normal: [f32; 3]) -> Vertex {
    Vertex {
        pos,
        normal,
        tex: [0., 0.],
        tangent: [0., 0., 0.],
        bitangent: [0., 0., 0.],
    }
}

pub fn texture(tex: [f32; 2]) -> [f32; 2] {
    tex
}

pub fn texture_vertex(vertex: Vertex, tex: [f32; 2]) -> Vertex {
    Vertex {
        pos: vertex.pos,
        normal: vertex.normal,
        tex,
        tangent: [0., 0., 0.],
        bitangent: [0., 0., 0.],
    }
}
