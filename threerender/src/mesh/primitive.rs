use super::{types::MeshType, util::Vertex};

pub trait Primitive {
    fn vertex(&self) -> &[Vertex];
    fn index(&self) -> Option<&[u16]>;
    fn mesh_type(&self) -> MeshType {
        Default::default()
    }
}
