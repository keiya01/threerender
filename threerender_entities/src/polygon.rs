use threerender_math::Vec3;
use threerender_traits::mesh::{EntityMesh, Mesh, Vertex};

pub struct Polygon {
    vertex: Vec<Vertex>,
    index: Option<Vec<u16>>,
}

impl Polygon {
    pub fn new(a: Vec3, b: Vec3, c: Vec3) -> Self {
        let normal = Vec3::new(0., 0., 1.0);
        Self {
            vertex: vec![
                Vertex::from_vec3(a, normal),
                Vertex::from_vec3(b, normal),
                Vertex::from_vec3(c, normal),
            ],
            index: None,
        }
    }

    pub fn push_vertex(&mut self, a: Vertex, b: Vertex, c: Vertex) {
        self.vertex.extend_from_slice(&[a, b, c]);
    }

    pub fn push_index(&mut self, v: [u16; 3]) {
        match &mut self.index {
            Some(idx) => idx.extend_from_slice(&v),
            None => self.index = Some(v.to_vec()),
        }
    }
}

impl EntityMesh for Polygon {
    fn vertex(&self) -> &[Vertex] {
        &self.vertex
    }

    fn index(&self) -> Option<&[u16]> {
        match &self.index {
            Some(idx) => Some(idx),
            None => None,
        }
    }

    fn use_entity(self) -> Mesh {
        Mesh::Entity(Box::new(self))
    }
}
