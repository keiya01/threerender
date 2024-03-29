use std::{cell::RefCell, rc::Rc};

use threerender_math::Vec3;
use threerender_traits::mesh::{Mesh, Topology, Vertex};

#[derive(Debug)]
pub struct Point {
    vertex: Rc<RefCell<Vec<Vertex>>>,
    index: Option<Vec<u16>>,
    pub topology: Topology,
}

impl Point {
    pub fn new(points: Vec<Vec3>) -> Self {
        Self {
            vertex: Rc::new(RefCell::new(Self::vec_to_vertex(points))),
            index: None,
            topology: Topology::PointList,
        }
    }

    pub fn push_vertex(&mut self, points: Vec<Vec3>) {
        self.vertex
            .borrow_mut()
            .extend_from_slice(&Self::vec_to_vertex(points));
    }

    pub fn push_index(&mut self, v: [u16; 3]) {
        match &mut self.index {
            Some(idx) => idx.extend_from_slice(&v),
            None => self.index = Some(v.to_vec()),
        }
    }

    fn vec_to_vertex(points: Vec<Vec3>) -> Vec<Vertex> {
        let normal = Vec3::new(0., 0., 1.0);
        points
            .into_iter()
            .map(|p| Vertex::from_vec3(p, normal))
            .collect()
    }
}

impl Mesh for Point {
    fn vertex(&self) -> Rc<RefCell<Vec<Vertex>>> {
        self.vertex.clone()
    }

    fn index(&self) -> Option<&[u16]> {
        match &self.index {
            Some(idx) => Some(idx),
            None => None,
        }
    }

    fn topology(&self) -> Topology {
        Topology::PointList
    }
}
