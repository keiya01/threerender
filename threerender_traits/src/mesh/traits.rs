use std::{fmt::Debug, rc::Rc};

use super::{
    types::Topology,
    utils::{TextureVertex, Vertex},
    MeshType, TextureFormat,
};

#[derive(Debug, Clone)]
pub enum Mesh {
    Entity(Rc<dyn EntityMesh>),
    Texture(Rc<dyn TextureMesh>),
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

/// Define an entity. Entity will be used to draw mesh.
pub trait EntityMesh: Debug {
    /// Required to return vertices.
    fn vertex(&self) -> &[Vertex];
    /// Define indices to draw an entity more efficiently.
    fn index(&self) -> Option<&[u16]>;
    /// Set topology type. Default is `TriangleList`.
    fn topology(&self) -> Topology {
        Default::default()
    }
    /// Make mesh from entity.
    fn use_entity(self) -> Mesh
    where
        Self: Sized + 'static,
    {
        Mesh::Entity(Rc::new(self))
    }
}

/// Define an entity that has a texture.
pub trait TextureMesh: Debug + EntityMesh {
    /// Define vertex for texture
    fn texture(&self) -> Option<&[TextureVertex]>;
    /// Texture width
    fn width(&self) -> u32;
    /// Texture height
    fn height(&self) -> u32;
    /// Texture bytes per pixel
    fn bytes_per_pixel(&self) -> u32 {
        4
    }
    /// Texture format
    fn format(&self) -> &TextureFormat;
    /// Texture data
    fn data(&self) -> &[u8];
    /// Make mesh from texture entity.
    fn use_texture(self) -> Mesh
    where
        Self: Sized + 'static,
    {
        Mesh::Texture(Rc::new(self))
    }
}
