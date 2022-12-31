use super::{
    primitive::Primitive,
    util::{vertex, Vertex},
};

pub struct Triangle {
    vertex: Vec<Vertex>,
}

impl Triangle {
    pub fn new() -> Self {
        Self {
            vertex: vec![
                vertex([-1., -1., 1., 1.], [0., 0., 1.]),
                vertex([1., -1., 1., 1.], [0., 0., 1.]),
                vertex([0., 1., 1., 1.], [0., 0., 1.]),
            ],
        }
    }
}

impl Primitive for Triangle {
    fn vertex(&self) -> &[Vertex] {
        &self.vertex
    }

    fn index(&self) -> Option<&[u16]> {
        None
    }
}

pub struct Quadrangle {
    vertex: Vec<Vertex>,
}

impl Quadrangle {
    pub fn new() -> Self {
        Self {
            vertex: vec![
                // Half triangle
                vertex([-1., -1., 1., 1.], [0., 0., 1.]),
                vertex([1., -1., 1., 1.], [0., 0., 1.]),
                vertex([1., 1., 1., 1.], [0., 0., 1.]),
                // Half triangle
                vertex([-1., -1., 1., 1.], [0., 0., 1.]),
                vertex([1., 1., 1., 1.], [0., 0., 1.]),
                vertex([-1., 1., 1., 1.], [0., 0., 1.]),
            ],
        }
    }
}

impl Primitive for Quadrangle {
    fn vertex(&self) -> &[Vertex] {
        &self.vertex
    }

    fn index(&self) -> Option<&[u16]> {
        None
    }
}

pub struct Square {
    vertex: Vec<Vertex>,
    index: Vec<u16>,
}

impl Square {
    pub fn new() -> Self {
        Self {
            vertex: vec![
                // Left
                vertex([-1., -1., -1., 1.], [-1., 0., 0.]),
                vertex([-1., -1., 1., 1.], [-1., 0., 0.]),
                vertex([-1., 1., 1., 1.], [-1., 0., 0.]),
                vertex([-1., 1., -1., 1.], [-1., 0., 0.]),
                // Back
                vertex([1., -1., -1., 1.], [0., 0., -1.]),
                vertex([-1., -1., -1., 1.], [0., 0., -1.]),
                vertex([-1., 1., -1., 1.], [0., 0., -1.]),
                vertex([1., 1., -1., 1.], [0., 0., -1.]),
                // Bottom
                vertex([-1., -1., -1., 1.], [0., -1., 0.]),
                vertex([1., -1., -1., 1.], [0., -1., 0.]),
                vertex([1., -1., 1., 1.], [0., -1., 0.]),
                vertex([-1., -1., 1., 1.], [0., -1., 0.]),
                // Right
                vertex([1., -1., 1., 1.], [1., 0., 0.]),
                vertex([1., -1., -1., 1.], [1., 0., 0.]),
                vertex([1., 1., -1., 1.], [1., 0., 0.]),
                vertex([1., 1., 1., 1.], [1., 0., 0.]),
                // Top
                vertex([-1., 1., -1., 1.], [0., 1., 0.]),
                vertex([-1., 1., 1., 1.], [0., 1., 0.]),
                vertex([1., 1., 1., 1.], [0., 1., 0.]),
                vertex([1., 1., -1., 1.], [0., 1., 0.]),
                // Front
                vertex([-1., -1., 1., 1.], [0., 0., 1.]),
                vertex([1., -1., 1., 1.], [0., 0., 1.]),
                vertex([1., 1., 1., 1.], [0., 0., 1.]),
                vertex([-1., 1., 1., 1.], [0., 0., 1.]),
            ],
            #[rustfmt::skip]
            index: vec![
                // Left
                0, 1, 2, 0, 2, 3,
                // Back
                4, 5, 6, 4, 6, 7,
                // Bottom
                8, 9, 10, 8, 10, 11,
                // Right
                12, 13, 14, 12, 14, 15,
                // Top
                16, 17, 18, 16, 18, 19,
                // Front
                20, 21, 22, 20, 22, 23,
            ],
        }
    }
}

impl Primitive for Square {
    fn vertex(&self) -> &[Vertex] {
        &self.vertex
    }

    fn index(&self) -> Option<&[u16]> {
        Some(&self.index)
    }
}
