use super::{
    types::Topology,
    util::{Texture2DVertex, Vertex},
    MeshType, Texture2DFormat, Texture2DDescriptor,
};

pub enum Mesh {
    Entity(Box<dyn EntityMesh>),
    Texture2D(Box<dyn Texture2DMesh>),
}

impl Mesh {
    pub fn vertex(&self) -> &[Vertex] {
        match self {
            Mesh::Entity(m) => m.vertex(),
            Mesh::Texture2D(_) => unreachable!(),
        }
    }

    pub fn index(&self) -> Option<&[u16]> {
        match self {
            Mesh::Entity(m) => m.index(),
            Mesh::Texture2D(m) => m.index(),
        }
    }

    pub fn texture(&self) -> Option<&[Texture2DVertex]> {
        match self {
            Mesh::Texture2D(m) => m.texture(),
            Mesh::Entity(_) => unreachable!(),
        }
    }

    pub fn mesh_type(&self) -> MeshType {
        match self {
            Mesh::Entity(_) => MeshType::Entity,
            Mesh::Texture2D(_) => MeshType::Texture2D,
        }
    }

    pub fn topology(&self) -> Topology {
        match self {
            Mesh::Entity(m) => m.topology(),
            Mesh::Texture2D(m) => m.topology(),
        }
    }
}

pub trait EntityMesh {
    fn vertex(&self) -> &[Vertex];
    fn index(&self) -> Option<&[u16]>;
    fn topology(&self) -> Topology {
        Default::default()
    }
    fn use_entity(self) -> Mesh;
}

pub trait Texture2DMesh: EntityMesh {
    fn texture(&self) -> Option<&[Texture2DVertex]>;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn bytes_per_pixel(&self) -> u32 {
        4
    }
    fn format(&self) -> &Texture2DFormat;
    fn data(&self) -> &[u8];
    fn use_texture2d(self, descriptor: Texture2DDescriptor) -> Mesh;
}
