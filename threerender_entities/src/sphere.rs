use std::f32::consts::PI;
use threerender_traits::mesh::{
    texture, texture_vertex, vertex, EntityMesh, Mesh, TextureFormat,
    TextureMesh, TextureVertex, Vertex,
};

use crate::TextureDescriptor;

#[derive(Default, Debug)]
pub struct Sphere {
    vertex: Option<Vec<Vertex>>,
    index: Option<Vec<u16>>,

    slices: u16,
    stacks: u16,

    texture_descriptor: Option<TextureDescriptor>,
    texture: Option<Vec<TextureVertex>>,
}

impl Sphere {
    pub fn new(slices: u16, stacks: u16, texture_descriptor: Option<TextureDescriptor>) -> Self {
        Self {
            vertex: None,
            index: None,
            texture_descriptor,
            texture: None,
            slices,
            stacks,
        }
    }

    fn make_data(&self) -> (Vec<Vertex>, Vec<u16>) {
        let slices = self.slices;
        let stacks = self.stacks;

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

    fn make_texture_data(&self) -> (Vec<TextureVertex>, Vec<u16>) {
        let slices = self.slices;
        let stacks = self.stacks;

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

impl EntityMesh for Sphere {
    fn vertex(&self) -> &[Vertex] {
        self.vertex
            .as_ref()
            .expect("You should invoke `use_entity` or `use_texture`.")
    }

    fn index(&self) -> Option<&[u16]> {
        self.index.as_ref().map(|v| &v[..])
    }

    fn use_entity(mut self) -> Mesh {
        let (vertex, index) = self.make_data();
        self.vertex = Some(vertex);
        self.index = Some(index);
        Mesh::Entity(Box::new(self))
    }
}

impl TextureMesh for Sphere {
    fn texture(&self) -> Option<&[TextureVertex]> {
        self.texture.as_ref().map(|t| &t[..])
    }
    fn width(&self) -> u32 {
        self.texture_descriptor.as_ref().unwrap().width
    }
    fn height(&self) -> u32 {
        self.texture_descriptor.as_ref().unwrap().height
    }
    fn format(&self) -> &TextureFormat {
        &self.texture_descriptor.as_ref().unwrap().format
    }
    fn data(&self) -> &[u8] {
        &self.texture_descriptor.as_ref().unwrap().data
    }

    fn use_texture(mut self) -> Mesh {
        let (vertex, index) = self.make_texture_data();
        self.texture = Some(vertex);
        self.index = Some(index);
        Mesh::Texture(Box::new(self))
    }
}
