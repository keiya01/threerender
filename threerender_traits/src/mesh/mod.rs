mod traits;
mod types;
mod utils;

use std::{cell::RefCell, rc::Rc};

pub use traits::*;
pub use types::*;
pub use utils::*;

#[derive(Debug)]
pub struct DefaultMesh;
impl Mesh for DefaultMesh {
    fn vertex(&self) -> Rc<RefCell<Vec<Vertex>>> {
        Rc::new(RefCell::new(vec![]))
    }
    fn index(&self) -> Option<&[u16]> {
        None
    }
}
