use super::{
    types::Topology,
    util::{TextureVertex, Vertex},
    MeshType, TextureDescriptor, TextureFormat,
};

pub enum Mesh {
    Entity(Box<dyn EntityMesh>),
    Texture(Box<dyn TextureMesh>),
}

impl Mesh {
    pub fn vertex(&self) -> &[Vertex] {
        match self {
            Mesh::Entity(m) => m.vertex(),
            Mesh::Texture(_) => unreachable!(),
        }
    }

    pub fn index(&self) -> Option<&[u16]> {
        match self {
            Mesh::Entity(m) => m.index(),
            Mesh::Texture(m) => m.index(),
        }
    }

    pub fn texture(&self) -> Option<&[TextureVertex]> {
        match self {
            Mesh::Texture(m) => m.texture(),
            Mesh::Entity(_) => unreachable!(),
        }
    }

    pub fn mesh_type(&self) -> MeshType {
        match self {
            Mesh::Entity(_) => MeshType::Entity,
            Mesh::Texture(_) => MeshType::Texture,
        }
    }

    pub fn topology(&self) -> Topology {
        match self {
            Mesh::Entity(m) => m.topology(),
            Mesh::Texture(m) => m.topology(),
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

pub trait TextureMesh: EntityMesh {
    fn texture(&self) -> Option<&[TextureVertex]>;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn bytes_per_pixel(&self) -> u32 {
        4
    }
    fn format(&self) -> &TextureFormat;
    fn data(&self) -> &[u8];
    fn use_texture(self, descriptor: TextureDescriptor) -> Mesh;
}
