use super::util::Vertex;

pub trait Primitive {
    fn vertex(&self) -> &[Vertex];
    fn index(&self) -> Option<&[u16]>;
}
