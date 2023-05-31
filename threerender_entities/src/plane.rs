use std::{cell::RefCell, rc::Rc};

use threerender_traits::mesh::{texture, vertex, Mesh, Vertex};

use crate::BuiltInEntityOption;

#[derive(Debug)]
pub struct Plane {
    vertex: Rc<RefCell<Vec<Vertex>>>,
    index: [u16; 6],
}

impl Plane {
    pub fn new(normal: [i8; 3], options: Option<BuiltInEntityOption>) -> Self {
        let mat: [[i8; 3]; 4] = match &normal {
            [1, 0, 0] => [[0, -1, 1], [0, -1, -1], [0, 1, -1], [0, 1, 1]],
            [-1, 0, 0] => [[0, -1, -1], [0, -1, 1], [0, 1, 1], [0, 1, -1]],
            [0, 1, 0] => [[-1, 0, -1], [-1, 0, 1], [1, 0, 1], [1, 0, -1]],
            [0, -1, 0] => [[-1, 0, -1], [1, 0, -1], [1, 0, 1], [-1, 0, 1]],
            [0, 0, 1] => [[-1, -1, 0], [1, -1, 0], [1, 1, 0], [-1, 1, 0]],
            [0, 0, -1] => [[1, -1, 0], [-1, -1, 0], [-1, 1, 0], [1, 1, 0]],
            _ => unimplemented!(),
        };

        let normal = normal.map(|v| v as f32);

        let mut vertex = mat
            .map(|v| vertex([v[0] as f32, v[1] as f32, v[2] as f32, 1.], normal))
            .to_vec();

        if options.map(|v| v.use_texture).unwrap_or_default() {
            let texs = vec![
                texture([0., 1.]),
                texture([1., 1.]),
                texture([1., 0.]),
                texture([0., 0.]),
            ];

            // TODO: use VecDequeue
            for (idx, vert) in vertex.iter_mut().enumerate() {
                let tex = *texs.get(idx).expect("`texs` length is incorrect");
                vert.tex = tex;
            }
        }

        Self {
            vertex: Rc::new(RefCell::new(vertex)),
            index: [0, 1, 2, 0, 2, 3],
        }
    }
}

impl Mesh for Plane {
    fn vertex(&self) -> Rc<RefCell<Vec<Vertex>>> {
        self.vertex.clone()
    }

    fn index(&self) -> Option<&[u16]> {
        Some(&self.index)
    }
}
