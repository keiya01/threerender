use std::{cell::RefCell, f32::consts::PI, rc::Rc};
use threerender_traits::mesh::{texture, texture_vertex, vertex, Mesh, Vertex};

use crate::BuiltInEntityOption;

#[derive(Default, Debug)]
pub struct Sphere {
    vertex: Rc<RefCell<Vec<Vertex>>>,
    index: Vec<u16>,
}

impl Sphere {
    pub fn new(slices: u16, stacks: u16, options: Option<BuiltInEntityOption>) -> Self {
        let (vertex, index) = if options.map(|v| v.use_texture).unwrap_or_default() {
            Self::make_texture_data(slices, stacks)
        } else {
            Self::make_data(slices, stacks)
        };

        Self {
            vertex: Rc::new(RefCell::new(vertex)),
            index,
        }
    }

    fn make_data(slices: u16, stacks: u16) -> (Vec<Vertex>, Vec<u16>) {
        let mut vertices = vec![];
        let mut indices = vec![];
        for j in 0..stacks + 1 {
            let t = (j as f32) / (stacks as f32);
            let y = (PI * t).cos();
            let r = (PI * t).sin();

            // For indices
            let k = (slices + 1) * j;

            for i in 0..slices + 1 {
                // Make vertices
                let s = (i as f32) / (slices as f32);
                let z = r * (2. * PI * s).cos();
                let x = r * (2. * PI * s).sin();
                vertices.push(vertex([x, y, z, 1.], [x, y, z]));

                if j < stacks && i < slices {
                    // Make indices
                    let k0 = k + i;
                    let k1 = k0 + 1;
                    let k2 = k1 + slices;
                    let k3 = k2 + 1;

                    indices.push(k0);
                    indices.push(k2);
                    indices.push(k3);

                    indices.push(k0);
                    indices.push(k3);
                    indices.push(k1);
                }
            }
        }
        (vertices, indices)
    }

    fn make_texture_data(slices: u16, stacks: u16) -> (Vec<Vertex>, Vec<u16>) {
        let mut vertices = vec![];
        let mut indices = vec![];

        for j in 0..stacks + 1 {
            let t = (j as f32) / (stacks as f32);
            let y = (PI * t).cos();
            let r = (PI * t).sin();

            // For indices
            let k = (slices + 1) * j;

            let ty = (j as f32) / (stacks as f32);

            for i in 0..slices + 1 {
                // Make vertices
                let s = (i as f32) / (slices as f32);
                let z = r * (2. * PI * s).cos();
                let x = r * (2. * PI * s).sin();

                let ver = vertex([x, y, z, 1.], [x, y, z]);
                let tex = texture([(i as f32 / slices as f32), ty]);

                vertices.push(texture_vertex(ver, tex));

                if j < stacks && i < slices {
                    // Make indices
                    let k0 = k + i;
                    let k1 = k0 + 1;
                    let k2 = k1 + slices;
                    let k3 = k2 + 1;

                    indices.push(k0);
                    indices.push(k2);
                    indices.push(k3);

                    indices.push(k0);
                    indices.push(k3);
                    indices.push(k1);
                }
            }
        }
        (vertices, indices)
    }
}

impl Mesh for Sphere {
    fn vertex(&self) -> Rc<RefCell<Vec<Vertex>>> {
        self.vertex.clone()
    }

    fn index(&self) -> Option<&[u16]> {
        Some(&self.index)
    }
}
