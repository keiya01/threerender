use std::{cell::RefCell, rc::Rc};

use threerender_math::Vec3;
use threerender_traits::mesh::{Mesh, Vertex};

#[derive(Debug)]
pub struct Polygon {
    vertex: Rc<RefCell<Vec<Vertex>>>,
    index: Option<Vec<u16>>,
}

impl Polygon {
    pub fn new(a: Vec3, b: Vec3, c: Vec3) -> Self {
        let normal = Vec3::new(0., 0., 1.0);
        Self {
            vertex: Rc::new(RefCell::new(vec![
                Vertex::from_vec3(a, normal),
                Vertex::from_vec3(b, normal),
                Vertex::from_vec3(c, normal),
            ])),
            index: None,
        }
    }

    pub fn push_vertex(&mut self, a: Vertex, b: Vertex, c: Vertex) {
        self.vertex.borrow_mut().extend_from_slice(&[a, b, c]);
    }

    pub fn push_index(&mut self, v: [u16; 3]) {
        match &mut self.index {
            Some(idx) => idx.extend_from_slice(&v),
            None => self.index = Some(v.to_vec()),
        }
    }
}

impl Mesh for Polygon {
    fn vertex(&self) -> Rc<RefCell<Vec<Vertex>>> {
        self.vertex.clone()
    }

    fn index(&self) -> Option<&[u16]> {
        match &self.index {
            Some(idx) => Some(idx),
            None => None,
        }
    }
}
