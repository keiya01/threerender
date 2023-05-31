use std::{cell::RefCell, rc::Rc};

use threerender_traits::mesh::{texture, vertex, Mesh, Vertex};

use crate::BuiltInEntityOption;

#[derive(Debug)]
pub struct Square {
    vertex: Rc<RefCell<Vec<Vertex>>>,
    index: Vec<u16>,
}

impl Square {
    pub fn new(options: Option<BuiltInEntityOption>) -> Self {
        let this = Self::default();
        if options.map(|v| v.use_texture).unwrap_or_default() {
            let texs = vec![
                texture([0., 1.]),
                texture([1., 1.]),
                texture([1., 0.]),
                texture([0., 0.]),
            ];

            for (idx, vert) in this.vertex.borrow_mut().iter_mut().enumerate() {
                let tex = *texs.get(idx % 4).expect("`texs` length is incorrect");
                vert.tex = tex;
            }
        }
        this
    }
}

impl Default for Square {
    fn default() -> Self {
        Self {
            vertex: Rc::new(RefCell::new(vec![
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
            ])),
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

impl Mesh for Square {
    fn vertex(&self) -> Rc<RefCell<Vec<Vertex>>> {
        self.vertex.clone()
    }

    fn index(&self) -> Option<&[u16]> {
        Some(&self.index)
    }
}
