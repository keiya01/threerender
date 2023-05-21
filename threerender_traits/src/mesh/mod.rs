mod traits;
mod types;
mod utils;

pub use traits::*;
pub use types::*;
pub use utils::*;

pub struct DefaultMesh;
impl EntityMesh for DefaultMesh {
    fn vertex(&self) -> &[Vertex] {
        &[]
    }
    fn index(&self) -> Option<&[u16]> {
        None
    }
}
